use crate::map_views::{build_region_map_response, MapPriority, RegionMapItem};
use crate::operational_timeline::{
    build_region_operational_timeline, OperationalPressureTimelineBucket, OperationalTimelineQuery,
};
use crate::state::{
    now_unix_seconds, AppError, AppState, PassiveSourceHealthStatus, PassiveWorkerDiagnostics,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PassiveRegionRemediationPosture {
    Healthy,
    AttentionNeeded,
    Degraded,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PassiveRegionRemediationSource {
    Overview,
    Diagnostics,
    MapPressure,
    OperationalTimeline,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveRegionRemediationAction {
    pub action_id: String,
    pub priority: MapPriority,
    pub title: String,
    pub reason: String,
    pub suggested_operator_step: String,
    pub source: PassiveRegionRemediationSource,
    pub related_sources: Vec<String>,
    pub suggested_read_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveRegionRemediationEvidence {
    pub overview_narrative: String,
    pub operational_summary: Option<String>,
    pub diagnostics_recommendation: String,
    pub pressure_buckets: Vec<PassiveRegionRemediationPressureBucket>,
    pub active_lease_count: usize,
    pub stale_lease_count: usize,
    pub active_worker_count: usize,
    pub stale_worker_count: usize,
    pub recent_failed_run_count: usize,
    pub recent_partial_run_count: usize,
    pub recent_lease_loss_count: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveRegionRemediationPressureBucket {
    pub label: String,
    pub start_unix_seconds: i64,
    pub end_unix_seconds: i64,
    pub pressure_priority: MapPriority,
    pub pressure_score: f64,
    pub failed_run_count: usize,
    pub partial_run_count: usize,
    pub lease_loss_count: usize,
    pub source_failure_count: usize,
    pub dominant_sources: Vec<String>,
    pub summary: String,
    pub suggested_read_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveRegionRemediationResponse {
    pub generated_at_unix_seconds: i64,
    pub region_id: String,
    pub summary: String,
    pub posture: PassiveRegionRemediationPosture,
    pub actions: Vec<PassiveRegionRemediationAction>,
    pub evidence: PassiveRegionRemediationEvidence,
}

pub fn build_passive_region_remediation_response(
    state: &AppState,
    region_id: &str,
    site_limit: usize,
    days: u32,
    stale_after_seconds: i64,
) -> Result<Option<PassiveRegionRemediationResponse>, AppError> {
    let Some(overview) = state.passive_region_overview(region_id, site_limit, days)? else {
        return Ok(None);
    };
    let region = build_region_map_response(state)?
        .regions
        .into_iter()
        .find(|item| item.region_id == region_id);
    let diagnostics = state.passive_worker_diagnostics(12, stale_after_seconds, Some(region_id))?;
    let pressure_buckets = build_region_operational_timeline(
        state,
        region_id,
        &OperationalTimelineQuery {
            window_hours: Some(72),
            bucket_hours: Some(6),
            include_current_snapshot: Some(false),
            include_empty_buckets: Some(false),
            min_priority: Some(MapPriority::High),
        },
    )?
    .map(|timeline| remediation_pressure_buckets(&timeline.buckets))
    .unwrap_or_default();
    let posture = remediation_posture(region.as_ref(), &diagnostics);
    let actions = remediation_actions(
        &overview.narrative,
        region.as_ref(),
        &diagnostics,
        &pressure_buckets,
    );
    let summary = remediation_summary(region.as_ref(), &diagnostics, posture, actions.len());

    Ok(Some(PassiveRegionRemediationResponse {
        generated_at_unix_seconds: now_unix_seconds(),
        region_id: region_id.to_string(),
        summary,
        posture,
        evidence: PassiveRegionRemediationEvidence {
            overview_narrative: overview.narrative,
            operational_summary: region.as_ref().map(|item| item.operational_summary.clone()),
            diagnostics_recommendation: diagnostics.recommendation.clone(),
            pressure_buckets,
            active_lease_count: diagnostics.active_region_lease_count,
            stale_lease_count: diagnostics.stale_region_lease_count,
            active_worker_count: diagnostics.active_worker_heartbeat_count,
            stale_worker_count: diagnostics.stale_worker_heartbeat_count,
            recent_failed_run_count: region
                .as_ref()
                .map_or(0, |item| item.recent_failed_run_count),
            recent_partial_run_count: region
                .as_ref()
                .map_or(0, |item| item.recent_partial_run_count),
            recent_lease_loss_count: region
                .as_ref()
                .map_or(0, |item| item.recent_lease_loss_count),
        },
        actions,
    }))
}

fn remediation_posture(
    region: Option<&RegionMapItem>,
    diagnostics: &PassiveWorkerDiagnostics,
) -> PassiveRegionRemediationPosture {
    if diagnostics.stale_region_lease_count > 0
        || diagnostics.stale_worker_heartbeat_count > 0
        || region.is_some_and(|item| item.recent_lease_loss_count > 0)
    {
        PassiveRegionRemediationPosture::Critical
    } else if region.is_some_and(|item| {
        item.recent_failed_run_count > 0
            || item.status == crate::map_views::RegionMapStatus::Degraded
    }) {
        PassiveRegionRemediationPosture::Degraded
    } else if diagnostics.source_metrics.iter().any(|metric| {
        matches!(
            metric.health_status,
            PassiveSourceHealthStatus::Degraded | PassiveSourceHealthStatus::Stale
        )
    }) || region.is_some_and(|item| item.recent_partial_run_count > 0)
    {
        PassiveRegionRemediationPosture::AttentionNeeded
    } else {
        PassiveRegionRemediationPosture::Healthy
    }
}

#[allow(clippy::too_many_lines)]
fn remediation_actions(
    overview_narrative: &str,
    region: Option<&RegionMapItem>,
    diagnostics: &PassiveWorkerDiagnostics,
    pressure_buckets: &[PassiveRegionRemediationPressureBucket],
) -> Vec<PassiveRegionRemediationAction> {
    let mut actions = Vec::new();
    let region_id = region.map_or("unknown-region", |item| item.region_id.as_str());
    let diagnostics_path = format!("/v1/passive/worker/diagnostics?region_id={region_id}");
    let region_runs_path = format!("/v1/passive/regions/runs?region_id={region_id}");
    let region_overview_path = format!("/v1/passive/regions/{region_id}/overview");
    let source_health_path =
        format!("/v1/passive/source-health/samples?limit=20&region_id={region_id}");
    let operational_timeline_path = format!(
        "/v1/passive/regions/{region_id}/operational-timeline?window_hours=72&bucket_hours=6&include_empty_buckets=false"
    );
    if let Some(hot_bucket) = pressure_buckets.first() {
        actions.push(PassiveRegionRemediationAction {
            action_id: "inspect-operational-pressure-bucket".to_string(),
            priority: hot_bucket.pressure_priority,
            title: "Inspect operational pressure bucket".to_string(),
            reason: format!(
                "{} is {:?} with {} failed runs, {} partial runs, {} lease-loss signals, and {} source failures.",
                hot_bucket.label,
                hot_bucket.pressure_priority,
                hot_bucket.failed_run_count,
                hot_bucket.partial_run_count,
                hot_bucket.lease_loss_count,
                hot_bucket.source_failure_count
            ),
            suggested_operator_step:
                "Open the operational timeline first, then drill into run logs and source health for the same region window."
                    .to_string(),
            source: PassiveRegionRemediationSource::OperationalTimeline,
            related_sources: hot_bucket.dominant_sources.clone(),
            suggested_read_paths: vec![
                operational_timeline_path.clone(),
                region_runs_path.clone(),
                source_health_path.clone(),
            ],
        });
    }
    if diagnostics.stale_region_lease_count > 0 {
        actions.push(PassiveRegionRemediationAction {
            action_id: "review-stale-lease-ownership".to_string(),
            priority: MapPriority::Critical,
            title: "Review stale lease ownership".to_string(),
            reason: format!(
                "{} stale region lease entries remain in scope.",
                diagnostics.stale_region_lease_count
            ),
            suggested_operator_step:
                "Open region diagnostics, inspect lease timestamps, and confirm whether the worker that owned this region is still alive."
                    .to_string(),
            source: PassiveRegionRemediationSource::Diagnostics,
            related_sources: Vec::new(),
            suggested_read_paths: vec![diagnostics_path.clone(), region_runs_path.clone()],
        });
    }
    if diagnostics.stale_worker_heartbeat_count > 0 {
        actions.push(PassiveRegionRemediationAction {
            action_id: "check-worker-liveness".to_string(),
            priority: MapPriority::High,
            title: "Check worker liveness and heartbeat cadence".to_string(),
            reason: format!(
                "{} worker heartbeats are stale for this region scope.",
                diagnostics.stale_worker_heartbeat_count
            ),
            suggested_operator_step:
                "Open stale heartbeats, compare current phases, and prune abandoned worker entries once ownership is confirmed."
                    .to_string(),
            source: PassiveRegionRemediationSource::Diagnostics,
            related_sources: Vec::new(),
            suggested_read_paths: vec![diagnostics_path.clone()],
        });
    }
    if let Some(region) = region {
        if region.recent_lease_loss_count > 0 {
            actions.push(PassiveRegionRemediationAction {
                action_id: "audit-lease-loss".to_string(),
                priority: MapPriority::Critical,
                title: "Audit recent lease loss".to_string(),
                reason: format!(
                    "{} lease-loss signals were recorded in recent runs.",
                    region.recent_lease_loss_count
                ),
                suggested_operator_step:
                    "Inspect recent failed runs and lease renewal timing before trusting autonomous coverage in this region."
                        .to_string(),
                source: PassiveRegionRemediationSource::MapPressure,
                related_sources: Vec::new(),
                suggested_read_paths: vec![region_runs_path.clone(), diagnostics_path.clone()],
            });
        }
        if region.recent_failed_run_count > 0 || region.recent_partial_run_count > 0 {
            actions.push(PassiveRegionRemediationAction {
                action_id: "review-failed-runs".to_string(),
                priority: if region.recent_failed_run_count > 0 {
                    MapPriority::High
                } else {
                    MapPriority::Medium
                },
                title: "Review failed and partial runs".to_string(),
                reason: format!(
                    "{} failed runs and {} partial runs were observed recently.",
                    region.recent_failed_run_count, region.recent_partial_run_count
                ),
                suggested_operator_step:
                    "Read run logs and source errors for the region before assuming data freshness is intact."
                        .to_string(),
                source: PassiveRegionRemediationSource::MapPressure,
                related_sources: Vec::new(),
                suggested_read_paths: vec![
                    region_runs_path.clone(),
                    diagnostics_path.clone(),
                    region_overview_path.clone(),
                ],
            });
        }
    }
    let degraded_sources = diagnostics
        .source_metrics
        .iter()
        .filter(|metric| {
            matches!(
                metric.health_status,
                PassiveSourceHealthStatus::Degraded | PassiveSourceHealthStatus::Stale
            )
        })
        .map(|metric| metric.source.clone())
        .collect::<Vec<_>>();
    if !degraded_sources.is_empty() {
        actions.push(PassiveRegionRemediationAction {
            action_id: "inspect-source-degradation".to_string(),
            priority: MapPriority::Medium,
            title: "Inspect source degradation".to_string(),
            reason: "At least one feed in this region shows failures or degraded reliability.".to_string(),
            suggested_operator_step:
                "Open source health samples and compare the failing feed with the region timeline before escalating."
                    .to_string(),
            source: PassiveRegionRemediationSource::Diagnostics,
            related_sources: degraded_sources,
            suggested_read_paths: vec![
                source_health_path,
                diagnostics_path.clone(),
                operational_timeline_path,
            ],
        });
    }
    if actions.is_empty() {
        actions.push(PassiveRegionRemediationAction {
            action_id: "monitor-region".to_string(),
            priority: MapPriority::Low,
            title: "Monitor region".to_string(),
            reason: overview_narrative.to_string(),
            suggested_operator_step:
                "No intervention needed right now; keep watching diagnostics and region overview for drift."
                    .to_string(),
            source: PassiveRegionRemediationSource::Overview,
            related_sources: Vec::new(),
            suggested_read_paths: vec![region_overview_path, diagnostics_path],
        });
    }
    actions.sort_by(|left, right| right.priority.cmp(&left.priority));
    actions
}

fn remediation_pressure_buckets(
    buckets: &[OperationalPressureTimelineBucket],
) -> Vec<PassiveRegionRemediationPressureBucket> {
    let mut pressure_buckets = buckets
        .iter()
        .filter(|bucket| {
            matches!(
                bucket.pressure_priority,
                MapPriority::Critical | MapPriority::High
            )
        })
        .map(|bucket| PassiveRegionRemediationPressureBucket {
            label: bucket.label.clone(),
            start_unix_seconds: bucket.start_unix_seconds,
            end_unix_seconds: bucket.end_unix_seconds,
            pressure_priority: bucket.pressure_priority,
            pressure_score: bucket.pressure_score,
            failed_run_count: bucket.failed_run_count,
            partial_run_count: bucket.partial_run_count,
            lease_loss_count: bucket.lease_loss_count,
            source_failure_count: bucket.source_failure_count,
            dominant_sources: bucket.dominant_sources.clone(),
            summary: bucket.summary.clone(),
            suggested_read_paths: bucket.suggested_read_paths.clone(),
        })
        .collect::<Vec<_>>();
    pressure_buckets.sort_by(|left, right| {
        right
            .pressure_priority
            .cmp(&left.pressure_priority)
            .then_with(|| {
                right
                    .pressure_score
                    .partial_cmp(&left.pressure_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| right.end_unix_seconds.cmp(&left.end_unix_seconds))
    });
    pressure_buckets.truncate(3);
    pressure_buckets
}

fn remediation_summary(
    region: Option<&RegionMapItem>,
    diagnostics: &PassiveWorkerDiagnostics,
    posture: PassiveRegionRemediationPosture,
    action_count: usize,
) -> String {
    let latest_run_status = region
        .and_then(|item| item.latest_run_status.as_deref())
        .unwrap_or("unknown");
    format!(
        "{:?} posture with {} recommended next reads. {} active leases, {} stale leases, {} active workers, {} stale workers; latest run {}.",
        posture,
        action_count,
        diagnostics.active_region_lease_count,
        diagnostics.stale_region_lease_count,
        diagnostics.active_worker_heartbeat_count,
        diagnostics.stale_worker_heartbeat_count,
        latest_run_status
    )
}
