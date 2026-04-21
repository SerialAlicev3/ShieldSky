use serde::{Deserialize, Serialize};

use crate::map_views::{build_region_map_response, MapPriority};
use crate::state::{now_unix_seconds, AppError, AppState, PassiveRegionLeaseDiagnostic};

#[derive(Debug, Clone, Deserialize)]
pub struct OperationalVisibilityQuery {
    pub region_id: Option<String>,
    pub limit: Option<usize>,
    pub stale_after_seconds: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationalVisibilityState {
    Healthy,
    Pressured,
    Degraded,
    Failing,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OperationalVisibilityDrivers {
    pub due: bool,
    pub run_count: usize,
    pub recent_failed_run_count: usize,
    pub recent_partial_run_count: usize,
    pub active_lease_count: usize,
    pub stale_lease_count: usize,
    pub active_worker_count: usize,
    pub stale_worker_count: usize,
    pub source_degraded_count: usize,
    pub source_stale_count: usize,
    pub latest_run_finished_at_unix_seconds: Option<i64>,
    pub cadence_gap_seconds: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OperationalVisibilityRegion {
    pub region_id: String,
    pub name: String,
    pub pressure_priority: MapPriority,
    pub state: OperationalVisibilityState,
    pub latest_run_status: Option<String>,
    pub drivers: OperationalVisibilityDrivers,
    pub narrative: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OperationalVisibilitySummary {
    pub generated_at_unix_seconds: i64,
    pub region_id: Option<String>,
    pub overall_state: OperationalVisibilityState,
    pub total_regions: usize,
    pub active_workers: usize,
    pub stale_workers: usize,
    pub active_leases: usize,
    pub stale_leases: usize,
    pub workers_visible: usize,
    pub stale_workers_visible: usize,
    pub active_leases_visible: usize,
    pub stale_leases_visible: usize,
    pub regions: Vec<OperationalVisibilityRegion>,
    pub worker_heartbeats: Vec<sss_storage::PassiveWorkerHeartbeat>,
    pub active_region_leases: Vec<PassiveRegionLeaseDiagnostic>,
    pub stale_region_leases: Vec<PassiveRegionLeaseDiagnostic>,
    pub panel_paths: Vec<String>,
    pub map_overlay_paths: Vec<String>,
    pub narrative: String,
}

pub fn build_operational_visibility_summary(
    state: &AppState,
    query: &OperationalVisibilityQuery,
) -> Result<OperationalVisibilitySummary, AppError> {
    let now = now_unix_seconds();
    let limit = query.limit.unwrap_or(25).clamp(1, 200);
    let stale_after_seconds = query.stale_after_seconds.unwrap_or(180).max(1);

    let operations = state.passive_operations_status()?;
    let diagnostics =
        state.passive_worker_diagnostics(limit, stale_after_seconds, query.region_id.as_deref())?;
    let regions = scoped_regions(state, query.region_id.as_deref())?;
    let visible_regions = regions
        .into_iter()
        .map(|region| build_region_visibility(region, &diagnostics.source_metrics, now))
        .collect::<Vec<_>>();

    let overall_state = visible_regions
        .iter()
        .map(|region| region.state)
        .max_by_key(|state| state_rank(*state))
        .unwrap_or(OperationalVisibilityState::Healthy);

    let worker_heartbeats = visible_worker_heartbeats(
        &operations.worker_heartbeats,
        query.region_id.as_deref(),
        limit,
    );

    let active_workers = diagnostics.active_worker_heartbeat_count;
    let stale_workers = diagnostics.stale_worker_heartbeat_count;
    let active_leases = diagnostics.active_region_lease_count;
    let stale_leases = diagnostics.stale_region_lease_count;

    let narrative = format!(
        "Operational visibility is {:?}: {} regions in scope, {} active workers, {} stale workers, {} active leases, {} stale leases.",
        overall_state,
        visible_regions.len(),
        active_workers,
        stale_workers,
        active_leases,
        stale_leases
    );

    Ok(OperationalVisibilitySummary {
        generated_at_unix_seconds: now,
        region_id: query.region_id.clone(),
        overall_state,
        total_regions: visible_regions.len(),
        active_workers,
        stale_workers,
        active_leases,
        stale_leases,
        workers_visible: worker_heartbeats.len(),
        stale_workers_visible: diagnostics.stale_worker_heartbeats.len(),
        active_leases_visible: diagnostics.active_region_leases.len(),
        stale_leases_visible: diagnostics.stale_region_leases.len(),
        regions: visible_regions,
        worker_heartbeats,
        active_region_leases: diagnostics.active_region_leases,
        stale_region_leases: diagnostics.stale_region_leases,
        panel_paths: vec![
            "/v1/passive/dashboard/summary?limit=5&semantic_window_hours=336".to_string(),
            "/v1/passive/command-center/summary?limit=5".to_string(),
            "/v1/passive/worker/diagnostics".to_string(),
            "/v1/passive/regions/runs?limit=20".to_string(),
        ],
        map_overlay_paths: vec![
            "/v1/passive/map/regions".to_string(),
            "/v1/passive/map/sites?region_id={region_id}".to_string(),
            "/v1/passive/map/canonical-events?region_id={region_id}&limit=50".to_string(),
        ],
        narrative,
    })
}

fn scoped_regions(
    state: &AppState,
    region_id: Option<&str>,
) -> Result<Vec<crate::map_views::RegionMapItem>, AppError> {
    let mut regions = build_region_map_response(state)?.regions;
    if let Some(region_id) = region_id {
        regions.retain(|region| region.region_id == region_id);
        if regions.is_empty() {
            return Err(AppError::SiteNotFound(format!(
                "passive region not found: {region_id}"
            )));
        }
    }
    Ok(regions)
}

fn build_region_visibility(
    region: crate::map_views::RegionMapItem,
    source_metrics: &[crate::state::PassiveWorkerSourceMetrics],
    now: i64,
) -> OperationalVisibilityRegion {
    let source_degraded_count = source_metrics
        .iter()
        .filter(|source| {
            source.region_id.as_deref() == Some(region.region_id.as_str())
                && matches!(
                    source.health_status,
                    crate::state::PassiveSourceHealthStatus::Degraded
                )
        })
        .count();
    let source_stale_count = source_metrics
        .iter()
        .filter(|source| {
            source.region_id.as_deref() == Some(region.region_id.as_str())
                && matches!(
                    source.health_status,
                    crate::state::PassiveSourceHealthStatus::Stale
                )
        })
        .count();

    let cadence_gap_seconds = region
        .latest_run_finished_at_unix_seconds
        .map(|finished| now.saturating_sub(finished));

    let drivers = OperationalVisibilityDrivers {
        due: region.discovery_due,
        run_count: region.recent_events,
        recent_failed_run_count: region.recent_failed_run_count,
        recent_partial_run_count: region.recent_partial_run_count,
        active_lease_count: region.active_lease_count,
        stale_lease_count: region.stale_lease_count,
        active_worker_count: region.active_worker_count,
        stale_worker_count: region.stale_worker_count,
        source_degraded_count,
        source_stale_count,
        latest_run_finished_at_unix_seconds: region.latest_run_finished_at_unix_seconds,
        cadence_gap_seconds,
    };

    let state = region_state(&drivers, region.operational_pressure_priority);

    OperationalVisibilityRegion {
        region_id: region.region_id,
        name: region.name,
        pressure_priority: region.operational_pressure_priority,
        state,
        latest_run_status: region.latest_run_status,
        drivers,
        narrative: region.operational_summary,
    }
}

fn visible_worker_heartbeats(
    heartbeats: &[sss_storage::PassiveWorkerHeartbeat],
    region_id: Option<&str>,
    limit: usize,
) -> Vec<sss_storage::PassiveWorkerHeartbeat> {
    heartbeats
        .iter()
        .filter(|heartbeat| {
            region_id.is_none() || heartbeat.current_region_id.as_deref() == region_id
        })
        .take(limit)
        .cloned()
        .collect::<Vec<_>>()
}

fn region_state(
    drivers: &OperationalVisibilityDrivers,
    pressure_priority: MapPriority,
) -> OperationalVisibilityState {
    if drivers.stale_lease_count > 0
        || drivers.stale_worker_count > 0
        || drivers.recent_failed_run_count > 0
        || drivers.source_stale_count > 0
        || pressure_priority == MapPriority::Critical
    {
        OperationalVisibilityState::Failing
    } else if drivers.recent_partial_run_count > 0
        || drivers.source_degraded_count > 0
        || pressure_priority == MapPriority::High
    {
        OperationalVisibilityState::Degraded
    } else if drivers.due || pressure_priority == MapPriority::Medium {
        OperationalVisibilityState::Pressured
    } else {
        OperationalVisibilityState::Healthy
    }
}

fn state_rank(state: OperationalVisibilityState) -> u8 {
    match state {
        OperationalVisibilityState::Healthy => 0,
        OperationalVisibilityState::Pressured => 1,
        OperationalVisibilityState::Degraded => 2,
        OperationalVisibilityState::Failing => 3,
    }
}
