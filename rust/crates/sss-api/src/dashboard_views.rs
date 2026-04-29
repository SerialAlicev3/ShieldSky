use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sss_storage::{PassiveRegionRunLog, PassiveRunOrigin, PassiveSourceHealthSample};

use crate::canonical_views::{
    build_canonical_events_response, CanonicalEventStatus, CanonicalEventsQuery,
    CanonicalPassiveEvent,
};
use crate::map_views::{
    build_event_map_response, build_region_map_response, build_site_map_response, EventMapItem,
    MapEventsQuery, MapPriority, MapSitesQuery, RegionMapItem, SiteMapItem,
};
use crate::semantic_timeline::{
    build_dashboard_semantic_timeline, SemanticTimelineEntry, SemanticTimelineQuery,
};
use crate::state::{
    now_unix_seconds, AppError, AppState, PassiveOperationsStatus, PassiveSourceHealthStatus,
};

#[derive(Debug, Clone, Deserialize)]
pub struct PassiveDashboardSummaryQuery {
    pub region_id: Option<String>,
    pub limit: Option<usize>,
    pub semantic_status: Option<CanonicalEventStatus>,
    pub semantic_window_hours: Option<u64>,
    pub min_pressure_priority: Option<MapPriority>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerHealthStatus {
    NoRegions,
    NeverRun,
    Healthy,
    Due,
    Stale,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PassiveWorkerHealth {
    pub status: WorkerHealthStatus,
    pub latest_run_at_unix_seconds: Option<i64>,
    pub seconds_since_latest_run: Option<i64>,
    pub latest_run_status: Option<String>,
    pub latest_run_region_ids: Vec<String>,
    pub active_region_lease_count: usize,
    pub worker_heartbeat_count: usize,
    pub active_worker_count: usize,
    pub stale_worker_count: usize,
    pub running_region_ids: Vec<String>,
    pub recommendation: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PassiveSourceHealth {
    pub source: String,
    pub region_id: Option<String>,
    pub success_count: usize,
    pub failure_count: usize,
    pub consecutive_failure_count: usize,
    pub success_rate: f64,
    pub reliability_score: f64,
    pub health_status: PassiveSourceHealthStatus,
    pub latest_generated_at_unix_seconds: Option<i64>,
    pub staleness_seconds: Option<i64>,
    pub last_error: Option<String>,
    pub recovery_hint: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PassiveDashboardSummary {
    pub generated_at_unix_seconds: i64,
    pub region_id: Option<String>,
    pub operations: PassiveOperationsStatus,
    pub worker_health: PassiveWorkerHealth,
    pub source_health: Vec<PassiveSourceHealth>,
    pub top_regions: Vec<RegionMapItem>,
    pub top_sites: Vec<SiteMapItem>,
    pub top_events: Vec<EventMapItem>,
    pub top_canonical_events: Vec<CanonicalPassiveEvent>,
    pub semantic_filter: Option<CanonicalEventStatus>,
    pub semantic_timeline: Vec<SemanticTimelineEntry>,
    pub recent_runs: Vec<PassiveRegionRunLog>,
    pub narrative: String,
}

type DashboardFocus = (Vec<RegionMapItem>, Vec<SiteMapItem>, Vec<EventMapItem>);

pub fn build_passive_dashboard_summary(
    state: &AppState,
    query: &PassiveDashboardSummaryQuery,
) -> Result<PassiveDashboardSummary, AppError> {
    let generated_at = now_unix_seconds();
    let limit = query.limit.unwrap_or(5).clamp(1, 25);
    let operations = state.passive_operations_status()?;
    let recent_source_runs = state.passive_region_run_logs(25, query.region_id.as_deref())?;
    let recent_runs = recent_source_runs
        .iter()
        .filter(|run| run.origin == PassiveRunOrigin::Worker)
        .take(10)
        .cloned()
        .collect::<Vec<_>>();
    let worker_health = worker_health(
        generated_at,
        &operations,
        recent_runs
            .first()
            .or(operations.latest_region_run.as_ref()),
    );
    let persisted_source_health = persisted_source_health(
        &state.passive_source_health_samples(250, None, None)?,
        query.region_id.as_deref(),
    );
    let source_health = if persisted_source_health.is_empty() {
        source_health_from_runs(&recent_source_runs, query.region_id.as_deref())
    } else {
        persisted_source_health
    };
    let regions = filtered_regions(state, query)?;
    let canonical_response = build_canonical_events_response(
        state,
        &CanonicalEventsQuery {
            region_id: query.region_id.clone(),
            window_hours: Some(24),
            bucket_minutes: Some(60),
            limit: Some(limit),
        },
    )?;
    let semantic_timeline = build_dashboard_semantic_timeline(
        state,
        query.region_id.as_deref(),
        &SemanticTimelineQuery {
            limit: Some(limit.saturating_mul(2)),
            window_hours: query.semantic_window_hours.or(Some(24 * 14)),
            status: query.semantic_status,
        },
    )?;
    let (top_regions, top_sites, top_events) = dashboard_focus(state, regions, limit)?;

    Ok(PassiveDashboardSummary {
        generated_at_unix_seconds: generated_at,
        region_id: query.region_id.clone(),
        narrative: dashboard_narrative(
            &operations,
            &worker_health,
            top_regions.len(),
            top_sites.len(),
            canonical_response.canonical_event_count,
        ),
        operations,
        worker_health,
        source_health,
        top_regions,
        top_sites,
        top_events,
        top_canonical_events: canonical_response.events,
        semantic_filter: query.semantic_status,
        semantic_timeline: semantic_timeline.entries,
        recent_runs,
    })
}

#[derive(Debug, Clone, Default)]
struct SourceHealthAccumulator {
    success_count: usize,
    failure_count: usize,
    consecutive_failure_count: usize,
    failure_streak_open: bool,
    latest_generated_at_unix_seconds: Option<i64>,
    last_error: Option<String>,
}

fn source_health_from_runs(
    recent_runs: &[PassiveRegionRunLog],
    region_id: Option<&str>,
) -> Vec<PassiveSourceHealth> {
    let mut sources = BTreeMap::<String, SourceHealthAccumulator>::new();

    for run in recent_runs {
        for source in &run.sources_used {
            let entry = sources.entry(source.clone()).or_default();
            if entry.success_count == 0 && entry.failure_count == 0 {
                entry.failure_streak_open = true;
            }
            entry.success_count += 1;
            entry.failure_streak_open = false;
        }
        for error in &run.source_errors {
            let source = infer_source_from_error(error);
            let entry = sources.entry(source).or_default();
            if entry.success_count == 0 && entry.failure_count == 0 {
                entry.failure_streak_open = true;
            }
            entry.failure_count += 1;
            if entry.failure_streak_open {
                entry.consecutive_failure_count += 1;
            }
            if entry.last_error.is_none() {
                entry.last_error = Some(error.clone());
            }
        }
    }

    let mut health = sources
        .into_iter()
        .map(|(source, accumulator)| {
            let total = accumulator
                .success_count
                .saturating_add(accumulator.failure_count);
            let success_rate = if total == 0 {
                1.0
            } else {
                let success = u32::try_from(accumulator.success_count).unwrap_or(u32::MAX);
                let total = u32::try_from(total).unwrap_or(u32::MAX);
                f64::from(success) / f64::from(total)
            };
            let staleness_seconds = None;
            let health_status = dashboard_source_health_status(
                success_rate,
                accumulator.failure_count,
                staleness_seconds,
            );
            PassiveSourceHealth {
                source,
                region_id: region_id.map(ToOwned::to_owned),
                success_count: accumulator.success_count,
                failure_count: accumulator.failure_count,
                consecutive_failure_count: accumulator.consecutive_failure_count,
                success_rate,
                reliability_score: success_rate,
                health_status,
                latest_generated_at_unix_seconds: None,
                staleness_seconds,
                last_error: accumulator.last_error,
                recovery_hint: dashboard_source_recovery_hint(
                    health_status,
                    accumulator.consecutive_failure_count,
                    staleness_seconds,
                ),
            }
        })
        .collect::<Vec<_>>();

    health.sort_by(|left, right| {
        left.reliability_score
            .total_cmp(&right.reliability_score)
            .then_with(|| left.source.cmp(&right.source))
    });
    health
}

fn persisted_source_health(
    samples: &[PassiveSourceHealthSample],
    region_id: Option<&str>,
) -> Vec<PassiveSourceHealth> {
    let mut sources = BTreeMap::<String, SourceHealthAccumulator>::new();

    for sample in samples
        .iter()
        .filter(|sample| region_id.is_none() || sample.region_id.as_deref() == region_id)
    {
        let entry = sources
            .entry(format!("{:?}", sample.source_kind))
            .or_default();
        if entry.success_count == 0 && entry.failure_count == 0 {
            entry.failure_streak_open = true;
        }
        if sample.fetched {
            entry.success_count += 1;
            entry.failure_streak_open = false;
        } else {
            entry.failure_count += 1;
            if entry.failure_streak_open {
                entry.consecutive_failure_count += 1;
            }
            if entry.last_error.is_none() {
                entry.last_error = Some(sample.detail.clone());
            }
        }
        if entry
            .latest_generated_at_unix_seconds
            .is_none_or(|current| sample.generated_at_unix_seconds > current)
        {
            entry.latest_generated_at_unix_seconds = Some(sample.generated_at_unix_seconds);
        }
    }

    let now = now_unix_seconds();
    let mut health = sources
        .into_iter()
        .map(|(source, accumulator)| {
            let total = accumulator
                .success_count
                .saturating_add(accumulator.failure_count);
            let success_rate = if total == 0 {
                1.0
            } else {
                let success = u32::try_from(accumulator.success_count).unwrap_or(u32::MAX);
                let total = u32::try_from(total).unwrap_or(u32::MAX);
                f64::from(success) / f64::from(total)
            };
            let staleness_seconds = accumulator
                .latest_generated_at_unix_seconds
                .map(|latest| now.saturating_sub(latest));
            let reliability_score =
                dashboard_source_reliability_score(success_rate, staleness_seconds);
            let health_status = dashboard_source_health_status(
                success_rate,
                accumulator.failure_count,
                staleness_seconds,
            );
            PassiveSourceHealth {
                source,
                region_id: region_id.map(ToOwned::to_owned),
                success_count: accumulator.success_count,
                failure_count: accumulator.failure_count,
                consecutive_failure_count: accumulator.consecutive_failure_count,
                success_rate,
                reliability_score,
                health_status,
                latest_generated_at_unix_seconds: accumulator.latest_generated_at_unix_seconds,
                staleness_seconds,
                last_error: accumulator.last_error,
                recovery_hint: dashboard_source_recovery_hint(
                    health_status,
                    accumulator.consecutive_failure_count,
                    staleness_seconds,
                ),
            }
        })
        .collect::<Vec<_>>();

    health.sort_by(|left, right| {
        left.reliability_score
            .total_cmp(&right.reliability_score)
            .then_with(|| left.source.cmp(&right.source))
    });
    health
}

fn dashboard_source_reliability_score(success_rate: f64, staleness_seconds: Option<i64>) -> f64 {
    let recency_factor = match staleness_seconds {
        Some(seconds) if seconds > 21_600 => 0.5,
        Some(seconds) if seconds > 3_600 => 0.8,
        Some(_) => 1.0,
        None => 0.5,
    };
    (success_rate * recency_factor).clamp(0.0, 1.0)
}

fn dashboard_source_health_status(
    success_rate: f64,
    failure_count: usize,
    staleness_seconds: Option<i64>,
) -> PassiveSourceHealthStatus {
    if staleness_seconds.is_some_and(|seconds| seconds > 21_600) {
        PassiveSourceHealthStatus::Stale
    } else if failure_count > 0 || success_rate < 0.8 {
        PassiveSourceHealthStatus::Degraded
    } else if success_rate < 0.95 {
        PassiveSourceHealthStatus::Watch
    } else {
        PassiveSourceHealthStatus::Healthy
    }
}

fn dashboard_source_recovery_hint(
    health_status: PassiveSourceHealthStatus,
    consecutive_failure_count: usize,
    staleness_seconds: Option<i64>,
) -> String {
    if staleness_seconds.is_some_and(|seconds| seconds > 21_600) {
        return "No recent source sample; run a live scan or inspect worker cadence.".to_string();
    }
    if consecutive_failure_count >= 3 {
        return format!(
            "{consecutive_failure_count} consecutive failures; check upstream quota, credentials, and timeout."
        );
    }
    if consecutive_failure_count > 0 {
        return format!(
            "{consecutive_failure_count} latest failure; retry with backoff and inspect source samples."
        );
    }
    match health_status {
        PassiveSourceHealthStatus::Healthy => {
            "Source is healthy; keep current cadence.".to_string()
        }
        PassiveSourceHealthStatus::Watch => {
            "Source is watchlisted; monitor the next scan cycle.".to_string()
        }
        PassiveSourceHealthStatus::Degraded => {
            "Source is degraded; inspect source samples and use fallback context if available."
                .to_string()
        }
        PassiveSourceHealthStatus::Stale => {
            "Source is stale; run a fresh scan before relying on this context.".to_string()
        }
    }
}

fn infer_source_from_error(error: &str) -> String {
    let normalized = error.to_ascii_lowercase();
    if normalized.contains("opensky") || normalized.contains("ads-b") || normalized.contains("adsb")
    {
        "Adsb".to_string()
    } else if normalized.contains("firms")
        || normalized.contains("fire")
        || normalized.contains("smoke")
    {
        "FireSmoke".to_string()
    } else if normalized.contains("meteo") || normalized.contains("weather") {
        "Weather".to_string()
    } else {
        "Unknown".to_string()
    }
}

fn dashboard_focus(
    state: &AppState,
    regions: Vec<RegionMapItem>,
    limit: usize,
) -> Result<DashboardFocus, AppError> {
    let mut top_sites = Vec::new();
    let mut top_events = Vec::new();

    for region in &regions {
        let site_response = build_site_map_response(
            state,
            &MapSitesQuery {
                region_id: region.region_id.clone(),
                site_type: None,
                min_priority: None,
                observed_only: None,
                elevated_only: None,
            },
        )?;
        top_sites.extend(site_response.sites);

        let event_response = build_event_map_response(
            state,
            &MapEventsQuery {
                region_id: region.region_id.clone(),
                future_only: None,
                event_type: None,
                severity: None,
                after_unix_seconds: None,
                before_unix_seconds: None,
                limit: Some(limit * 4),
            },
        )?;
        top_events.extend(event_response.events);
    }

    let mut top_regions = regions;
    top_regions.sort_by(|left, right| {
        right
            .stale_lease_count
            .cmp(&left.stale_lease_count)
            .then_with(|| right.stale_worker_count.cmp(&left.stale_worker_count))
            .then_with(|| {
                right
                    .recent_lease_loss_count
                    .cmp(&left.recent_lease_loss_count)
            })
            .then_with(|| {
                right
                    .recent_failed_run_count
                    .cmp(&left.recent_failed_run_count)
            })
            .then_with(|| {
                right
                    .operational_pressure_priority
                    .cmp(&left.operational_pressure_priority)
            })
            .then_with(|| right.critical_events.cmp(&left.critical_events))
            .then_with(|| right.recent_events.cmp(&left.recent_events))
            .then_with(|| right.seeds_elevated.cmp(&left.seeds_elevated))
            .then_with(|| region_focus_rank(left).cmp(&region_focus_rank(right)))
    });
    top_regions.truncate(limit);

    top_sites.sort_by(|left, right| {
        right
            .risk_score
            .partial_cmp(&left.risk_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| right.confidence.total_cmp(&left.confidence))
    });
    top_sites.truncate(limit);

    top_events.sort_by(|left, right| {
        right
            .risk_score
            .partial_cmp(&left.risk_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                right
                    .observed_at_unix_seconds
                    .cmp(&left.observed_at_unix_seconds)
            })
    });
    top_events.truncate(limit);

    Ok((top_regions, top_sites, top_events))
}

fn region_focus_rank(region: &RegionMapItem) -> u8 {
    if region.country_code.as_deref() == Some("PT")
        || region.region_id.contains("portugal")
        || region.region_id.contains("alentejo")
    {
        0
    } else if region.region_id.contains("iberia") {
        1
    } else if matches!(
        region.country_code.as_deref(),
        Some("ES" | "FR" | "IT" | "DE")
    ) {
        2
    } else {
        3
    }
}

fn filtered_regions(
    state: &AppState,
    query: &PassiveDashboardSummaryQuery,
) -> Result<Vec<RegionMapItem>, AppError> {
    let mut regions = build_region_map_response(state)?.regions;
    if let Some(region_id) = query.region_id.as_ref() {
        regions.retain(|region| &region.region_id == region_id);
        if regions.is_empty() {
            return Err(AppError::SiteNotFound(format!(
                "passive region not found: {region_id}"
            )));
        }
    }
    if let Some(min_priority) = query.min_pressure_priority {
        regions.retain(|region| region.operational_pressure_priority >= min_priority);
    }
    Ok(regions)
}

fn worker_health(
    now: i64,
    operations: &PassiveOperationsStatus,
    latest_run: Option<&PassiveRegionRunLog>,
) -> PassiveWorkerHealth {
    let stale_cutoff = now.saturating_sub(180);
    let active_worker_count = operations
        .worker_heartbeats
        .iter()
        .filter(|heartbeat| heartbeat.last_heartbeat_unix_seconds >= stale_cutoff)
        .count();
    let stale_worker_count = operations
        .worker_heartbeats
        .len()
        .saturating_sub(active_worker_count);
    let running_region_ids = operations
        .active_region_leases
        .iter()
        .map(|lease| lease.region_id.clone())
        .collect::<Vec<_>>();
    let status = match latest_run {
        None if operations.enabled_region_count == 0 => WorkerHealthStatus::NoRegions,
        None if active_worker_count > 0 || operations.active_region_lease_count > 0 => {
            WorkerHealthStatus::Healthy
        }
        None => WorkerHealthStatus::NeverRun,
        Some(run) if operations.due_region_count > 0 => {
            let age = now.saturating_sub(run.finished_at_unix_seconds);
            if age > 7_200 {
                WorkerHealthStatus::Stale
            } else {
                WorkerHealthStatus::Due
            }
        }
        Some(run) => {
            let age = now.saturating_sub(run.finished_at_unix_seconds);
            if age > 7_200 && active_worker_count == 0 {
                WorkerHealthStatus::Stale
            } else {
                WorkerHealthStatus::Healthy
            }
        }
    };

    PassiveWorkerHealth {
        status,
        latest_run_at_unix_seconds: latest_run.map(|run| run.finished_at_unix_seconds),
        seconds_since_latest_run: latest_run
            .map(|run| now.saturating_sub(run.finished_at_unix_seconds)),
        latest_run_status: latest_run.map(|run| format!("{:?}", run.status)),
        latest_run_region_ids: latest_run.map_or_else(Vec::new, |run| run.region_ids.clone()),
        active_region_lease_count: operations.active_region_lease_count,
        worker_heartbeat_count: operations.worker_heartbeat_count,
        active_worker_count,
        stale_worker_count,
        running_region_ids,
        recommendation: worker_recommendation(status, operations),
    }
}

fn worker_recommendation(
    status: WorkerHealthStatus,
    operations: &PassiveOperationsStatus,
) -> String {
    match status {
        WorkerHealthStatus::NoRegions => {
            "No passive regions configured. Add regional targets before starting autonomous observation."
                .to_string()
        }
        WorkerHealthStatus::NeverRun => {
            "Passive regions exist but no worker run has been recorded yet. Start sss-passive-worker or run the regional scheduler once."
                .to_string()
        }
        WorkerHealthStatus::Healthy if operations.elevated_seed_count > 0 => {
            format!(
                "Worker is current. {} elevated seeds should remain under operator review.",
                operations.elevated_seed_count
            )
        }
        WorkerHealthStatus::Healthy => {
            "Worker is current and passive observation is within cadence.".to_string()
        }
        WorkerHealthStatus::Due => {
            format!(
                "{} enabled regions are due. Let the worker run or trigger /v1/passive/regions/run.",
                operations.due_region_count
            )
        }
        WorkerHealthStatus::Stale => {
            "Latest passive worker run is stale. Check worker process, source credentials, and storage connectivity."
                .to_string()
        }
    }
}

fn dashboard_narrative(
    operations: &PassiveOperationsStatus,
    worker_health: &PassiveWorkerHealth,
    region_count: usize,
    site_count: usize,
    event_count: usize,
) -> String {
    format!(
        "Passive intelligence is tracking {} enabled regions, {} seeds and {} observed sites. Dashboard focus: {} regions, {} sites and {} events. Worker status: {:?}. {}",
        operations.enabled_region_count,
        operations.seed_count,
        operations.observed_seed_count,
        region_count,
        site_count,
        event_count,
        worker_health.status,
        worker_health.recommendation
    )
}
