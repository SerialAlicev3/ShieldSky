use serde::{Deserialize, Serialize};
use std::env;

use crate::canonical_views::CanonicalEventStatus;
use crate::dashboard_views::{
    build_passive_dashboard_summary, PassiveDashboardSummary, PassiveDashboardSummaryQuery,
};
use crate::map_views::{MapPriority, RegionMapItem, SiteMapItem};
use crate::state::{
    default_passive_region_requests, now_unix_seconds, AppError, AppState, NeoRiskFeed,
    NeoRiskObject, PassiveSourceReadinessLevel,
};
use serde_json::json;

#[derive(Debug, Clone, Deserialize)]
pub struct PassiveCommandCenterSummaryQuery {
    pub region_id: Option<String>,
    pub limit: Option<usize>,
    pub semantic_status: Option<CanonicalEventStatus>,
    pub semantic_window_hours: Option<u64>,
    pub min_pressure_priority: Option<MapPriority>,
    pub attention_kind: Option<PassiveCommandCenterAttentionKind>,
    pub min_attention_priority: Option<MapPriority>,
    pub heartbeat_retention_seconds: Option<i64>,
    pub source_retention_seconds: Option<i64>,
    pub source_kind: Option<sss_passive_scanner::PassiveSourceKind>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PassiveCommandCenterAction {
    pub action_id: String,
    pub title: String,
    pub reason: String,
    pub method: &'static str,
    pub path: String,
    pub payload: Option<serde_json::Value>,
    pub confirmation_read_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PassiveCommandCenterMaintenance {
    pub heartbeat_retention_seconds: i64,
    pub source_retention_seconds: i64,
    pub stale_heartbeat_count: usize,
    pub source_health_prune_candidate_count: usize,
    pub source_kind: Option<sss_passive_scanner::PassiveSourceKind>,
    pub region_id: Option<String>,
    pub suggested_actions: Vec<PassiveCommandCenterAction>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PassiveCommandCenterRegionHighlight {
    pub region_id: String,
    pub name: String,
    pub operational_pressure_priority: crate::map_views::MapPriority,
    pub dominant_status: Option<crate::canonical_views::CanonicalEventStatus>,
    pub risk_delta_classification: Option<crate::canonical_views::RiskDeltaType>,
    pub risk_delta_explanation: Option<String>,
    pub narrative_summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PassiveCommandCenterSiteHighlight {
    pub site_id: String,
    pub name: String,
    pub site_type: crate::map_views::SiteMapType,
    pub risk_score: f64,
    pub risk_trend: crate::map_views::RiskTrend,
    pub top_canonical_status: Option<crate::canonical_views::CanonicalEventStatus>,
    pub recommendation_review_state: Option<sss_storage::PassiveRecommendationReviewState>,
    pub has_recommendation: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PassiveCommandCenterEventHighlight {
    pub canonical_event_id: String,
    pub site_id: String,
    pub site_name: String,
    pub event_type: String,
    pub status: crate::canonical_views::CanonicalEventStatus,
    pub temporal_phase: crate::canonical_views::TemporalPhase,
    pub operational_readout: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PassiveCommandCenterHighlights {
    pub top_region: Option<PassiveCommandCenterRegionHighlight>,
    pub top_site: Option<PassiveCommandCenterSiteHighlight>,
    pub top_event: Option<PassiveCommandCenterEventHighlight>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PassiveCommandCenterAttentionKind {
    Maintenance,
    Neo,
    CanonicalEvent,
    Site,
    Region,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PassiveCommandCenterAttentionItem {
    pub item_id: String,
    pub kind: PassiveCommandCenterAttentionKind,
    pub priority: MapPriority,
    pub urgency_label: String,
    pub age_seconds: Option<i64>,
    pub age_bucket: Option<crate::map_views::ReviewAgeBucket>,
    pub title: String,
    pub reason: String,
    pub primary_action_label: String,
    pub region_id: Option<String>,
    pub site_id: Option<String>,
    pub canonical_event_id: Option<String>,
    pub action_path: Option<String>,
    pub operator_state: Option<String>,
    pub confirmation_read_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PassiveCommandCenterSummary {
    pub generated_at_unix_seconds: i64,
    pub focus_region_id: Option<String>,
    pub summary: String,
    pub highlights: PassiveCommandCenterHighlights,
    pub attention_queue: Vec<PassiveCommandCenterAttentionItem>,
    pub operator_paths: Vec<String>,
    pub focus_paths: Vec<String>,
    pub dashboard: PassiveDashboardSummary,
    pub maintenance: PassiveCommandCenterMaintenance,
}

pub fn build_passive_command_center_summary(
    state: &AppState,
    query: &PassiveCommandCenterSummaryQuery,
) -> Result<PassiveCommandCenterSummary, AppError> {
    let dashboard = build_passive_dashboard_summary(
        state,
        &PassiveDashboardSummaryQuery {
            region_id: query.region_id.clone(),
            limit: query.limit,
            semantic_status: query.semantic_status,
            semantic_window_hours: query.semantic_window_hours,
            min_pressure_priority: query.min_pressure_priority,
        },
    )?;
    let generated_at_unix_seconds = now_unix_seconds();
    let focus_region_id = query.region_id.clone().or_else(|| {
        preferred_focus_region(&dashboard.top_regions).map(|region| region.region_id.clone())
    });
    let maintenance = build_maintenance_projection(
        state,
        generated_at_unix_seconds,
        query.heartbeat_retention_seconds.unwrap_or(86_400).max(1),
        query.source_retention_seconds.unwrap_or(604_800).max(1),
        query.source_kind,
        focus_region_id.as_deref(),
    )?;
    let operator_paths = command_center_operator_paths();
    let focus_paths = focus_region_paths(focus_region_id.as_deref());
    let summary =
        build_command_center_summary(&dashboard, &maintenance, focus_region_id.as_deref());
    let highlights = build_command_center_highlights(&dashboard);
    let attention_queue = build_attention_queue(
        state,
        &dashboard,
        &maintenance,
        query.limit.unwrap_or(5),
        query.attention_kind,
        query.min_attention_priority,
    );

    Ok(PassiveCommandCenterSummary {
        generated_at_unix_seconds,
        focus_region_id,
        summary,
        highlights,
        attention_queue,
        operator_paths,
        focus_paths,
        dashboard,
        maintenance,
    })
}

fn preferred_focus_region(regions: &[RegionMapItem]) -> Option<&RegionMapItem> {
    regions
        .iter()
        .find(|region| {
            region.country_code.as_deref() == Some("PT")
                || region.region_id.contains("portugal")
                || region.region_id.contains("alentejo")
        })
        .or_else(|| regions.first())
}

fn build_maintenance_projection(
    state: &AppState,
    generated_at_unix_seconds: i64,
    heartbeat_retention_seconds: i64,
    source_retention_seconds: i64,
    source_kind: Option<sss_passive_scanner::PassiveSourceKind>,
    region_id: Option<&str>,
) -> Result<PassiveCommandCenterMaintenance, AppError> {
    let heartbeat_cutoff = generated_at_unix_seconds.saturating_sub(heartbeat_retention_seconds);
    let source_cutoff = generated_at_unix_seconds.saturating_sub(source_retention_seconds);
    let stale_heartbeat_count = state.count_stale_passive_worker_heartbeats(heartbeat_cutoff)?;
    let source_health_prune_candidate_count =
        state.count_passive_source_health_samples_before(source_cutoff, source_kind, region_id)?;

    let mut suggested_actions = Vec::new();
    if stale_heartbeat_count > 0 {
        suggested_actions.push(PassiveCommandCenterAction {
            action_id: "prune-stale-heartbeats".to_string(),
            title: "Prune stale worker heartbeats".to_string(),
            reason: format!("{stale_heartbeat_count} heartbeats are older than retention."),
            method: "POST",
            path: "/v1/passive/worker/heartbeats/prune".to_string(),
            payload: None,
            confirmation_read_paths: vec![
                "/v1/passive/worker/diagnostics?stale_after_seconds=86400".to_string(),
                "/v1/passive/worker/heartbeats?stale_only=true".to_string(),
            ],
        });
    }
    if source_health_prune_candidate_count > 0 {
        suggested_actions.push(PassiveCommandCenterAction {
            action_id: "prune-source-health-samples".to_string(),
            title: "Prune old source health samples".to_string(),
            reason: format!(
                "{source_health_prune_candidate_count} source health samples are older than retention."
            ),
            method: "POST",
            path: "/v1/passive/source-health/samples/prune".to_string(),
            payload: None,
            confirmation_read_paths: vec![
                format!(
                    "/v1/passive/source-health/samples?limit=20{}{}",
                    source_kind
                        .map(|kind| format!("&source_kind={kind:?}"))
                        .unwrap_or_default(),
                    region_id
                        .map(|region_id| format!("&region_id={region_id}"))
                        .unwrap_or_default()
                ),
                "/v1/passive/worker/diagnostics".to_string(),
            ],
        });
    }

    suggested_actions.extend(build_source_readiness_actions(state, region_id)?);

    Ok(PassiveCommandCenterMaintenance {
        heartbeat_retention_seconds,
        source_retention_seconds,
        stale_heartbeat_count,
        source_health_prune_candidate_count,
        source_kind,
        region_id: region_id.map(ToOwned::to_owned),
        suggested_actions,
    })
}

#[allow(clippy::too_many_lines)]
fn build_source_readiness_actions(
    state: &AppState,
    region_id: Option<&str>,
) -> Result<Vec<PassiveCommandCenterAction>, AppError> {
    let scheduler_enabled = env::var("SSS_PASSIVE_REGION_POLL_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0)
        > 0;
    let total_default_regions = default_passive_region_requests().len();
    let configured_region_count = state.passive_region_targets(1_000, false)?.len();
    let missing_region_count = total_default_regions.saturating_sub(configured_region_count);
    let readiness = state.passive_source_readiness(region_id)?;
    let ready_count = readiness
        .iter()
        .filter(|source| source.readiness == PassiveSourceReadinessLevel::Ready)
        .count();
    let total_samples = readiness
        .iter()
        .map(|source| source.sample_count)
        .sum::<usize>();
    let blocking_sources = readiness
        .iter()
        .filter(|source| source.readiness == PassiveSourceReadinessLevel::NeedsConfig)
        .map(|source| source.label.clone())
        .collect::<Vec<_>>();
    let lagging_sources = readiness
        .iter()
        .filter(|source| {
            matches!(
                source.readiness,
                PassiveSourceReadinessLevel::AwaitingData | PassiveSourceReadinessLevel::Degraded
            )
        })
        .map(|source| source.label.clone())
        .collect::<Vec<_>>();
    let mut actions = Vec::new();

    if missing_region_count > 0 {
        actions.push(PassiveCommandCenterAction {
            action_id: "bootstrap-default-passive-regions".to_string(),
            title: "Bootstrap strategic regional coverage".to_string(),
            reason: format!(
                "This instance only has {configured_region_count} of {total_default_regions} default watch regions. Backfill the missing {missing_region_count} regions to expand Europe, MENA, Russia, and Iran coverage while keeping Portugal as the primary focus."
            ),
            method: "POST",
            path: "/v1/passive/regions/bootstrap-defaults".to_string(),
            payload: Some(json!({
                "overwrite_existing": false
            })),
            confirmation_read_paths: vec![
                "/v1/passive/regions?limit=1000".to_string(),
                "/v1/passive/dashboard/summary".to_string(),
                "/v1/passive/command-center/summary".to_string(),
            ],
        });
    }

    if !blocking_sources.is_empty() {
        actions.push(PassiveCommandCenterAction {
            action_id: "inspect-source-readiness".to_string(),
            title: "Inspect source readiness".to_string(),
            reason: format!(
                "{} still need configuration before the passive surface can wake up.",
                blocking_sources.join(", ")
            ),
            method: "GET",
            path: format!(
                "/v1/passive/source-health/readiness{}",
                region_id
                    .map(|current| format!("?region_id={current}"))
                    .unwrap_or_default()
            ),
            payload: None,
            confirmation_read_paths: vec![
                "/v1/passive/source-health/readiness".to_string(),
                "/v1/passive/worker/diagnostics".to_string(),
            ],
        });
    }

    if total_samples == 0 {
        actions.push(PassiveCommandCenterAction {
            action_id: "prime-passive-region-cycle".to_string(),
            title: if scheduler_enabled {
                "Prime passive region cycle".to_string()
            } else {
                "Run first passive cycle".to_string()
            },
            reason: if ready_count == 0 {
                if scheduler_enabled {
                    "No source samples have landed yet. Force a discovery and scan cycle to seed the console.".to_string()
                } else {
                    "The passive scheduler is off and no source samples have landed yet. Run a manual discovery and scan cycle to seed the console.".to_string()
                }
            } else if scheduler_enabled {
                "Ready feeds exist but the console has not received source samples yet. Force a discovery and scan cycle now.".to_string()
            } else {
                "Ready feeds exist, but the passive scheduler is off. Run one manual cycle to seed the console while the deploy stays in manual mode.".to_string()
            },
            method: "POST",
            path: "/v1/passive/regions/run".to_string(),
            payload: Some(passive_regions_run_payload(region_id)),
            confirmation_read_paths: vec![
                "/v1/passive/dashboard/summary".to_string(),
                "/v1/passive/source-health/readiness".to_string(),
                "/v1/passive/map/regions".to_string(),
            ],
        });
    } else if !lagging_sources.is_empty() {
        actions.push(PassiveCommandCenterAction {
            action_id: "run-passive-scheduler-now".to_string(),
            title: if scheduler_enabled {
                "Run passive scheduler now".to_string()
            } else {
                "Run manual passive refresh".to_string()
            },
            reason: if scheduler_enabled {
                format!(
                    "{} are configured but still waiting for fresh samples. Trigger the scheduler for a fresh pass.",
                    lagging_sources.join(", ")
                )
            } else {
                format!(
                    "{} are configured but the passive scheduler is off. Run a manual refresh pass now.",
                    lagging_sources.join(", ")
                )
            },
            method: "POST",
            path: "/v1/passive/scheduler/run".to_string(),
            payload: Some(json!({
                "limit": 25,
                "window_hours": 24,
                "include_adsb": true,
                "include_weather": true,
                "include_fire_smoke": true,
                "force": true,
                "dry_run": false
            })),
            confirmation_read_paths: vec![
                "/v1/passive/dashboard/summary".to_string(),
                "/v1/passive/source-health/samples?limit=20".to_string(),
                "/v1/passive/worker/diagnostics".to_string(),
            ],
        });
    }

    Ok(actions)
}

fn passive_regions_run_payload(region_id: Option<&str>) -> serde_json::Value {
    if let Some(current_region_id) = region_id {
        json!({
            "region_ids": [current_region_id],
            "force_discovery": true,
            "dry_run": false,
            "window_hours": 24,
            "include_adsb": true,
            "include_weather": true,
            "include_fire_smoke": true
        })
    } else {
        json!({
            "force_discovery": true,
            "dry_run": false,
            "window_hours": 24,
            "include_adsb": true,
            "include_weather": true,
            "include_fire_smoke": true
        })
    }
}

fn command_center_operator_paths() -> Vec<String> {
    vec![
        "/v1/passive/dashboard/summary?limit=5&semantic_window_hours=336".to_string(),
        "/v1/passive/operational-visibility?limit=25".to_string(),
        "/v1/passive/maintenance/summary".to_string(),
        "/v1/passive/map/regions".to_string(),
        "/v1/passive/map/canonical-events?limit=50".to_string(),
        "/v1/passive/worker/diagnostics".to_string(),
    ]
}

fn focus_region_paths(region_id: Option<&str>) -> Vec<String> {
    let Some(region_id) = region_id else {
        return Vec::new();
    };
    vec![
        format!("/v1/passive/operational-visibility?region_id={region_id}&limit=25"),
        format!("/v1/passive/map/sites?region_id={region_id}"),
        format!("/v1/passive/map/canonical-events?region_id={region_id}&limit=50"),
        format!("/v1/passive/regions/{region_id}/overview"),
        format!("/v1/passive/regions/{region_id}/semantic-timeline?limit=12&window_hours=336"),
        format!(
            "/v1/passive/regions/{region_id}/operational-timeline?window_hours=72&bucket_hours=6&include_empty_buckets=false"
        ),
    ]
}

fn build_command_center_summary(
    dashboard: &PassiveDashboardSummary,
    maintenance: &PassiveCommandCenterMaintenance,
    focus_region_id: Option<&str>,
) -> String {
    let top_region = preferred_focus_region(&dashboard.top_regions).map_or_else(
        || "no priority region".to_string(),
        |region| region.name.clone(),
    );
    let semantic_count = dashboard.semantic_timeline.len();
    let prune_suffix = if maintenance.stale_heartbeat_count > 0
        || maintenance.source_health_prune_candidate_count > 0
    {
        format!(
            " Maintenance queue: {} stale heartbeats, {} source prune candidates.",
            maintenance.stale_heartbeat_count, maintenance.source_health_prune_candidate_count
        )
    } else {
        " Maintenance queue is clear.".to_string()
    };
    match focus_region_id {
        Some(region_id) => format!(
            "Command center focused on {region_id}. Top region in view: {top_region}. {semantic_count} semantic entries are active.{prune_suffix}"
        ),
        None => format!(
            "Command center is in global mode. Top region in view: {top_region}. {semantic_count} semantic entries are active.{prune_suffix}"
        ),
    }
}

fn build_command_center_highlights(
    dashboard: &PassiveDashboardSummary,
) -> PassiveCommandCenterHighlights {
    let top_region = preferred_focus_region(&dashboard.top_regions).map(|region| {
        PassiveCommandCenterRegionHighlight {
            region_id: region.region_id.clone(),
            name: region.name.clone(),
            operational_pressure_priority: region.operational_pressure_priority,
            dominant_status: region.dominant_status,
            risk_delta_classification: region.risk_delta_classification,
            risk_delta_explanation: region.risk_delta_explanation.clone(),
            narrative_summary: region.narrative_summary.clone(),
        }
    });
    let top_site = dashboard
        .top_sites
        .iter()
        .find(|site| site.site_id.is_some())
        .map(|site| PassiveCommandCenterSiteHighlight {
            site_id: site.site_id.clone().unwrap_or_default(),
            name: site.name.clone(),
            site_type: site.site_type,
            risk_score: site.risk_score,
            risk_trend: site.risk_trend,
            top_canonical_status: site.top_canonical_status,
            recommendation_review_state: site.recommendation_review_state,
            has_recommendation: site.has_recommendation,
        });
    let top_event =
        dashboard
            .top_canonical_events
            .first()
            .map(|event| PassiveCommandCenterEventHighlight {
                canonical_event_id: event.canonical_event_id.clone(),
                site_id: event.site_id.clone(),
                site_name: event.site_name.clone(),
                event_type: event.event_type.clone(),
                status: event.status,
                temporal_phase: event.temporal_phase,
                operational_readout: event.operational_readout.clone(),
            });
    PassiveCommandCenterHighlights {
        top_region,
        top_site,
        top_event,
    }
}

fn build_attention_queue(
    state: &AppState,
    dashboard: &PassiveDashboardSummary,
    maintenance: &PassiveCommandCenterMaintenance,
    limit: usize,
    attention_kind: Option<PassiveCommandCenterAttentionKind>,
    min_attention_priority: Option<MapPriority>,
) -> Vec<PassiveCommandCenterAttentionItem> {
    let mut items = Vec::new();
    let now_unix_seconds = now_unix_seconds();

    for action in &maintenance.suggested_actions {
        items.push(PassiveCommandCenterAttentionItem {
            item_id: format!("maintenance:{}", action.action_id),
            kind: PassiveCommandCenterAttentionKind::Maintenance,
            priority: maintenance_action_priority(action),
            urgency_label: "maintenance backlog".to_string(),
            age_seconds: None,
            age_bucket: None,
            title: action.title.clone(),
            reason: action.reason.clone(),
            primary_action_label: maintenance_action_label(action),
            region_id: maintenance.region_id.clone(),
            site_id: None,
            canonical_event_id: None,
            action_path: Some(action.path.clone()),
            operator_state: None,
            confirmation_read_paths: action.confirmation_read_paths.clone(),
        });
    }

    items.extend(latest_neows_attention_items(state, limit).unwrap_or_default());

    items.extend(
        dashboard
            .top_canonical_events
            .iter()
            .take(limit)
            .map(|event| PassiveCommandCenterAttentionItem {
                item_id: format!("canonical:{}", event.canonical_event_id),
                kind: PassiveCommandCenterAttentionKind::CanonicalEvent,
                priority: event.priority,
                urgency_label: canonical_attention_urgency(
                    event.last_observed_at_unix_seconds,
                    now_unix_seconds,
                ),
                age_seconds: Some(
                    now_unix_seconds.saturating_sub(event.last_observed_at_unix_seconds),
                ),
                age_bucket: None,
                title: format!("{} @ {}", event.event_type, event.site_name),
                reason: event.operational_readout.clone(),
                primary_action_label: "Focus Event".to_string(),
                region_id: event.region_id.clone(),
                site_id: Some(event.site_id.clone()),
                canonical_event_id: Some(event.canonical_event_id.clone()),
                action_path: None,
                operator_state: None,
                confirmation_read_paths: event
                    .bundle_hashes
                    .iter()
                    .take(2)
                    .map(|hash| format!("/v1/evidence/{hash}"))
                    .chain(
                        event
                            .manifest_hashes
                            .iter()
                            .take(2)
                            .map(|hash| format!("/v1/replay/{hash}")),
                    )
                    .collect(),
            }),
    );

    items.extend(
        dashboard
            .top_sites
            .iter()
            .filter(|site| site.site_id.is_some())
            .take(limit)
            .map(|site| site_attention_item(site, now_unix_seconds)),
    );

    items.extend(
        dashboard
            .top_regions
            .iter()
            .take(limit)
            .map(|region| region_attention_item(region, now_unix_seconds)),
    );

    if let Some(kind) = attention_kind {
        items.retain(|item| item.kind == kind);
    }
    if let Some(min_priority) = min_attention_priority {
        items.retain(|item| item.priority >= min_priority);
    }

    items.sort_by(|left, right| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| attention_urgency_rank(left).cmp(&attention_urgency_rank(right)))
            .then_with(|| {
                left.age_seconds
                    .unwrap_or(i64::MAX)
                    .cmp(&right.age_seconds.unwrap_or(i64::MAX))
            })
            .then_with(|| attention_kind_rank(left.kind).cmp(&attention_kind_rank(right.kind)))
            .then_with(|| left.title.cmp(&right.title))
    });
    items.truncate(limit.max(1));
    items
}

fn latest_neows_attention_items(
    state: &AppState,
    limit: usize,
) -> Result<Vec<PassiveCommandCenterAttentionItem>, AppError> {
    let Some(feed) = state.cached_neows_feed()? else {
        return Ok(Vec::new());
    };
    Ok(feed
        .objects
        .iter()
        .take(limit.min(3))
        .map(|object| neows_attention_item(&feed, object))
        .collect())
}

fn neows_attention_item(
    feed: &NeoRiskFeed,
    object: &NeoRiskObject,
) -> PassiveCommandCenterAttentionItem {
    let feed_path = format!(
        "/v1/briefing/neows/feed?start_date={}&end_date={}",
        feed.start_date, feed.end_date
    );
    let briefing_path = format!(
        "/v1/briefing/neows?start_date={}&end_date={}",
        feed.start_date, feed.end_date
    );
    let mut confirmation_read_paths = vec![briefing_path.clone(), feed_path.clone()];
    if let Some(date) = object.close_approach_date.as_ref() {
        confirmation_read_paths.push(format!(
            "/v1/briefing/neows?start_date={date}&end_date={date}"
        ));
    }
    PassiveCommandCenterAttentionItem {
        item_id: format!("neo:{}", object.neo_reference_id),
        kind: PassiveCommandCenterAttentionKind::Neo,
        priority: priority_from_risk(object.priority_score),
        urgency_label: "orbital context".to_string(),
        age_seconds: None,
        age_bucket: None,
        title: object.name.clone(),
        reason: format!(
            "{} Miss distance: {}. Relative velocity: {}.",
            object.briefing_summary,
            object
                .miss_distance_km
                .map_or_else(|| "n/a".to_string(), |distance| format!("{distance:.0} km")),
            object.relative_velocity_km_s.map_or_else(
                || "n/a".to_string(),
                |velocity| format!("{velocity:.1} km/s")
            )
        ),
        primary_action_label: "Open NEO Feed".to_string(),
        region_id: None,
        site_id: None,
        canonical_event_id: None,
        action_path: Some(feed_path),
        operator_state: None,
        confirmation_read_paths,
    }
}

fn site_attention_item(
    site: &SiteMapItem,
    now_unix_seconds: i64,
) -> PassiveCommandCenterAttentionItem {
    let site_id = site.site_id.clone().unwrap_or_default();
    let priority = std::cmp::max(site.scan_priority, priority_from_risk(site.risk_score));
    let operator_state = site_operator_state_label(site);
    let reason = format!("{} {}", site.status_reason, site.priority_reason);
    let age_seconds = site.recommendation_age_seconds.or_else(|| {
        site.last_event_at_unix_seconds
            .or(site.last_scanned_at_unix_seconds)
            .map(|ts| now_unix_seconds.saturating_sub(ts))
    });
    PassiveCommandCenterAttentionItem {
        item_id: format!("site:{site_id}"),
        kind: PassiveCommandCenterAttentionKind::Site,
        priority,
        urgency_label: site_attention_urgency(site, age_seconds),
        age_seconds,
        age_bucket: site.recommendation_age_bucket,
        title: site.name.clone(),
        reason,
        primary_action_label: if site.has_recommendation
            && site.recommendation_review_state.is_none()
        {
            "Review Recommendation".to_string()
        } else if matches!(
            site.recommendation_review_state,
            Some(sss_storage::PassiveRecommendationReviewState::Applied)
        ) {
            "Inspect Applied".to_string()
        } else if site.observed {
            "Open Site".to_string()
        } else {
            "Prime Site".to_string()
        },
        region_id: Some(site.region_id.clone()),
        site_id: Some(site_id),
        canonical_event_id: None,
        action_path: site.overview_path.clone(),
        operator_state,
        confirmation_read_paths: site
            .overview_path
            .iter()
            .cloned()
            .chain(site.narrative_path.iter().cloned())
            .chain(
                site.site_id
                    .iter()
                    .map(|site_id| format!("/v1/orchestrator/sites/{site_id}/decisions?limit=5")),
            )
            .chain(
                site.site_id
                    .iter()
                    .map(|site_id| format!("/v1/orchestrator/sites/{site_id}/reviews?limit=5")),
            )
            .collect(),
    }
}

fn region_attention_item(
    region: &RegionMapItem,
    now_unix_seconds: i64,
) -> PassiveCommandCenterAttentionItem {
    let age_seconds = region.pending_review_oldest_age_seconds.or_else(|| {
        region
            .latest_run_finished_at_unix_seconds
            .map(|ts| now_unix_seconds.saturating_sub(ts))
    });
    PassiveCommandCenterAttentionItem {
        item_id: format!("region:{}", region.region_id),
        kind: PassiveCommandCenterAttentionKind::Region,
        priority: region.operational_pressure_priority,
        urgency_label: region_attention_urgency(region, age_seconds),
        age_seconds,
        age_bucket: region.pending_review_age_bucket,
        title: region.name.clone(),
        reason: region.operational_summary.clone(),
        primary_action_label: "Open Region".to_string(),
        region_id: Some(region.region_id.clone()),
        site_id: None,
        canonical_event_id: None,
        action_path: Some(format!("/v1/passive/regions/{}/overview", region.region_id)),
        operator_state: region_operator_state_label(region),
        confirmation_read_paths: vec![
            format!("/v1/passive/map/sites?region_id={}", region.region_id),
            format!(
                "/v1/passive/map/canonical-events?region_id={}&limit=50",
                region.region_id
            ),
        ],
    }
}

fn site_operator_state_label(site: &SiteMapItem) -> Option<String> {
    match site.recommendation_review_state {
        Some(sss_storage::PassiveRecommendationReviewState::Reviewed) => {
            Some("reviewed".to_string())
        }
        Some(sss_storage::PassiveRecommendationReviewState::Applied) => Some("applied".to_string()),
        Some(sss_storage::PassiveRecommendationReviewState::Dismissed) => {
            Some("dismissed".to_string())
        }
        None if site.has_recommendation => Some("pending_review".to_string()),
        None => None,
    }
}

fn region_operator_state_label(region: &RegionMapItem) -> Option<String> {
    if region.recommendation_pending_review_count > 0 {
        Some("pending_review".to_string())
    } else if region.recommendation_applied_count > 0 {
        Some("applied".to_string())
    } else if region.recommendation_reviewed_count > 0 {
        Some("reviewed".to_string())
    } else if region.recommendation_dismissed_count > 0 {
        Some("dismissed".to_string())
    } else {
        None
    }
}

fn maintenance_action_priority(action: &PassiveCommandCenterAction) -> MapPriority {
    match action.action_id.as_str() {
        "prune-stale-heartbeats" => MapPriority::High,
        "prune-source-health-samples" => MapPriority::Medium,
        _ => MapPriority::Low,
    }
}

fn canonical_attention_urgency(
    last_observed_at_unix_seconds: i64,
    now_unix_seconds: i64,
) -> String {
    let age_seconds = now_unix_seconds.saturating_sub(last_observed_at_unix_seconds);
    if age_seconds <= 900 {
        "new signal".to_string()
    } else if age_seconds <= 3_600 {
        "active now".to_string()
    } else {
        "watching".to_string()
    }
}

fn site_attention_urgency(site: &SiteMapItem, age_seconds: Option<i64>) -> String {
    if site.has_recommendation && site.recommendation_review_state.is_none() {
        return match age_seconds {
            Some(seconds) if seconds > 21_600 => "stale pending review".to_string(),
            Some(seconds) if seconds > 3_600 => "pending review".to_string(),
            _ => "new recommendation".to_string(),
        };
    }
    if matches!(
        site.recommendation_review_state,
        Some(sss_storage::PassiveRecommendationReviewState::Applied)
    ) {
        return "applied action".to_string();
    }
    if site.elevated {
        return "elevated site".to_string();
    }
    "monitoring".to_string()
}

fn region_attention_urgency(region: &RegionMapItem, age_seconds: Option<i64>) -> String {
    if region.recommendation_pending_review_count > 0 {
        return if age_seconds.is_some_and(|seconds| seconds > 21_600) {
            "stale regional backlog".to_string()
        } else {
            "needs review now".to_string()
        };
    }
    if region.recent_failed_run_count > 0 || region.stale_worker_count > 0 {
        return "degraded runtime".to_string();
    }
    if region.seeds_elevated > 0 || region.recent_events > 0 {
        return "active pressure".to_string();
    }
    "stable watch".to_string()
}

fn attention_urgency_rank(item: &PassiveCommandCenterAttentionItem) -> u8 {
    match item.urgency_label.as_str() {
        "needs review now" => 0,
        "new recommendation" => 1,
        "new signal" => 2,
        "active now" => 3,
        "stale pending review" => 4,
        "pending review" => 5,
        "degraded runtime" => 6,
        "active pressure" => 7,
        "applied action" => 8,
        "watching" => 9,
        "stable watch" => 10,
        "monitoring" => 11,
        "maintenance backlog" => 12,
        "orbital context" => 13,
        _ => 20,
    }
}

fn maintenance_action_label(action: &PassiveCommandCenterAction) -> String {
    match action.action_id.as_str() {
        "prune-stale-heartbeats" => "Prune Heartbeats".to_string(),
        "prune-source-health-samples" => "Preview Prune".to_string(),
        _ => "Review Action".to_string(),
    }
}

fn priority_from_risk(risk_score: f64) -> MapPriority {
    if risk_score >= 0.85 {
        MapPriority::Critical
    } else if risk_score >= 0.65 {
        MapPriority::High
    } else if risk_score >= 0.35 {
        MapPriority::Medium
    } else {
        MapPriority::Low
    }
}

fn attention_kind_rank(kind: PassiveCommandCenterAttentionKind) -> u8 {
    match kind {
        PassiveCommandCenterAttentionKind::Maintenance => 0,
        PassiveCommandCenterAttentionKind::Neo => 1,
        PassiveCommandCenterAttentionKind::CanonicalEvent => 2,
        PassiveCommandCenterAttentionKind::Site => 3,
        PassiveCommandCenterAttentionKind::Region => 4,
    }
}
