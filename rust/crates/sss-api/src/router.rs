use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::{
    analyze_object, bootstrap_default_passive_regions, discover_and_scan_passive_sites,
    discover_passive_sites, dispatch_event_notifications, dispatch_notifications, execute_replay,
    get_apod_briefing, get_assessment_history, get_events, get_events_timeline,
    get_evidence_bundle, get_ingest_batches, get_ingest_status, get_neows_briefing, get_neows_feed,
    get_notification_deliveries, get_object_close_approaches, get_object_timeline,
    get_passive_canonical_event, get_passive_canonical_events, get_passive_command_center_summary,
    get_passive_dashboard_summary, get_passive_maintenance_summary, get_passive_map_events,
    get_passive_map_regions, get_passive_map_sites, get_passive_observations,
    get_passive_operational_visibility, get_passive_operations_status, get_passive_region_leases,
    get_passive_region_operational_timeline, get_passive_region_overview,
    get_passive_region_remediation, get_passive_region_run, get_passive_region_runs,
    get_passive_region_semantic_timeline, get_passive_regions, get_passive_seed, get_passive_seeds,
    get_passive_site_events, get_passive_site_load_forecast, get_passive_site_narrative,
    get_passive_site_orchestrator_decisions, get_passive_site_overview, get_passive_site_patterns,
    get_passive_site_recommendation_reviews, get_passive_site_risk_history,
    get_passive_site_scenario_forecast, get_passive_site_semantic_timeline,
    get_passive_site_solar_forecast, get_passive_sites, get_passive_source_health_samples,
    get_passive_source_readiness, get_passive_worker_diagnostics, get_passive_worker_heartbeats,
    get_prediction_snapshots, get_replay_manifest, get_replay_manifest_for_bundle, health,
    ingest_celestrak_active, ingest_tle, landing_page, operator_console, passive_scan,
    passive_scan_live, predict, prune_passive_source_health_samples,
    prune_passive_worker_heartbeats, recommend_passive_site_action,
    review_passive_site_recommendation, run_passive_regions, run_passive_scheduler, space_overview,
    upsert_passive_region, version,
};
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(landing_page))
        .route("/health", get(health))
        .route("/console", get(operator_console))
        .route("/v1/health", get(health))
        .route("/v1/version", get(version))
        .route("/v1/briefing/apod", get(get_apod_briefing))
        .route("/v1/briefing/neows", get(get_neows_briefing))
        .route("/v1/briefing/neows/feed", get(get_neows_feed))
        .route("/v1/ingest/tle", post(ingest_tle))
        .route("/v1/ingest/celestrak-active", post(ingest_celestrak_active))
        .route("/v1/ingest/batches", get(get_ingest_batches))
        .route("/v1/ingest/status", get(get_ingest_status))
        .route("/v1/analyze-object", post(analyze_object))
        .route("/v1/space-overview", post(space_overview))
        .route("/v1/predict", post(predict))
        .merge(passive_routes())
        .route("/v1/notifications/dispatch", post(dispatch_notifications))
        .route("/v1/events/dispatch", post(dispatch_event_notifications))
        .route("/v1/notifications", get(get_notification_deliveries))
        .route("/v1/events/timeline", get(get_events_timeline))
        .route("/v1/events", get(get_events))
        .route(
            "/v1/objects/:object_id/assessments",
            get(get_assessment_history),
        )
        .route(
            "/v1/objects/:object_id/predictions",
            get(get_prediction_snapshots),
        )
        .route(
            "/v1/objects/:object_id/close-approaches",
            get(get_object_close_approaches),
        )
        .route("/v1/objects/:object_id/timeline", get(get_object_timeline))
        .route(
            "/v1/evidence/:bundle_hash/replay",
            get(get_replay_manifest_for_bundle),
        )
        .route("/v1/evidence/:bundle_hash", get(get_evidence_bundle))
        .route("/v1/replay/:manifest_hash/execute", get(execute_replay))
        .route("/v1/replay/:manifest_hash", get(get_replay_manifest))
        .with_state(state)
}

#[allow(clippy::too_many_lines)]
fn passive_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/passive/discover-sites", post(discover_passive_sites))
        .route(
            "/v1/passive/discover-and-scan",
            post(discover_and_scan_passive_sites),
        )
        .route("/v1/passive/scan", post(passive_scan))
        .route("/v1/passive/scan/live", post(passive_scan_live))
        .route("/v1/passive/map/regions", get(get_passive_map_regions))
        .route("/v1/passive/map/sites", get(get_passive_map_sites))
        .route("/v1/passive/map/events", get(get_passive_map_events))
        .route(
            "/v1/passive/map/canonical-events",
            get(get_passive_canonical_events),
        )
        .route(
            "/v1/passive/operations/status",
            get(get_passive_operations_status),
        )
        .route(
            "/v1/passive/maintenance/summary",
            get(get_passive_maintenance_summary),
        )
        .route(
            "/v1/passive/dashboard/summary",
            get(get_passive_dashboard_summary),
        )
        .route(
            "/v1/passive/operational-visibility",
            get(get_passive_operational_visibility),
        )
        .route(
            "/v1/passive/command-center/summary",
            get(get_passive_command_center_summary),
        )
        .route(
            "/v1/passive/canonical-events",
            get(get_passive_canonical_events),
        )
        .route(
            "/v1/passive/canonical-events/:canonical_event_id",
            get(get_passive_canonical_event),
        )
        .route("/v1/passive/scheduler/run", post(run_passive_scheduler))
        .route(
            "/v1/passive/regions",
            get(get_passive_regions).post(upsert_passive_region),
        )
        .route(
            "/v1/passive/regions/bootstrap-defaults",
            post(bootstrap_default_passive_regions),
        )
        .route("/v1/passive/regions/run", post(run_passive_regions))
        .route("/v1/passive/regions/runs", get(get_passive_region_runs))
        .route("/v1/passive/regions/leases", get(get_passive_region_leases))
        .route(
            "/v1/passive/worker/heartbeats",
            get(get_passive_worker_heartbeats),
        )
        .route(
            "/v1/passive/worker/heartbeats/prune",
            post(prune_passive_worker_heartbeats),
        )
        .route(
            "/v1/passive/worker/diagnostics",
            get(get_passive_worker_diagnostics),
        )
        .route(
            "/v1/passive/source-health/samples",
            get(get_passive_source_health_samples),
        )
        .route(
            "/v1/passive/source-health/readiness",
            get(get_passive_source_readiness),
        )
        .route(
            "/v1/passive/source-health/samples/prune",
            post(prune_passive_source_health_samples),
        )
        .route(
            "/v1/passive/regions/runs/:run_id",
            get(get_passive_region_run),
        )
        .route(
            "/v1/passive/regions/:region_id/overview",
            get(get_passive_region_overview),
        )
        .route(
            "/v1/passive/regions/:region_id/semantic-timeline",
            get(get_passive_region_semantic_timeline),
        )
        .route(
            "/v1/passive/regions/:region_id/operational-timeline",
            get(get_passive_region_operational_timeline),
        )
        .route(
            "/v1/passive/regions/:region_id/remediation",
            get(get_passive_region_remediation),
        )
        .route("/v1/passive/seeds", get(get_passive_seeds))
        .route("/v1/passive/seeds/:seed_key", get(get_passive_seed))
        .route("/v1/passive/sites", get(get_passive_sites))
        .route("/v1/passive/observations", get(get_passive_observations))
        .route(
            "/v1/passive/sites/:site_id/overview",
            get(get_passive_site_overview),
        )
        .route(
            "/v1/passive/sites/:site_id/events",
            get(get_passive_site_events),
        )
        .route(
            "/v1/passive/sites/:site_id/risk-history",
            get(get_passive_site_risk_history),
        )
        .route(
            "/v1/passive/sites/:site_id/patterns",
            get(get_passive_site_patterns),
        )
        .route(
            "/v1/passive/sites/:site_id/narrative",
            get(get_passive_site_narrative),
        )
        .route(
            "/v1/forecast/sites/:site_id/solar",
            get(get_passive_site_solar_forecast),
        )
        .route(
            "/v1/forecast/sites/:site_id/load",
            get(get_passive_site_load_forecast),
        )
        .route(
            "/v1/forecast/sites/:site_id/scenario",
            get(get_passive_site_scenario_forecast),
        )
        .route(
            "/v1/orchestrator/sites/:site_id/recommend",
            post(recommend_passive_site_action),
        )
        .route(
            "/v1/orchestrator/sites/:site_id/decisions",
            get(get_passive_site_orchestrator_decisions),
        )
        .route(
            "/v1/orchestrator/sites/:site_id/reviews",
            get(get_passive_site_recommendation_reviews).post(review_passive_site_recommendation),
        )
        .route(
            "/v1/passive/sites/:site_id/semantic-timeline",
            get(get_passive_site_semantic_timeline),
        )
}
