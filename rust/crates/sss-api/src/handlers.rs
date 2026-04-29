use std::sync::atomic::{AtomicU64, Ordering};

use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Html;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::state::{AppError, AppState};
use crate::types::{ApiEnvelope, ApiError, API_VERSION};

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct VersionResponse {
    pub api_version: &'static str,
    pub core_model_version: &'static str,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IngestTleRequest {
    pub source: String,
    pub payload: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventDispatchRequest {
    pub event_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IngestTleResponse {
    pub source: String,
    pub source_url: Option<String>,
    pub records_received: usize,
    pub observations_created: usize,
    pub object_ids: Vec<String>,
    pub payload_bytes: usize,
    pub skipped_duplicate: bool,
    pub freshness_seconds: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventsQuery {
    pub limit: Option<usize>,
    pub object_id: Option<String>,
    pub event_type: Option<sss_core::RankedEventType>,
    pub future_only: Option<bool>,
    pub after_unix_seconds: Option<i64>,
    pub before_unix_seconds: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventsTimelineQuery {
    pub horizon: Option<f64>,
    pub limit_per_bucket: Option<usize>,
    pub object_id: Option<String>,
    pub event_type: Option<sss_core::RankedEventType>,
    pub reference_unix_seconds: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IngestBatchesQuery {
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotificationDeliveriesQuery {
    pub limit: Option<usize>,
    pub object_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PredictionSnapshotsQuery {
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PassiveRecommendationReviewsQuery {
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PassiveObservationsQuery {
    pub limit: Option<usize>,
    pub source_kind: Option<sss_passive_scanner::PassiveSourceKind>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PassiveSiteCorpusQuery {
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PassiveSiteForecastQuery {
    pub horizon_hours: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PassiveSiteRecommendationReviewRequest {
    pub state: sss_storage::PassiveRecommendationReviewState,
    pub actor: Option<String>,
    pub rationale: Option<String>,
    pub snapshot_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PassiveRecommendationReviewView {
    pub review_id: String,
    pub site_id: String,
    pub snapshot_id: String,
    pub request_id: String,
    pub endpoint: String,
    pub state: sss_storage::PassiveRecommendationReviewState,
    pub actor: Option<String>,
    pub rationale: Option<String>,
    pub evidence_bundle_hash: Option<String>,
    pub decided_at_unix_seconds: i64,
    pub age_seconds: i64,
    pub age_bucket: crate::map_views::ReviewAgeBucket,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PassiveSeedsQuery {
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PassiveRegionsQuery {
    pub limit: Option<usize>,
    pub enabled_only: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PassiveRegionRunsQuery {
    pub limit: Option<usize>,
    pub region_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LimitQuery {
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PassiveWorkerHeartbeatsQuery {
    pub limit: Option<usize>,
    pub stale_only: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PassiveWorkerDiagnosticsQuery {
    pub limit: Option<usize>,
    pub stale_after_seconds: Option<i64>,
    pub region_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PassiveMaintenanceSummaryQuery {
    pub heartbeat_retention_seconds: Option<i64>,
    pub source_retention_seconds: Option<i64>,
    pub source_kind: Option<sss_passive_scanner::PassiveSourceKind>,
    pub region_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PassiveMaintenanceSummary {
    pub generated_at_unix_seconds: i64,
    pub heartbeat_retention_seconds: i64,
    pub source_retention_seconds: i64,
    pub stale_heartbeat_count: usize,
    pub source_health_prune_candidate_count: usize,
    pub source_kind: Option<sss_passive_scanner::PassiveSourceKind>,
    pub region_id: Option<String>,
    pub suggested_actions: Vec<PassiveMaintenanceAction>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PassiveMaintenanceAction {
    pub action_id: String,
    pub title: String,
    pub reason: String,
    pub method: &'static str,
    pub path: String,
    pub confirmation_read_paths: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PassiveRegionRemediationQuery {
    pub stale_after_seconds: Option<i64>,
    pub site_limit: Option<usize>,
    pub days: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PassiveWorkerHeartbeatPruneRequest {
    pub older_than_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PassiveWorkerHeartbeatPruneResponse {
    pub generated_at_unix_seconds: i64,
    pub stale_before_unix_seconds: i64,
    pub pruned_count: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PassiveSourceHealthSamplesQuery {
    pub limit: Option<usize>,
    pub source_kind: Option<sss_passive_scanner::PassiveSourceKind>,
    pub region_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PassiveSourceHealthPruneRequest {
    pub older_than_seconds: Option<i64>,
    pub source_kind: Option<sss_passive_scanner::PassiveSourceKind>,
    pub region_id: Option<String>,
    pub dry_run: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PassiveSourceHealthPruneResponse {
    pub generated_at_unix_seconds: i64,
    pub older_than_unix_seconds: i64,
    pub source_kind: Option<sss_passive_scanner::PassiveSourceKind>,
    pub region_id: Option<String>,
    pub dry_run: bool,
    pub pruned_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct PassiveSchedulerRuntimeReadiness {
    pub enabled: bool,
    pub poll_seconds: Option<u64>,
    pub retry_seconds: Option<u64>,
    pub window_hours: Option<u64>,
    pub include_weather: bool,
    pub include_fire_smoke: bool,
    pub include_adsb: bool,
    pub force_discovery: bool,
    pub startup_discovery_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PassiveSourceReadinessResponse {
    pub generated_at_unix_seconds: i64,
    pub scheduler: PassiveSchedulerRuntimeReadiness,
    pub sources: Vec<crate::state::PassiveSourceReadiness>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PassiveRegionOverviewQuery {
    pub site_limit: Option<usize>,
    pub days: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PassiveSiteNarrativeQuery {
    pub days: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApodBriefingQuery {
    pub date: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NeoWsBriefingQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IngestStatusQuery {
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TimelineQuery {
    pub horizon: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CloseApproachesQuery {
    pub horizon: Option<f64>,
    pub threshold_km: Option<f64>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IngestStatusResponse {
    pub source: String,
    pub latest_request_id: String,
    pub latest_timestamp_unix_seconds: i64,
    pub records_received: usize,
    pub observations_created: usize,
    pub object_count: usize,
    pub freshness_seconds: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EventsTimelineResponse {
    pub generated_at_unix_seconds: i64,
    pub horizon_hours: f64,
    pub total_events: usize,
    pub buckets: Vec<EventsTimelineBucket>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EventsTimelineBucket {
    pub label: String,
    pub start_offset_hours: f64,
    pub end_offset_hours: f64,
    pub events: Vec<sss_core::RankedEvent>,
}

pub async fn health(headers: HeaderMap) -> Json<ApiEnvelope<HealthResponse>> {
    Json(ApiEnvelope::new(
        request_id(&headers),
        HealthResponse { status: "ok" },
    ))
}

pub async fn version(headers: HeaderMap) -> Json<ApiEnvelope<VersionResponse>> {
    Json(ApiEnvelope::new(
        request_id(&headers),
        VersionResponse {
            api_version: API_VERSION,
            core_model_version: "sss-behavioral-mvp-v1",
        },
    ))
}

pub async fn get_apod_briefing(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ApodBriefingQuery>,
) -> Result<Json<ApiEnvelope<crate::state::ApodBriefing>>, ApiError> {
    let request_id = request_id(&headers);
    tracing::info!(%request_id, ?query.date, "get_apod_briefing");
    let briefing = state
        .apod_briefing(&request_id, query.date.as_deref())
        .await
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, briefing)))
}

pub async fn get_neows_briefing(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<NeoWsBriefingQuery>,
) -> Result<Json<ApiEnvelope<crate::state::NeoRiskBriefing>>, ApiError> {
    let request_id = request_id(&headers);
    let start_date = query.start_date.unwrap_or_else(|| "2026-04-18".to_string());
    let end_date = query.end_date.unwrap_or_else(|| start_date.clone());
    tracing::info!(%request_id, %start_date, %end_date, "get_neows_briefing");
    let briefing = state
        .neows_briefing(&request_id, &start_date, &end_date)
        .await
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, briefing)))
}

pub async fn get_neows_feed(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<NeoWsBriefingQuery>,
) -> Result<Json<ApiEnvelope<crate::state::NeoRiskFeed>>, ApiError> {
    let request_id = request_id(&headers);
    let start_date = query.start_date.unwrap_or_else(|| "2026-04-18".to_string());
    let end_date = query.end_date.unwrap_or_else(|| start_date.clone());
    tracing::info!(%request_id, %start_date, %end_date, "get_neows_feed");
    let feed = state
        .neows_feed(&request_id, &start_date, &end_date)
        .await
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, feed)))
}

pub async fn operator_console() -> Html<&'static str> {
    Html(OPERATOR_CONSOLE_HTML)
}

pub async fn landing_page() -> Html<&'static str> {
    Html(LANDING_PAGE_HTML)
}

pub async fn ingest_tle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<IngestTleRequest>,
) -> Result<Json<ApiEnvelope<IngestTleResponse>>, ApiError> {
    let request_id = request_id(&headers);
    tracing::info!(%request_id, source = %request.source, "ingest_tle");
    let outcome = state
        .ingest_tle(&request_id, &request.source, &request.payload)
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(
        request_id,
        IngestTleResponse {
            source: request.source,
            source_url: None,
            records_received: outcome.records_received,
            observations_created: outcome.observations_created,
            object_ids: outcome.object_ids,
            payload_bytes: request.payload.len(),
            skipped_duplicate: outcome.skipped_duplicate,
            freshness_seconds: outcome.freshness_seconds,
        },
    )))
}

pub async fn ingest_celestrak_active(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<IngestTleResponse>>, ApiError> {
    let request_id = request_id(&headers);
    let url = sss_ingest::celestrak_active_url();
    tracing::info!(%request_id, %url, "ingest_celestrak_active");
    let (payload, outcome) = state
        .ingest_live_tle_source(&request_id, "celestrak-active", url)
        .await
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    tracing::info!(
        %request_id,
        %url,
        payload_bytes = payload.len(),
        skipped_duplicate = outcome.skipped_duplicate,
        freshness_seconds = outcome.freshness_seconds,
        "ingest_celestrak_active_fetched"
    );

    Ok(Json(ApiEnvelope::new(
        request_id,
        IngestTleResponse {
            source: "celestrak-active".to_string(),
            source_url: Some(url.to_string()),
            records_received: outcome.records_received,
            observations_created: outcome.observations_created,
            object_ids: outcome.object_ids,
            payload_bytes: payload.len(),
            skipped_duplicate: outcome.skipped_duplicate,
            freshness_seconds: outcome.freshness_seconds,
        },
    )))
}

pub async fn analyze_object(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<sss_core::AnalyzeObjectRequest>,
) -> Result<Json<ApiEnvelope<sss_core::AnalyzeObjectResponse>>, ApiError> {
    let request_id = request_id(&headers);
    tracing::info!(%request_id, object_id = %request.object_id, "analyze_object");
    let response = state
        .analyze(&request_id, request)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    tracing::info!(
        %request_id,
        bundle_hash = %response.evidence_bundle.bundle_hash,
        risk_score = response.risk_score,
        "analysis_completed"
    );
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn space_overview(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<sss_core::SpaceOverviewRequest>,
) -> Result<Json<ApiEnvelope<sss_core::SpaceOverviewResponse>>, ApiError> {
    let request_id = request_id(&headers);
    tracing::info!(%request_id, window_hours = request.window_hours, "space_overview");
    let response = state
        .overview(&request_id, request)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn predict(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<sss_core::PredictRequest>,
) -> Result<Json<ApiEnvelope<sss_core::PredictResponse>>, ApiError> {
    let request_id = request_id(&headers);
    tracing::info!(%request_id, object_id = %request.object_id, "predict");
    let response = state
        .predict(&request_id, request)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn passive_scan(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<sss_passive_scanner::PassiveScanInput>,
) -> Result<Json<ApiEnvelope<sss_passive_scanner::PassiveScanOutput>>, ApiError> {
    let request_id = request_id(&headers);
    tracing::info!(%request_id, sites = request.infra_sites.len(), "passive_scan");
    let response = state
        .passive_scan(&request_id, &request)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn passive_scan_live(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<crate::state::PassiveLiveScanRequest>,
) -> Result<Json<ApiEnvelope<crate::state::PassiveLiveScanResponse>>, ApiError> {
    let request_id = request_id(&headers);
    tracing::info!(%request_id, sites = request.infra_sites.len(), "passive_scan_live");
    let response = state
        .passive_scan_live(&request_id, &request)
        .await
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn discover_passive_sites(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<crate::state::PassiveInfraDiscoverRequest>,
) -> Result<Json<ApiEnvelope<crate::state::PassiveInfraDiscoverResponse>>, ApiError> {
    let request_id = request_id(&headers);
    tracing::info!(
        %request_id,
        south = request.south,
        west = request.west,
        north = request.north,
        east = request.east,
        "discover_passive_sites"
    );
    let response = state
        .discover_passive_infra_sites(&request_id, &request)
        .await
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn discover_and_scan_passive_sites(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<crate::state::PassiveDiscoverAndScanRequest>,
) -> Result<Json<ApiEnvelope<crate::state::PassiveDiscoverAndScanResponse>>, ApiError> {
    let request_id = request_id(&headers);
    tracing::info!(
        %request_id,
        south = request.discovery.south,
        west = request.discovery.west,
        north = request.discovery.north,
        east = request.discovery.east,
        "discover_and_scan_passive_sites"
    );
    let response = state
        .discover_and_scan_passive_sites(&request_id, &request)
        .await
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn run_passive_scheduler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<crate::state::PassiveScheduleScanRequest>,
) -> Result<Json<ApiEnvelope<crate::state::PassiveScheduleScanResponse>>, ApiError> {
    let request_id = request_id(&headers);
    tracing::info!(
        %request_id,
        limit = ?request.limit,
        minimum_priority = ?request.minimum_priority,
        force = request.force.unwrap_or(false),
        dry_run = request.dry_run.unwrap_or(false),
        "run_passive_scheduler"
    );
    let response = state
        .run_passive_scheduler(&request_id, &request)
        .await
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn upsert_passive_region(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<crate::state::PassiveRegionTargetRequest>,
) -> Result<Json<ApiEnvelope<sss_storage::PassiveRegionTarget>>, ApiError> {
    let request_id = request_id(&headers);
    tracing::info!(
        %request_id,
        name = %request.name,
        south = request.south,
        west = request.west,
        north = request.north,
        east = request.east,
        "upsert_passive_region"
    );
    let response = state
        .upsert_passive_region_target(&request)
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn bootstrap_default_passive_regions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<crate::state::PassiveRegionDefaultsBootstrapRequest>,
) -> Result<Json<ApiEnvelope<crate::state::PassiveRegionDefaultsBootstrapResponse>>, ApiError> {
    let request_id = request_id(&headers);
    tracing::info!(
        %request_id,
        overwrite_existing = request.overwrite_existing.unwrap_or(false),
        "bootstrap_default_passive_regions"
    );
    let response = state
        .bootstrap_default_passive_regions(&request)
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_regions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PassiveRegionsQuery>,
) -> Result<Json<ApiEnvelope<Vec<sss_storage::PassiveRegionTarget>>>, ApiError> {
    let request_id = request_id(&headers);
    let limit = query.limit.unwrap_or(50).clamp(1, 500);
    let response = state
        .passive_region_targets(limit, query.enabled_only.unwrap_or(false))
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_region_runs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PassiveRegionRunsQuery>,
) -> Result<Json<ApiEnvelope<Vec<sss_storage::PassiveRegionRunLog>>>, ApiError> {
    let request_id = request_id(&headers);
    let limit = query.limit.unwrap_or(25).clamp(1, 200);
    let response = state
        .passive_region_run_logs(limit, query.region_id.as_deref())
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_region_run(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(run_id): Path<String>,
) -> Result<Json<ApiEnvelope<sss_storage::PassiveRegionRunLog>>, ApiError> {
    let request_id = request_id(&headers);
    let response = state
        .passive_region_run_log(&run_id)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?
        .ok_or_else(|| {
            ApiError::not_found(
                request_id.clone(),
                "passive_region_run_not_found",
                format!("passive region run not found: {run_id}"),
            )
        })?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_region_leases(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<LimitQuery>,
) -> Result<Json<ApiEnvelope<Vec<sss_storage::PassiveRegionLease>>>, ApiError> {
    let request_id = request_id(&headers);
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    let response = state
        .passive_region_leases(limit)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_worker_heartbeats(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PassiveWorkerHeartbeatsQuery>,
) -> Result<Json<ApiEnvelope<Vec<sss_storage::PassiveWorkerHeartbeat>>>, ApiError> {
    let request_id = request_id(&headers);
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    let mut response = state
        .passive_worker_heartbeats(limit)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    if query.stale_only.unwrap_or(false) {
        let stale_cutoff = crate::state::now_unix_seconds().saturating_sub(180);
        response.retain(|heartbeat| heartbeat.last_heartbeat_unix_seconds < stale_cutoff);
    }
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_worker_diagnostics(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PassiveWorkerDiagnosticsQuery>,
) -> Result<Json<ApiEnvelope<crate::state::PassiveWorkerDiagnostics>>, ApiError> {
    let request_id = request_id(&headers);
    let limit = query.limit.unwrap_or(10).clamp(1, 100);
    let stale_after_seconds = query.stale_after_seconds.unwrap_or(180).max(1);
    let response = state
        .passive_worker_diagnostics(limit, stale_after_seconds, query.region_id.as_deref())
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_maintenance_summary(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PassiveMaintenanceSummaryQuery>,
) -> Result<Json<ApiEnvelope<PassiveMaintenanceSummary>>, ApiError> {
    let request_id = request_id(&headers);
    let generated_at_unix_seconds = crate::state::now_unix_seconds();
    let heartbeat_retention_seconds = query.heartbeat_retention_seconds.unwrap_or(86_400).max(1);
    let source_retention_seconds = query.source_retention_seconds.unwrap_or(604_800).max(1);
    let heartbeat_cutoff = generated_at_unix_seconds.saturating_sub(heartbeat_retention_seconds);
    let source_cutoff = generated_at_unix_seconds.saturating_sub(source_retention_seconds);
    let stale_heartbeat_count = state
        .count_stale_passive_worker_heartbeats(heartbeat_cutoff)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    let source_health_prune_candidate_count = state
        .count_passive_source_health_samples_before(
            source_cutoff,
            query.source_kind,
            query.region_id.as_deref(),
        )
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    let mut suggested_actions = Vec::new();
    if stale_heartbeat_count > 0 {
        suggested_actions.push(PassiveMaintenanceAction {
            action_id: "prune-stale-heartbeats".to_string(),
            title: "Prune stale worker heartbeats".to_string(),
            reason: format!("{stale_heartbeat_count} heartbeats are older than retention."),
            method: "POST",
            path: "/v1/passive/worker/heartbeats/prune".to_string(),
            confirmation_read_paths: vec![
                "/v1/passive/worker/diagnostics?stale_after_seconds=86400".to_string(),
                "/v1/passive/worker/heartbeats?stale_only=true".to_string(),
            ],
        });
    }
    if source_health_prune_candidate_count > 0 {
        suggested_actions.push(PassiveMaintenanceAction {
            action_id: "prune-source-health-samples".to_string(),
            title: "Prune old source health samples".to_string(),
            reason: format!(
                "{source_health_prune_candidate_count} source health samples are older than retention."
            ),
            method: "POST",
            path: "/v1/passive/source-health/samples/prune".to_string(),
            confirmation_read_paths: vec![
                format!(
                    "/v1/passive/source-health/samples?limit=20{}{}",
                    query
                        .source_kind
                        .map(|source_kind| format!("&source_kind={source_kind:?}"))
                        .unwrap_or_default(),
                    query
                        .region_id
                        .as_ref()
                        .map(|region_id| format!("&region_id={region_id}"))
                        .unwrap_or_default()
                ),
                "/v1/passive/worker/diagnostics".to_string(),
            ],
        });
    }
    Ok(Json(ApiEnvelope::new(
        request_id,
        PassiveMaintenanceSummary {
            generated_at_unix_seconds,
            heartbeat_retention_seconds,
            source_retention_seconds,
            stale_heartbeat_count,
            source_health_prune_candidate_count,
            source_kind: query.source_kind,
            region_id: query.region_id,
            suggested_actions,
        },
    )))
}

pub async fn get_passive_region_remediation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(region_id): Path<String>,
    Query(query): Query<PassiveRegionRemediationQuery>,
) -> Result<Json<ApiEnvelope<crate::remediation_views::PassiveRegionRemediationResponse>>, ApiError>
{
    let request_id = request_id(&headers);
    let response = crate::remediation_views::build_passive_region_remediation_response(
        &state,
        &region_id,
        query.site_limit.unwrap_or(10).clamp(1, 50),
        query.days.unwrap_or(30).clamp(1, 365),
        query.stale_after_seconds.unwrap_or(180).max(1),
    )
    .map_err(|error| app_error_to_api_error(request_id.clone(), error))?
    .ok_or_else(|| {
        ApiError::not_found(
            request_id.clone(),
            "passive_region_not_found",
            format!("passive region not found: {region_id}"),
        )
    })?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_region_operational_timeline(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(region_id): Path<String>,
    Query(query): Query<crate::operational_timeline::OperationalTimelineQuery>,
) -> Result<Json<ApiEnvelope<crate::operational_timeline::OperationalTimelineResponse>>, ApiError> {
    let request_id = request_id(&headers);
    let response =
        crate::operational_timeline::build_region_operational_timeline(&state, &region_id, &query)
            .map_err(|error| app_error_to_api_error(request_id.clone(), error))?
            .ok_or_else(|| {
                ApiError::not_found(
                    request_id.clone(),
                    "passive_region_not_found",
                    format!("passive region not found: {region_id}"),
                )
            })?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn prune_passive_worker_heartbeats(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<PassiveWorkerHeartbeatPruneRequest>,
) -> Result<Json<ApiEnvelope<PassiveWorkerHeartbeatPruneResponse>>, ApiError> {
    let request_id = request_id(&headers);
    let generated_at_unix_seconds = crate::state::now_unix_seconds();
    let older_than_seconds = request.older_than_seconds.unwrap_or(86_400).max(1);
    let stale_before_unix_seconds = generated_at_unix_seconds.saturating_sub(older_than_seconds);
    let pruned_count = state
        .purge_stale_passive_worker_heartbeats(stale_before_unix_seconds)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    Ok(Json(ApiEnvelope::new(
        request_id,
        PassiveWorkerHeartbeatPruneResponse {
            generated_at_unix_seconds,
            stale_before_unix_seconds,
            pruned_count,
        },
    )))
}

pub async fn get_passive_source_health_samples(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PassiveSourceHealthSamplesQuery>,
) -> Result<Json<ApiEnvelope<Vec<sss_storage::PassiveSourceHealthSample>>>, ApiError> {
    let request_id = request_id(&headers);
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    let response = state
        .passive_source_health_samples(limit, query.source_kind, query.region_id.as_deref())
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_source_readiness(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PassiveSourceHealthSamplesQuery>,
) -> Result<Json<ApiEnvelope<PassiveSourceReadinessResponse>>, ApiError> {
    let request_id = request_id(&headers);
    let scheduler = PassiveSchedulerRuntimeReadiness {
        enabled: env_u64("SSS_PASSIVE_REGION_POLL_SECONDS").unwrap_or(0) > 0,
        poll_seconds: env_u64("SSS_PASSIVE_REGION_POLL_SECONDS"),
        retry_seconds: env_u64("SSS_PASSIVE_REGION_RETRY_SECONDS"),
        window_hours: env_u64("SSS_PASSIVE_REGION_WINDOW_HOURS"),
        include_weather: env_bool("SSS_PASSIVE_REGION_INCLUDE_WEATHER", true),
        include_fire_smoke: env_bool("SSS_PASSIVE_REGION_INCLUDE_FIRE_SMOKE", true),
        include_adsb: env_bool("SSS_PASSIVE_REGION_INCLUDE_ADSB", true),
        force_discovery: env_bool("SSS_PASSIVE_REGION_FORCE_DISCOVERY", false),
        startup_discovery_enabled: env_bool("SSS_API_ENABLE_STARTUP_DISCOVERY", false),
    };
    let response = PassiveSourceReadinessResponse {
        generated_at_unix_seconds: crate::state::now_unix_seconds(),
        scheduler,
        sources: state
            .passive_source_readiness(query.region_id.as_deref())
            .map_err(|error| app_error_to_api_error(request_id.clone(), error))?,
    };
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn prune_passive_source_health_samples(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<PassiveSourceHealthPruneRequest>,
) -> Result<Json<ApiEnvelope<PassiveSourceHealthPruneResponse>>, ApiError> {
    let request_id = request_id(&headers);
    let generated_at_unix_seconds = crate::state::now_unix_seconds();
    let older_than_seconds = request.older_than_seconds.unwrap_or(2_592_000).max(1);
    let older_than_unix_seconds = generated_at_unix_seconds.saturating_sub(older_than_seconds);
    let dry_run = request.dry_run.unwrap_or(false);
    let pruned_count = if dry_run {
        state.count_passive_source_health_samples_before(
            older_than_unix_seconds,
            request.source_kind,
            request.region_id.as_deref(),
        )
    } else {
        state.purge_passive_source_health_samples(
            older_than_unix_seconds,
            request.source_kind,
            request.region_id.as_deref(),
        )
    }
    .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    Ok(Json(ApiEnvelope::new(
        request_id,
        PassiveSourceHealthPruneResponse {
            generated_at_unix_seconds,
            older_than_unix_seconds,
            source_kind: request.source_kind,
            region_id: request.region_id,
            dry_run,
            pruned_count,
        },
    )))
}

pub async fn run_passive_regions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<crate::state::PassiveRegionRunRequest>,
) -> Result<Json<ApiEnvelope<crate::state::PassiveRegionRunResponse>>, ApiError> {
    let request_id = request_id(&headers);
    tracing::info!(
        %request_id,
        force_discovery = request.force_discovery.unwrap_or(false),
        dry_run = request.dry_run.unwrap_or(false),
        "run_passive_regions"
    );
    let response = state
        .run_passive_regions(&request_id, &request)
        .await
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_operations_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<crate::state::PassiveOperationsStatus>>, ApiError> {
    let request_id = request_id(&headers);
    let response = state
        .passive_operations_status()
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_dashboard_summary(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<crate::dashboard_views::PassiveDashboardSummaryQuery>,
) -> Result<Json<ApiEnvelope<crate::dashboard_views::PassiveDashboardSummary>>, ApiError> {
    let request_id = request_id(&headers);
    let response = crate::dashboard_views::build_passive_dashboard_summary(&state, &query)
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_operational_visibility(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<crate::operational_visibility_views::OperationalVisibilityQuery>,
) -> Result<
    Json<ApiEnvelope<crate::operational_visibility_views::OperationalVisibilitySummary>>,
    ApiError,
> {
    let request_id = request_id(&headers);
    let response =
        crate::operational_visibility_views::build_operational_visibility_summary(&state, &query)
            .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_command_center_summary(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<crate::command_center_views::PassiveCommandCenterSummaryQuery>,
) -> Result<Json<ApiEnvelope<crate::command_center_views::PassiveCommandCenterSummary>>, ApiError> {
    let request_id = request_id(&headers);
    let response =
        crate::command_center_views::build_passive_command_center_summary(&state, &query)
            .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_canonical_events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<crate::canonical_views::CanonicalEventsQuery>,
) -> Result<Json<ApiEnvelope<crate::canonical_views::CanonicalEventsResponse>>, ApiError> {
    let request_id = request_id(&headers);
    let response = crate::canonical_views::build_canonical_events_response(&state, &query)
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_canonical_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(canonical_event_id): Path<String>,
) -> Result<Json<ApiEnvelope<sss_storage::CanonicalPassiveEvent>>, ApiError> {
    let request_id = request_id(&headers);
    let response = state
        .canonical_passive_event(&canonical_event_id)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?
        .ok_or_else(|| {
            ApiError::not_found(
                request_id.clone(),
                "passive_canonical_event_not_found",
                format!("passive canonical event not found: {canonical_event_id}"),
            )
        })?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_map_regions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<crate::map_views::RegionMapResponse>>, ApiError> {
    let request_id = request_id(&headers);
    let response = crate::map_views::build_region_map_response(&state)
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_map_sites(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<crate::map_views::MapSitesQuery>,
) -> Result<Json<ApiEnvelope<crate::map_views::SiteMapResponse>>, ApiError> {
    let request_id = request_id(&headers);
    let response = crate::map_views::build_site_map_response(&state, &query)
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_map_events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<crate::map_views::MapEventsQuery>,
) -> Result<Json<ApiEnvelope<crate::map_views::EventMapResponse>>, ApiError> {
    let request_id = request_id(&headers);
    let response = crate::map_views::build_event_map_response(&state, &query)
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_region_overview(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(region_id): Path<String>,
    Query(query): Query<PassiveRegionOverviewQuery>,
) -> Result<Json<ApiEnvelope<crate::state::PassiveRegionOverview>>, ApiError> {
    let request_id = request_id(&headers);
    let response = state
        .passive_region_overview(
            &region_id,
            query.site_limit.unwrap_or(10).clamp(1, 50),
            query.days.unwrap_or(30).clamp(1, 365),
        )
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?
        .ok_or_else(|| {
            ApiError::not_found(
                request_id.clone(),
                "passive_region_not_found",
                format!("passive region not found: {region_id}"),
            )
        })?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_sites(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<Vec<sss_passive_scanner::PassiveSiteProfile>>>, ApiError> {
    let request_id = request_id(&headers);
    let response = state
        .passive_sites()
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_seeds(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PassiveSeedsQuery>,
) -> Result<Json<ApiEnvelope<Vec<sss_storage::PassiveSeedRecord>>>, ApiError> {
    let request_id = request_id(&headers);
    let limit = query.limit.unwrap_or(50).clamp(1, 500);
    let response = state
        .passive_seed_records(limit)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_seed(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(seed_key): Path<String>,
) -> Result<Json<ApiEnvelope<sss_storage::PassiveSeedRecord>>, ApiError> {
    let request_id = request_id(&headers);
    let response = state
        .passive_seed_record(&seed_key)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?
        .ok_or_else(|| {
            ApiError::not_found(
                request_id.clone(),
                "passive_seed_not_found",
                format!("passive seed not found: {seed_key}"),
            )
        })?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_observations(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PassiveObservationsQuery>,
) -> Result<Json<ApiEnvelope<Vec<sss_passive_scanner::PassiveObservation>>>, ApiError> {
    let request_id = request_id(&headers);
    let limit = query.limit.unwrap_or(25).clamp(1, 100);
    let response = state
        .passive_observations(limit, query.source_kind)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_site_events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_id): Path<String>,
    Query(query): Query<PassiveSiteCorpusQuery>,
) -> Result<Json<ApiEnvelope<Vec<sss_passive_scanner::PassiveEvent>>>, ApiError> {
    let request_id = request_id(&headers);
    let limit = query.limit.unwrap_or(25).clamp(1, 100);
    let response = state
        .passive_events_for_site(&site_id, limit)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_site_risk_history(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_id): Path<String>,
    Query(query): Query<PassiveSiteCorpusQuery>,
) -> Result<Json<ApiEnvelope<Vec<sss_passive_scanner::PassiveRiskRecord>>>, ApiError> {
    let request_id = request_id(&headers);
    let limit = query.limit.unwrap_or(25).clamp(1, 100);
    let response = state
        .passive_risk_history(&site_id, limit)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_site_patterns(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_id): Path<String>,
    Query(query): Query<PassiveSiteCorpusQuery>,
) -> Result<Json<ApiEnvelope<Vec<sss_passive_scanner::PatternSignal>>>, ApiError> {
    let request_id = request_id(&headers);
    let limit = query.limit.unwrap_or(25).clamp(1, 100);
    let response = state
        .passive_patterns(&site_id, limit)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_site_overview(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_id): Path<String>,
    Query(query): Query<PassiveSiteCorpusQuery>,
) -> Result<Json<ApiEnvelope<crate::state::PassiveSiteOverview>>, ApiError> {
    let request_id = request_id(&headers);
    let limit = query.limit.unwrap_or(25).clamp(1, 100);
    let response = state
        .passive_site_overview(&site_id, limit, 30)
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_site_narrative(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_id): Path<String>,
    Query(query): Query<PassiveSiteNarrativeQuery>,
) -> Result<Json<ApiEnvelope<crate::state::RiskHistoryNarrative>>, ApiError> {
    let request_id = request_id(&headers);
    let days = query.days.unwrap_or(30).clamp(1, 365);
    let response = state
        .passive_risk_history_narrative(&site_id, days)
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_site_solar_forecast(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_id): Path<String>,
    Query(query): Query<PassiveSiteForecastQuery>,
) -> Result<Json<ApiEnvelope<sss_forecast::SolarForecastResponse>>, ApiError> {
    let request_id = request_id(&headers);
    let horizon_hours = query.horizon_hours.unwrap_or(24).clamp(1, 72);
    let response = state
        .passive_site_solar_forecast(&request_id, &site_id, horizon_hours)
        .await
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_site_load_forecast(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_id): Path<String>,
    Query(query): Query<PassiveSiteForecastQuery>,
) -> Result<Json<ApiEnvelope<sss_forecast::LoadForecastResponse>>, ApiError> {
    let request_id = request_id(&headers);
    let horizon_hours = query.horizon_hours.unwrap_or(24).clamp(1, 72);
    let response = state
        .passive_site_load_forecast(&request_id, &site_id, horizon_hours)
        .await
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_site_scenario_forecast(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_id): Path<String>,
    Query(query): Query<PassiveSiteForecastQuery>,
) -> Result<Json<ApiEnvelope<sss_forecast::ScenarioForecastResponse>>, ApiError> {
    let request_id = request_id(&headers);
    let horizon_hours = query.horizon_hours.unwrap_or(24).clamp(1, 72);
    let response = state
        .passive_site_scenario_forecast(&request_id, &site_id, horizon_hours)
        .await
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn recommend_passive_site_action(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_id): Path<String>,
    Json(request): Json<sss_forecast::OrchestratorRecommendationRequest>,
) -> Result<Json<ApiEnvelope<sss_forecast::OrchestratorRecommendation>>, ApiError> {
    let request_id = request_id(&headers);
    let response = state
        .passive_site_recommendation(&request_id, &site_id, &request)
        .await
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_site_orchestrator_decisions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_id): Path<String>,
    Query(query): Query<PredictionSnapshotsQuery>,
) -> Result<Json<ApiEnvelope<Vec<sss_storage::PredictionSnapshot>>>, ApiError> {
    let request_id = request_id(&headers);
    let limit = query.limit.unwrap_or(10).clamp(1, 100);
    let response = state
        .passive_site_orchestrator_decisions(&site_id, limit)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_site_recommendation_reviews(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_id): Path<String>,
    Query(query): Query<PassiveRecommendationReviewsQuery>,
) -> Result<Json<ApiEnvelope<Vec<PassiveRecommendationReviewView>>>, ApiError> {
    let request_id = request_id(&headers);
    let limit = query.limit.unwrap_or(10).clamp(1, 100);
    let now = crate::state::now_unix_seconds();
    let response = state
        .passive_site_recommendation_reviews(&site_id, limit)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?
        .into_iter()
        .map(|review| {
            let age_seconds = now.saturating_sub(review.decided_at_unix_seconds);
            let age_bucket = if age_seconds <= 6 * 3_600 {
                crate::map_views::ReviewAgeBucket::New
            } else if age_seconds <= 24 * 3_600 {
                crate::map_views::ReviewAgeBucket::Aging
            } else {
                crate::map_views::ReviewAgeBucket::Stale
            };
            PassiveRecommendationReviewView {
                review_id: review.review_id,
                site_id: review.site_id,
                snapshot_id: review.snapshot_id,
                request_id: review.request_id,
                endpoint: review.endpoint,
                state: review.state,
                actor: review.actor,
                rationale: review.rationale,
                evidence_bundle_hash: review.evidence_bundle_hash,
                decided_at_unix_seconds: review.decided_at_unix_seconds,
                age_seconds,
                age_bucket,
            }
        })
        .collect::<Vec<_>>();
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn review_passive_site_recommendation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_id): Path<String>,
    Json(request): Json<PassiveSiteRecommendationReviewRequest>,
) -> Result<Json<ApiEnvelope<sss_storage::PassiveRecommendationReview>>, ApiError> {
    let request_id = request_id(&headers);
    let response = state
        .set_passive_site_recommendation_review(
            &site_id,
            request.state,
            request.actor,
            request.rationale,
            request.snapshot_id.as_ref(),
        )
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_site_semantic_timeline(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(site_id): Path<String>,
    Query(query): Query<crate::semantic_timeline::SemanticTimelineQuery>,
) -> Result<Json<ApiEnvelope<crate::semantic_timeline::SemanticTimelineResponse>>, ApiError> {
    let request_id = request_id(&headers);
    let response = crate::semantic_timeline::build_site_semantic_timeline(&state, &site_id, &query)
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_passive_region_semantic_timeline(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(region_id): Path<String>,
    Query(query): Query<crate::semantic_timeline::SemanticTimelineQuery>,
) -> Result<Json<ApiEnvelope<crate::semantic_timeline::SemanticTimelineResponse>>, ApiError> {
    let request_id = request_id(&headers);
    let response =
        crate::semantic_timeline::build_region_semantic_timeline(&state, &region_id, &query)
            .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<EventsQuery>,
) -> Result<Json<ApiEnvelope<Vec<sss_core::RankedEvent>>>, ApiError> {
    let request_id = request_id(&headers);
    let limit = query.limit.unwrap_or(25).clamp(1, 100);
    tracing::info!(
        %request_id,
        ?query.object_id,
        ?query.event_type,
        future_only = query.future_only.unwrap_or(false),
        ?query.after_unix_seconds,
        ?query.before_unix_seconds,
        "get_events"
    );
    let events = state
        .ranked_events(
            limit,
            &crate::state::RankedEventsFilter {
                object_id: query.object_id,
                event_type: query.event_type,
                future_only: query.future_only.unwrap_or(false),
                after_unix_seconds: query.after_unix_seconds,
                before_unix_seconds: query.before_unix_seconds,
            },
        )
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    Ok(Json(ApiEnvelope::new(request_id, events)))
}

pub async fn get_events_timeline(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<EventsTimelineQuery>,
) -> Result<Json<ApiEnvelope<EventsTimelineResponse>>, ApiError> {
    let request_id = request_id(&headers);
    let horizon_hours = query.horizon.unwrap_or(72.0).clamp(1.0, 168.0);
    let limit_per_bucket = query.limit_per_bucket.unwrap_or(10).clamp(1, 100);
    let reference_unix_seconds = query
        .reference_unix_seconds
        .unwrap_or_else(crate::state::now_unix_seconds);
    tracing::info!(
        %request_id,
        horizon_hours,
        limit_per_bucket,
        reference_unix_seconds,
        ?query.object_id,
        ?query.event_type,
        "get_events_timeline"
    );
    let timeline = state
        .ranked_events_timeline(
            horizon_hours,
            limit_per_bucket,
            reference_unix_seconds,
            &crate::state::RankedEventsFilter {
                object_id: query.object_id,
                event_type: query.event_type,
                future_only: true,
                after_unix_seconds: None,
                before_unix_seconds: None,
            },
        )
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    Ok(Json(ApiEnvelope::new(request_id, timeline)))
}

pub async fn get_ingest_batches(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<IngestBatchesQuery>,
) -> Result<Json<ApiEnvelope<Vec<sss_storage::IngestBatchLog>>>, ApiError> {
    let request_id = request_id(&headers);
    let limit = query.limit.unwrap_or(10).clamp(1, 100);
    let batches = state
        .ingest_batches(limit)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    Ok(Json(ApiEnvelope::new(request_id, batches)))
}

pub async fn dispatch_notifications(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<sss_core::AnalyzeObjectRequest>,
) -> Result<Json<ApiEnvelope<crate::state::NotificationDispatchResponse>>, ApiError> {
    let request_id = request_id(&headers);
    tracing::info!(%request_id, object_id = %request.object_id, "dispatch_notifications");
    let response = state
        .dispatch_notifications_for_analysis(&request_id, request)
        .await
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn dispatch_event_notifications(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<EventDispatchRequest>,
) -> Result<Json<ApiEnvelope<crate::state::EventDispatchResponse>>, ApiError> {
    let request_id = request_id(&headers);
    tracing::info!(%request_id, event_id = %request.event_id, "dispatch_event_notifications");
    let response = state
        .dispatch_notifications_for_event(&request_id, &request.event_id)
        .await
        .map_err(|error| app_error_to_api_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, response)))
}

pub async fn get_notification_deliveries(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<NotificationDeliveriesQuery>,
) -> Result<Json<ApiEnvelope<Vec<sss_storage::NotificationDeliveryLog>>>, ApiError> {
    let request_id = request_id(&headers);
    let limit = query.limit.unwrap_or(25).clamp(1, 100);
    let deliveries = state
        .notification_deliveries(limit, query.object_id.as_deref())
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    Ok(Json(ApiEnvelope::new(request_id, deliveries)))
}

pub async fn get_prediction_snapshots(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(object_id): Path<String>,
    Query(query): Query<PredictionSnapshotsQuery>,
) -> Result<Json<ApiEnvelope<Vec<sss_storage::PredictionSnapshot>>>, ApiError> {
    let request_id = request_id(&headers);
    let limit = query.limit.unwrap_or(10).clamp(1, 100);
    tracing::info!(%request_id, %object_id, limit, "get_prediction_snapshots");
    let snapshots = state
        .prediction_snapshots(&object_id, limit)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    Ok(Json(ApiEnvelope::new(request_id, snapshots)))
}

pub async fn get_ingest_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<IngestStatusQuery>,
) -> Result<Json<ApiEnvelope<IngestStatusResponse>>, ApiError> {
    let request_id = request_id(&headers);
    let Some(status) = state
        .ingest_source_status(&query.source)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?
    else {
        return Err(ApiError::not_found(
            request_id,
            "ingest_source_not_found",
            format!("ingest source not found: {}", query.source),
        ));
    };

    Ok(Json(ApiEnvelope::new(
        request_id,
        IngestStatusResponse {
            source: status.source,
            latest_request_id: status.latest_request_id,
            latest_timestamp_unix_seconds: status.latest_timestamp_unix_seconds,
            records_received: status.records_received,
            observations_created: status.observations_created,
            object_count: status.object_count,
            freshness_seconds: status.freshness_seconds,
        },
    )))
}

pub async fn get_assessment_history(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(object_id): Path<String>,
) -> Result<Json<ApiEnvelope<Vec<sss_core::IntelligenceAssessment>>>, ApiError> {
    let request_id = request_id(&headers);
    let assessments = state
        .assessment_history(&object_id)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?;
    Ok(Json(ApiEnvelope::new(request_id, assessments)))
}

pub async fn get_object_timeline(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(object_id): Path<String>,
    Query(query): Query<TimelineQuery>,
) -> Result<Json<ApiEnvelope<sss_core::ObjectTimelineResponse>>, ApiError> {
    let request_id = request_id(&headers);
    let horizon_hours = query.horizon.unwrap_or(72.0).clamp(1.0, 168.0);
    let Some(timeline) = state
        .timeline(
            &request_id,
            sss_core::TimelineRequest {
                object_id: object_id.clone(),
                horizon_hours,
            },
        )
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?
    else {
        return Err(ApiError::not_found(
            request_id,
            "object_not_found",
            format!("timeline object not found: {object_id}"),
        ));
    };
    Ok(Json(ApiEnvelope::new(request_id, timeline)))
}

pub async fn get_object_close_approaches(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(object_id): Path<String>,
    Query(query): Query<CloseApproachesQuery>,
) -> Result<Json<ApiEnvelope<crate::state::CloseApproachBriefing>>, ApiError> {
    let request_id = request_id(&headers);
    let horizon_hours = query.horizon.unwrap_or(72.0).clamp(1.0, 168.0);
    let threshold_km = query.threshold_km.unwrap_or(250.0).clamp(1.0, 20_000.0);
    let limit = query.limit.unwrap_or(5).clamp(1, 25);
    tracing::info!(
        %request_id,
        %object_id,
        horizon_hours,
        threshold_km,
        limit,
        "get_object_close_approaches"
    );
    let Some(briefing) = state
        .close_approaches(&request_id, &object_id, horizon_hours, threshold_km, limit)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?
    else {
        return Err(ApiError::not_found(
            request_id,
            "object_not_found",
            format!("close-approach object not found: {object_id}"),
        ));
    };
    Ok(Json(ApiEnvelope::new(request_id, briefing)))
}

pub async fn get_evidence_bundle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(bundle_hash): Path<String>,
) -> Result<Json<ApiEnvelope<sss_core::EvidenceBundle>>, ApiError> {
    let request_id = request_id(&headers);
    let Some(bundle) = state
        .evidence_bundle(&bundle_hash)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?
    else {
        return Err(ApiError::not_found(
            request_id,
            "evidence_bundle_not_found",
            format!("evidence bundle not found: {bundle_hash}"),
        ));
    };
    Ok(Json(ApiEnvelope::new(request_id, bundle)))
}

pub async fn get_replay_manifest_for_bundle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(bundle_hash): Path<String>,
) -> Result<Json<ApiEnvelope<sss_core::ReplayManifest>>, ApiError> {
    let request_id = request_id(&headers);
    let Some(manifest) = state
        .replay_manifest_for_bundle(&bundle_hash)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?
    else {
        return Err(ApiError::not_found(
            request_id,
            "replay_manifest_not_found",
            format!("replay manifest not found for bundle: {bundle_hash}"),
        ));
    };
    Ok(Json(ApiEnvelope::new(request_id, manifest)))
}

pub async fn get_replay_manifest(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(manifest_hash): Path<String>,
) -> Result<Json<ApiEnvelope<sss_core::ReplayManifest>>, ApiError> {
    let request_id = request_id(&headers);
    let Some(manifest) = state
        .replay_manifest(&manifest_hash)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?
    else {
        return Err(ApiError::not_found(
            request_id,
            "replay_manifest_not_found",
            format!("replay manifest not found: {manifest_hash}"),
        ));
    };
    Ok(Json(ApiEnvelope::new(request_id, manifest)))
}

pub async fn execute_replay(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(manifest_hash): Path<String>,
) -> Result<Json<ApiEnvelope<sss_core::ReplayExecution>>, ApiError> {
    let request_id = request_id(&headers);
    let Some(execution) = state
        .execute_replay(&manifest_hash)
        .map_err(|error| ApiError::storage(request_id.clone(), error.to_string()))?
    else {
        return Err(ApiError::not_found(
            request_id,
            "replay_execution_not_found",
            format!("replay execution not found: {manifest_hash}"),
        ));
    };
    Ok(Json(ApiEnvelope::new(request_id, execution)))
}

fn app_error_to_api_error(request_id: String, error: AppError) -> ApiError {
    match error {
        AppError::Ingest(error) => ApiError::invalid_request(request_id, error.to_string()),
        AppError::Serde(error) => ApiError::invalid_request(request_id, error.to_string()),
        AppError::InvalidRequest(message) | AppError::InvalidDateRange(message) => {
            ApiError::invalid_request(request_id, message)
        }
        AppError::Storage(error) => ApiError::storage(request_id, error.to_string()),
        AppError::NasaApiUnavailable(message) => {
            ApiError::nasa_api_unavailable(request_id, message)
        }
        AppError::SourceUnavailable(message) => ApiError::source_unavailable(request_id, message),
        AppError::NotificationUnavailable(message) => {
            ApiError::notification_unavailable(request_id, message)
        }
        AppError::EventNotFound(message) => {
            ApiError::not_found(request_id, "ranked_event_not_found", message)
        }
        AppError::SiteNotFound(message) => {
            ApiError::not_found(request_id, "site_not_found", message)
        }
    }
}

fn env_bool(key: &str, default: bool) -> bool {
    std::env::var(key).ok().map_or(default, |value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

fn env_u64(key: &str) -> Option<u64> {
    std::env::var(key).ok()?.trim().parse::<u64>().ok()
}

fn request_id(headers: &HeaderMap) -> String {
    headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
        .map_or_else(
            || {
                let id = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
                format!("sss-req-{id}")
            },
            str::to_string,
        )
}

const OPERATOR_CONSOLE_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8" />
<meta name="viewport" content="width=device-width, initial-scale=1.0"/>
<title>ShieldSky &middot; Operator Console</title>
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Sora:wght@300;400;500;600;700&family=Inter:wght@300;400;500;600&family=JetBrains+Mono:wght@400;500;600&display=swap" rel="stylesheet">
<link href="https://cesium.com/downloads/cesiumjs/releases/1.127/Build/Cesium/Widgets/widgets.css" rel="stylesheet">
<script src="https://cesium.com/downloads/cesiumjs/releases/1.127/Build/Cesium/Cesium.js"></script>
<style>
  /* ============================================================
     SHIELDSKY · COMMAND SURFACE
     Design tokens — operational intelligence, not corporate UI
     ============================================================ */
  :root {
    --bg-deep:        #07111B;
    --bg-surface:     #0D1826;
    --bg-elevated:    #122235;
    --bg-panel:       rgba(13, 24, 38, 0.72);
    --line:           rgba(160, 190, 220, 0.14);
    --line-strong:    rgba(160, 190, 220, 0.28);
    --text-primary:   #ECF4FF;
    --text-secondary: #9EB3C8;
    --text-tertiary:  #5F7389;
    --teal:           #4FE0D0;
    --teal-dim:       rgba(79, 224, 208, 0.15);
    --amber:          #F1C96B;
    --amber-dim:      rgba(241, 201, 107, 0.15);
    --coral:          #FF7C6E;
    --coral-dim:      rgba(255, 124, 110, 0.18);
    --lime:           #8EE59B;
    --lime-dim:       rgba(142, 229, 155, 0.15);
    --violet:         #9B8CFF;

    --font-display: 'Sora', sans-serif;
    --font-body:    'Inter', sans-serif;
    --font-mono:    'JetBrains Mono', monospace;

    --rail-width: 340px;
    --top-height: 56px;
    --bottom-height: 112px;
  }

  * { box-sizing: border-box; margin: 0; padding: 0; }

  html, body {
    height: 100%;
    overflow: hidden;
    background: var(--bg-deep);
    font-family: var(--font-body);
    color: var(--text-primary);
    -webkit-font-smoothing: antialiased;
  }

  /* ============================================================
     MASTER GRID
     ============================================================ */
  .console {
    display: grid;
    height: 100vh;
    grid-template-columns: var(--rail-width) 1fr var(--rail-width);
    grid-template-rows: var(--top-height) 1fr var(--bottom-height);
    grid-template-areas:
      "top    top    top"
      "left   center right"
      "bottom bottom bottom";
  }

  /* ============================================================
     TOP COMMAND BAR
     ============================================================ */
  .top-bar {
    grid-area: top;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--line);
    display: flex;
    align-items: center;
    gap: 0;
    padding: 0;
    position: relative;
    z-index: 10;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 18px;
    height: 100%;
    border-right: 1px solid var(--line);
    min-width: 180px;
  }

  .brand-mark {
    width: 28px;
    height: 28px;
    flex-shrink: 0;
  }

  .brand-mark svg { width: 100%; height: 100%; }

  .brand-text { display: flex; flex-direction: column; gap: 1px; }

  .brand-name {
    font-family: var(--font-display);
    font-size: 13px;
    font-weight: 600;
    letter-spacing: 0.02em;
    color: var(--text-primary);
  }

  .brand-context {
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 0.2em;
    text-transform: uppercase;
    color: var(--text-tertiary);
  }

  /* Chips */
  .chip-group {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 0 12px;
    height: 100%;
    border-right: 1px solid var(--line);
  }

  .chip {
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--text-tertiary);
    background: transparent;
    border: none;
    padding: 5px 9px;
    border-radius: 2px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .chip:hover { color: var(--text-secondary); background: var(--bg-elevated); }
  .chip.active { color: var(--teal); background: var(--teal-dim); }

  /* Search */
  .search {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 16px;
    height: 100%;
    border-right: 1px solid var(--line);
  }

  .search-icon { color: var(--text-tertiary); flex-shrink: 0; }

  .search input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    font-family: var(--font-body);
    font-size: 12.5px;
    color: var(--text-primary);
  }

  .search input::placeholder { color: var(--text-tertiary); }

  .search-kbd {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-tertiary);
    background: var(--bg-elevated);
    border: 1px solid var(--line-strong);
    padding: 2px 5px;
    border-radius: 2px;
  }

  /* Sys status */
  .sys-status {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 0 16px;
    height: 100%;
    border-right: 1px solid var(--line);
  }

  .sys-item {
    display: flex;
    align-items: center;
    gap: 6px;
    font-family: var(--font-mono);
    font-size: 10.5px;
    color: var(--text-secondary);
  }

  .pulse-dot {
    width: 6px; height: 6px;
    border-radius: 50%;
    background: var(--lime);
    box-shadow: 0 0 6px var(--lime);
    animation: pulse 2s infinite ease-in-out;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  /* Operator avatar */
  .operator {
    width: 30px; height: 30px;
    border-radius: 50%;
    background: var(--teal-dim);
    border: 1px solid var(--teal);
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 500;
    color: var(--teal);
    margin: 0 14px;
    cursor: pointer;
    flex-shrink: 0;
  }

  /* ============================================================
     LEFT INTELLIGENCE RAIL
     ============================================================ */
  .left-rail {
    grid-area: left;
    background: var(--bg-surface);
    border-right: 1px solid var(--line);
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  /* Panel structure */
  .panel {
    border-bottom: 1px solid var(--line);
    padding: 14px 16px;
    flex-shrink: 0;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
  }

  .panel-title {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 500;
    letter-spacing: 0.22em;
    text-transform: uppercase;
    color: var(--text-tertiary);
  }

  .panel-title.violet { color: var(--violet); }
  .panel-title.amber  { color: var(--amber); }
  .panel-title.coral  { color: var(--coral); }

  .panel-count {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-tertiary);
    letter-spacing: 0.06em;
  }

  /* Change items */
  .change-item {
    display: flex;
    gap: 10px;
    padding: 9px 0;
    border-bottom: 1px solid var(--line);
  }
  .change-item:last-child { border-bottom: none; }

  .change-marker {
    width: 2px;
    border-radius: 1px;
    flex-shrink: 0;
    align-self: stretch;
  }
  .change-marker.critical { background: var(--coral); }
  .change-marker.elevated { background: var(--amber); }
  .change-marker.active   { background: var(--teal); }
  .change-marker.resolved { background: var(--lime); }

  .change-body { flex: 1; min-width: 0; }

  .change-meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 3px;
  }

  .change-type {
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--text-tertiary);
  }

  .change-time {
    font-family: var(--font-mono);
    font-size: 9px;
    color: var(--text-tertiary);
    letter-spacing: 0.04em;
  }

  .change-title {
    font-size: 12px;
    line-height: 1.4;
    color: var(--text-secondary);
  }

  .change-title strong { color: var(--text-primary); font-weight: 500; }
  .change-title strong.crit { color: var(--coral); }
  .change-title strong.warn { color: var(--amber); }

  /* Narrative block */
  .narrative {
    background: var(--bg-elevated);
    border-left: 2px solid var(--teal);
    padding: 10px 12px;
    border-radius: 0 2px 2px 0;
  }

  .narrative-text {
    font-size: 12px;
    line-height: 1.6;
    color: var(--text-secondary);
  }

  .narrative-text em { font-style: normal; color: var(--text-primary); }
  .narrative-text em.crit { color: var(--coral); }
  .narrative-text em.warn { color: var(--amber); }

  .narrative-meta {
    display: flex;
    gap: 12px;
    margin-top: 8px;
    font-family: var(--font-mono);
    font-size: 9.5px;
    color: var(--text-tertiary);
    letter-spacing: 0.1em;
  }

  /* Source health */
  .source-grid {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .source-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .source-name {
    font-family: var(--font-mono);
    font-size: 9.5px;
    letter-spacing: 0.1em;
    color: var(--text-tertiary);
    text-transform: uppercase;
    min-width: 90px;
    flex-shrink: 0;
  }

  .source-bar {
    flex: 1;
    height: 3px;
    background: var(--bg-deep);
    border-radius: 1px;
    overflow: hidden;
  }

  .source-bar-fill {
    height: 100%;
    border-radius: 1px;
    transition: width 0.6s ease;
  }
  .source-bar-fill.healthy  { background: var(--lime); }
  .source-bar-fill.degraded { background: var(--amber); }
  .source-bar-fill.failing  { background: var(--coral); }

  .source-value {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    min-width: 24px;
    text-align: right;
  }

  /* ============================================================
     CENTER — HERO OPERATIONAL SURFACE
     ============================================================ */
  .center {
    grid-area: center;
    position: relative;
    overflow: hidden;
    background: var(--bg-deep);
  }

  #cesium-globe {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }
  /* Hide Cesium toolbar chrome */
  .cesium-viewer-toolbar,
  .cesium-viewer-animationContainer,
  .cesium-viewer-timelineContainer,
  .cesium-viewer-bottom,
  .cesium-viewer-fullscreenContainer,
  .cesium-viewer-geocoderContainer { display: none !important; }
  .cesium-widget-credits { display: none !important; }
  /* Darken Cesium background to match theme */
  .cesium-viewer canvas { background: var(--bg-deep) !important; }

  /* Crosshair */
  .crosshair {
    position: absolute;
    inset: 0;
    pointer-events: none;
    opacity: 0.15;
  }

  /* Scanlines overlay */
  .scanlines {
    position: absolute;
    inset: 0;
    pointer-events: none;
    background: repeating-linear-gradient(
      0deg,
      transparent,
      transparent 2px,
      rgba(0, 0, 0, 0.08) 2px,
      rgba(0, 0, 0, 0.08) 4px
    );
    z-index: 1;
  }

  /* Vignette */
  .vignette {
    position: absolute;
    inset: 0;
    pointer-events: none;
    background: radial-gradient(ellipse at center, transparent 40%, rgba(7, 17, 27, 0.85) 100%);
    z-index: 1;
  }

  /* Operational Picture — bottom left overlay */
  .op-picture {
    position: absolute;
    bottom: 20px;
    left: 20px;
    background: var(--bg-panel);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border: 1px solid var(--line-strong);
    border-radius: 2px;
    padding: 14px 18px;
    min-width: 240px;
    z-index: 2;
  }

  .op-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
    font-family: var(--font-mono);
    font-size: 9.5px;
    letter-spacing: 0.22em;
    text-transform: uppercase;
    color: var(--text-tertiary);
  }

  .op-stats {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0;
  }

  .op-stats > div {
    padding: 0 10px 0 0;
    border-right: 1px solid var(--line);
    margin-right: 10px;
  }
  .op-stats > div:last-child {
    border-right: none;
    margin-right: 0;
    padding-right: 0;
  }

  .op-stat-value {
    font-family: var(--font-display);
    font-size: 24px;
    font-weight: 500;
    line-height: 1;
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }

  .op-stat-value.teal  { color: var(--teal); }
  .op-stat-value.amber { color: var(--amber); }
  .op-stat-value.coral { color: var(--coral); }

  .op-stat-label {
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--text-tertiary);
    margin-top: 5px;
  }

  .op-message {
    margin-top: 12px;
    font-size: 12px;
    line-height: 1.5;
    color: var(--text-secondary);
  }

  .op-message strong {
    color: var(--amber);
    font-weight: 500;
  }

  /* Legend — top right overlay */
  .legend {
    position: absolute;
    top: 20px;
    right: 20px;
    background: var(--bg-panel);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border: 1px solid var(--line-strong);
    border-radius: 2px;
    padding: 14px 16px;
    min-width: 190px;
    z-index: 2;
  }

  .legend-header {
    font-family: var(--font-mono);
    font-size: 9.5px;
    letter-spacing: 0.22em;
    text-transform: uppercase;
    color: var(--text-tertiary);
    margin-bottom: 10px;
  }

  .legend-row {
    display: flex;
    align-items: center;
    gap: 9px;
    font-family: var(--font-mono);
    font-size: 10.5px;
    color: var(--text-secondary);
    padding: 5px 0;
    cursor: pointer;
    transition: color 0.15s ease;
  }

  .legend-row:hover { color: var(--text-primary); }

  .legend-dot {
    width: 8px; height: 8px;
    border-radius: 50%;
  }

  .legend-dot.coral  { background: var(--coral); box-shadow: 0 0 6px var(--coral); }
  .legend-dot.amber  { background: var(--amber); box-shadow: 0 0 5px var(--amber); }
  .legend-dot.teal   { background: var(--teal);  box-shadow: 0 0 5px var(--teal); }
  .legend-dot.lime   { background: var(--lime);  box-shadow: 0 0 5px var(--lime); }

  .legend-toggle {
    margin-left: auto;
    width: 20px; height: 10px;
    background: var(--bg-deep);
    border-radius: 5px;
    position: relative;
    border: 1px solid var(--line);
  }

  .legend-toggle.on { background: var(--teal-dim); border-color: var(--teal); }
  .legend-toggle::after {
    content: '';
    position: absolute;
    width: 6px; height: 6px;
    border-radius: 50%;
    background: var(--text-tertiary);
    top: 1px; left: 1px;
    transition: all 0.15s ease;
  }
  .legend-toggle.on::after {
    background: var(--teal);
    left: 11px;
    box-shadow: 0 0 4px var(--teal);
  }

  /* Corner bracket frames */
  .corner-frame {
    position: absolute;
    width: 24px; height: 24px;
    border-color: var(--text-tertiary);
    opacity: 0.4;
    pointer-events: none;
    z-index: 2;
  }
  .corner-frame.tl { top: 12px; left: 12px; border-top: 1px solid; border-left: 1px solid; }
  .corner-frame.tr { top: 12px; right: 12px; border-top: 1px solid; border-right: 1px solid; }
  .corner-frame.bl { bottom: 12px; left: 12px; border-bottom: 1px solid; border-left: 1px solid; }
  .corner-frame.br { bottom: 12px; right: 12px; border-bottom: 1px solid; border-right: 1px solid; }

  /* Coordinates badge */
  .coord-badge {
    position: absolute;
    top: 50%; left: 20px;
    transform: translateY(-50%);
    font-family: var(--font-mono);
    font-size: 9.5px;
    color: var(--text-tertiary);
    letter-spacing: 0.1em;
    writing-mode: vertical-rl;
    text-orientation: mixed;
    opacity: 0.5;
    z-index: 2;
  }

  /* Telemetry ticker */
  .telemetry {
    position: absolute;
    top: 20px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    gap: 24px;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-tertiary);
    letter-spacing: 0.12em;
    background: var(--bg-panel);
    backdrop-filter: blur(8px);
    padding: 8px 16px;
    border: 1px solid var(--line);
    border-radius: 2px;
    z-index: 2;
  }

  .tel-item { display: flex; gap: 6px; }
  .tel-item span:last-child { color: var(--text-secondary); }

  /* ============================================================
     RIGHT DECISION RAIL
     ============================================================ */
  .right-rail {
    grid-area: right;
    background: var(--bg-surface);
    border-left: 1px solid var(--line);
    overflow-y: auto;
  }

  /* Attention queue */
  .attention-item {
    padding: 13px 0;
    border-bottom: 1px solid var(--line);
    cursor: pointer;
    position: relative;
    display: flex;
    gap: 11px;
    transition: transform 0.15s ease;
  }
  .attention-item:last-child { border-bottom: none; }
  .attention-item:hover { transform: translateX(2px); }

  .priority-number {
    font-family: var(--font-display);
    font-weight: 600;
    font-size: 18px;
    color: var(--text-tertiary);
    min-width: 22px;
    line-height: 1;
    padding-top: 1px;
    letter-spacing: -0.02em;
  }

  .priority-number.p1 { color: var(--coral); }
  .priority-number.p2 { color: var(--amber); }
  .priority-number.p3 { color: var(--teal); }

  .att-body { flex: 1; min-width: 0; }

  .att-head {
    display: flex;
    align-items: center;
    gap: 7px;
    margin-bottom: 4px;
  }

  .att-kind {
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    padding: 2px 6px;
    border-radius: 2px;
  }

  .att-kind.focus    { background: var(--coral-dim); color: var(--coral); }
  .att-kind.region   { background: var(--amber-dim); color: var(--amber); }
  .att-kind.evidence { background: var(--teal-dim);  color: var(--teal); }
  .att-kind.replay   { background: rgba(155, 140, 255, 0.15); color: var(--violet); }

  .att-title {
    font-size: 12.5px;
    color: var(--text-primary);
    line-height: 1.35;
    margin-bottom: 4px;
    font-weight: 400;
  }

  .att-reason {
    font-size: 11px;
    color: var(--text-secondary);
    line-height: 1.4;
    margin-bottom: 7px;
  }

  .att-action {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-family: var(--font-mono);
    font-size: 9.5px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--teal);
    padding: 4px 9px;
    border: 1px solid rgba(79, 224, 208, 0.3);
    border-radius: 2px;
    background: transparent;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .att-action:hover {
    background: var(--teal-dim);
    border-color: var(--teal);
  }

  .att-action.warn {
    color: var(--amber);
    border-color: rgba(241, 201, 107, 0.3);
  }
  .att-action.warn:hover {
    background: var(--amber-dim);
    border-color: var(--amber);
  }
  .att-action.subtle {
    color: var(--text-secondary);
    border-color: var(--line-strong);
  }
  .att-action.subtle:hover {
    color: var(--text-primary);
    background: var(--bg-elevated);
    border-color: var(--text-secondary);
  }

  /* Risk trend mini chart */
  .chart-container {
    padding: 14px 0 4px;
  }
  .chart-context {
    margin-top: 10px;
    font-size: 11px;
    line-height: 1.55;
    color: var(--text-secondary);
  }

  .chart-title {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    margin-bottom: 10px;
  }

  .chart-metric {
    font-family: var(--font-display);
    font-size: 22px;
    font-weight: 500;
    line-height: 1;
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }

  .chart-metric .unit {
    font-size: 11px;
    color: var(--text-tertiary);
    margin-left: 4px;
    font-weight: 400;
  }

  .chart-delta {
    font-family: var(--font-mono);
    font-size: 10.5px;
    color: var(--coral);
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .chart-svg {
    width: 100%;
    height: 64px;
  }

  .chart-x-labels {
    display: flex;
    justify-content: space-between;
    font-family: var(--font-mono);
    font-size: 9px;
    color: var(--text-tertiary);
    letter-spacing: 0.06em;
    margin-top: 4px;
  }

  /* ============================================================
     BOTTOM TEMPORAL STRIP
     ============================================================ */
  .bottom-bar {
    grid-area: bottom;
    background: var(--bg-surface);
    border-top: 1px solid var(--line);
    display: grid;
    grid-template-columns: 160px 1fr 160px;
    position: relative;
  }

  .bottom-bar::before {
    content: '';
    position: absolute;
    left: 0; right: 0; top: -1px;
    height: 1px;
    background: linear-gradient(90deg, transparent, var(--line-strong) 20%, var(--line-strong) 80%, transparent);
  }

  .time-label {
    padding: 14px 18px;
    border-right: 1px solid var(--line);
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 5px;
  }

  .time-label.right {
    border-right: none;
    border-left: 1px solid var(--line);
    text-align: right;
  }

  .time-label-kicker {
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 0.22em;
    text-transform: uppercase;
    color: var(--text-tertiary);
  }

  .time-label-time {
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--text-primary);
    letter-spacing: 0.04em;
  }

  .time-label-date {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
  }

  .timeline-container {
    position: relative;
    padding: 12px 16px;
  }
  .timeline-summary {
    position: absolute;
    left: 16px;
    top: -2px;
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--text-tertiary);
  }
  .drawer {
    position: fixed;
    top: 60px;
    right: 0;
    width: min(520px, 92vw);
    height: calc(100vh - 60px);
    background: var(--bg-panel);
    backdrop-filter: blur(18px);
    -webkit-backdrop-filter: blur(18px);
    border-left: 1px solid var(--line-strong);
    transform: translateX(100%);
    transition: transform 0.22s ease;
    z-index: 30;
    display: flex;
    flex-direction: column;
  }
  .drawer.visible { transform: translateX(0); }
  .drawer-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 18px;
    border-bottom: 1px solid var(--line);
  }
  .drawer-title {
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: var(--text-secondary);
  }
  .drawer-close {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 18px;
    cursor: pointer;
  }
  .drawer-path {
    padding: 0 18px 12px;
    font-family: var(--font-mono);
    font-size: 9px;
    color: var(--text-tertiary);
    letter-spacing: 0.08em;
    word-break: break-all;
  }
  .drawer-body {
    flex: 1;
    overflow: auto;
    padding: 14px 18px 20px;
  }
  .drawer-pre {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
    font-family: var(--font-mono);
    font-size: 10px;
    line-height: 1.6;
    color: var(--text-secondary);
  }

  .timeline-track {
    position: absolute;
    left: 16px; right: 16px;
    top: 50%;
    height: 1px;
    background: var(--line);
  }

  .now-marker {
    position: absolute;
    top: 8px; bottom: 8px;
    width: 1px;
    background: var(--teal);
    box-shadow: 0 0 8px var(--teal);
    z-index: 3;
  }

  .now-marker::before {
    content: 'NOW';
    position: absolute;
    top: -2px;
    left: 50%;
    transform: translateX(-50%);
    font-family: var(--font-mono);
    font-size: 8.5px;
    letter-spacing: 0.2em;
    color: var(--teal);
    background: var(--bg-surface);
    padding: 0 4px;
  }

  .now-marker::after {
    content: '';
    position: absolute;
    top: 50%; left: 50%;
    transform: translate(-50%, -50%);
    width: 8px; height: 8px;
    border-radius: 50%;
    background: var(--teal);
    box-shadow: 0 0 12px var(--teal);
    animation: pulse 2s infinite ease-in-out;
  }

  .future-window {
    position: absolute;
    top: 10px; bottom: 10px;
    background: linear-gradient(90deg, rgba(79,224,208,0.08), rgba(79,224,208,0.02));
    border-left: 1px solid rgba(79,224,208,0.3);
    border-right: 1px solid rgba(79,224,208,0.1);
    z-index: 1;
  }

  .timeline-event {
    position: absolute;
    top: 50%;
    transform: translate(-50%, -50%);
    width: 10px; height: 10px;
    border-radius: 50%;
    z-index: 2;
    cursor: pointer;
    transition: transform 0.15s ease;
  }

  .timeline-event:hover { transform: translate(-50%, -50%) scale(1.5); }

  .timeline-event.critical { background: var(--coral); box-shadow: 0 0 10px var(--coral); }
  .timeline-event.elevated { background: var(--amber); box-shadow: 0 0 8px var(--amber); }
  .timeline-event.active   { background: var(--teal);  box-shadow: 0 0 6px var(--teal); }
  .timeline-event.resolved { background: var(--lime); }
  .timeline-event.forecast {
    background: transparent;
    border: 1px solid var(--teal);
    width: 8px; height: 8px;
  }

  .timeline-event-label {
    position: absolute;
    top: calc(100% + 6px);
    left: 50%;
    transform: translateX(-50%);
    font-family: var(--font-mono);
    font-size: 9px;
    color: var(--text-secondary);
    white-space: nowrap;
    letter-spacing: 0.06em;
    opacity: 0;
    transition: opacity 0.15s;
    pointer-events: none;
  }

  .timeline-event:hover .timeline-event-label { opacity: 1; }

  .timeline-tick {
    position: absolute;
    top: 50%;
    width: 1px; height: 4px;
    background: var(--text-tertiary);
    transform: translate(-50%, -50%);
    opacity: 0.4;
  }

  .timeline-tick-label {
    position: absolute;
    top: calc(50% + 10px);
    transform: translateX(-50%);
    font-family: var(--font-mono);
    font-size: 8.5px;
    color: var(--text-tertiary);
    letter-spacing: 0.08em;
  }

  /* Scrollbars */
  ::-webkit-scrollbar { width: 6px; height: 6px; }
  ::-webkit-scrollbar-track { background: transparent; }
  ::-webkit-scrollbar-thumb { background: var(--line-strong); border-radius: 3px; }
  ::-webkit-scrollbar-thumb:hover { background: var(--text-tertiary); }

  /* Shimmer loading state */
  .loading-shimmer {
    height: 10px;
    background: linear-gradient(90deg, var(--bg-elevated) 25%, var(--bg-surface) 50%, var(--bg-elevated) 75%);
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
    border-radius: 2px;
    margin: 4px 0;
  }
  @keyframes shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }

  /* ============================================================
     FOCUS PANEL
     ============================================================ */
  .focus-panel {
    position: absolute;
    top: 80px; left: 50%;
    transform: translateX(-50%) translateY(-12px);
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.22s ease, transform 0.22s cubic-bezier(0.2, 0.8, 0.2, 1);
    background: var(--bg-panel);
    backdrop-filter: blur(18px);
    -webkit-backdrop-filter: blur(18px);
    border: 1px solid var(--line-strong);
    border-radius: 4px;
    padding: 16px 20px;
    min-width: 300px;
    max-width: 420px;
    z-index: 5;
  }
  .focus-panel.visible {
    opacity: 1;
    transform: translateX(-50%) translateY(0);
    pointer-events: auto;
  }
  .focus-panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 10px;
  }
  .focus-kind-badge {
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--teal);
    background: var(--teal-dim);
    padding: 3px 8px;
    border-radius: 2px;
  }
  .focus-kind-badge.region { color: var(--amber); background: var(--amber-dim); }
  .focus-kind-badge.event  { color: var(--coral); background: var(--coral-dim); }
  .focus-close {
    background: none;
    border: none;
    color: var(--text-tertiary);
    font-size: 20px;
    cursor: pointer;
    line-height: 1;
    padding: 0 4px;
    transition: color 0.15s;
  }
  .focus-close:hover { color: var(--text-primary); }
  .focus-ftitle {
    font-size: 15px;
    font-weight: 500;
    color: var(--text-primary);
    margin-bottom: 7px;
    line-height: 1.35;
  }
  .focus-reason {
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.55;
    margin-bottom: 10px;
  }
  .focus-coords {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-tertiary);
    letter-spacing: 0.1em;
    margin-bottom: 12px;
  }
  .focus-detail {
    font-family: var(--font-mono);
    font-size: 9.5px;
    color: var(--text-tertiary);
    letter-spacing: 0.08em;
    line-height: 1.6;
    text-transform: uppercase;
    margin-bottom: 10px;
  }

  .focus-review {
    font-family: var(--font-mono);
    font-size: 9.5px;
    color: var(--text-secondary);
    line-height: 1.7;
    margin-top: 10px;
    white-space: pre-line;
  }
  .op-review-strip {
    margin-top: 10px;
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .op-review-pill {
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-secondary);
    background: rgba(255,255,255,0.04);
    border: 1px solid var(--line);
    border-radius: 999px;
    padding: 5px 8px;
  }
  .op-review-pill.pending {
    color: var(--amber);
    border-color: rgba(241, 201, 107, 0.28);
  }
  .op-review-pill.applied {
    color: var(--teal);
    border-color: rgba(79, 224, 208, 0.28);
  }
  .op-review-pill.dismissed {
    color: var(--coral);
    border-color: rgba(255, 124, 110, 0.28);
  }
  .focus-divider { border: none; border-top: 1px solid var(--line); margin: 10px 0; }
  .focus-actions { display: flex; gap: 8px; flex-wrap: wrap; }
  .focus-links {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    margin-top: 8px;
  }
  .focus-state {
    margin-top: 8px;
    font-family: var(--font-mono);
    font-size: 9.5px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-tertiary);
  }

  /* Load animation — staggered reveals */
  .console > * {
    opacity: 0;
    animation: rise 0.6s cubic-bezier(0.2, 0.8, 0.2, 1) forwards;
  }
  .top-bar    { animation-delay: 0.0s; }
  .left-rail  { animation-delay: 0.15s; }
  .center     { animation-delay: 0.08s; }
  .right-rail { animation-delay: 0.2s; }
  .bottom-bar { animation-delay: 0.25s; }

  @keyframes rise {
    from { opacity: 0; transform: translateY(6px); }
    to   { opacity: 1; transform: translateY(0); }
  }

  .panel { opacity: 0; animation: rise 0.5s cubic-bezier(0.2, 0.8, 0.2, 1) forwards; }
  .panel:nth-child(1) { animation-delay: 0.3s; }
  .panel:nth-child(2) { animation-delay: 0.4s; }
  .panel:nth-child(3) { animation-delay: 0.5s; }
  .panel:nth-child(4) { animation-delay: 0.6s; }
  .panel:nth-child(5) { animation-delay: 0.7s; }

  /* ============================================================
     MOBILE LAYOUT — Globe visible, rails stacked below
     ============================================================ */
  @media (max-width: 860px) {
    html, body { overflow-y: auto; overflow-x: hidden; }

    .console {
      display: flex;
      flex-direction: column;
      height: auto;
      min-height: 100vh;
    }

    .top-bar {
      position: sticky;
      top: 0;
      z-index: 100;
      width: 100%;
      flex-shrink: 0;
      /* Collapse search bar on small screens */
    }

    /* Hide some desktop-only top bar elements to save space */
    .search-wrap { display: none; }

    .center {
      width: 100%;
      height: 45vh;
      min-height: 260px;
      flex-shrink: 0;
      order: 1;
    }

    /* Simplify overlays on mobile */
    .crosshair, .scanlines, .corner-frame { display: none; }
    .telemetry { display: none; }
    .coord-badge { font-size: 9px; padding: 4px 8px; bottom: 4px; left: 4px; }

    /* Op picture and legend: stack inside globe panel */
    .op-picture { bottom: 4px; left: 4px; right: 4px; max-width: none; font-size: 11px; }
    .op-stats { gap: 12px; }
    .legend { bottom: auto; top: 4px; right: 4px; left: auto; }

    .left-rail {
      grid-area: unset;
      width: 100%;
      height: auto;
      max-height: none;
      overflow-y: visible;
      border-right: none;
      border-top: 1px solid var(--line);
      order: 2;
      flex-shrink: 0;
    }

    .right-rail {
      grid-area: unset;
      width: 100%;
      height: auto;
      max-height: none;
      overflow-y: visible;
      border-left: none;
      border-top: 1px solid var(--line);
      order: 3;
      flex-shrink: 0;
    }

    .bottom-bar {
      grid-area: unset;
      width: 100%;
      height: auto;
      order: 4;
      flex-wrap: wrap;
      gap: 8px;
      padding: 12px 16px;
      flex-shrink: 0;
    }

    /* Ensure panels don't clip content */
    .panel { flex-shrink: 0; }

    /* Focus panel: full width on mobile */
    .focus-panel {
      left: 8px;
      right: 8px;
      bottom: 8px;
      top: auto;
      max-width: none;
    }
  }

  @media (max-width: 480px) {
    .center { height: 38vh; min-height: 220px; }
    .brand-name { display: none; }
    .op-stat-value { font-size: 18px; }
  }
</style>
</head>
<body>

<div class="console">

  <!-- ============================================================
       TOP COMMAND BAR
       ============================================================ -->
  <header class="top-bar">
    <div class="brand">
      <div class="brand-mark">
        <svg viewBox="0 0 32 32" fill="none">
          <defs>
            <linearGradient id="g1" x1="0" y1="0" x2="32" y2="32">
              <stop offset="0" stop-color="#4FE0D0"/>
              <stop offset="1" stop-color="#9B8CFF"/>
            </linearGradient>
          </defs>
          <path d="M16 2 L28 9 L28 23 L16 30 L4 23 L4 9 Z" stroke="url(#g1)" stroke-width="1.5" fill="none"/>
          <path d="M16 8 L22 11.5 L22 20.5 L16 24 L10 20.5 L10 11.5 Z" stroke="#4FE0D0" stroke-width="1" fill="rgba(79,224,208,0.1)"/>
          <circle cx="16" cy="16" r="2" fill="#4FE0D0"/>
        </svg>
      </div>
      <div class="brand-text">
        <div class="brand-name">ShieldSky</div>
        <div class="brand-context">Operator Console</div>
      </div>
    </div>

    <div class="chip-group" style="border-left: none; padding-left: 0;">
      <button class="chip active" data-filter="global">Global</button>
      <button class="chip" data-filter="regional">Regional</button>
      <button class="chip" data-filter="site">Site</button>
      <button class="chip" data-filter="incident">Incident</button>
    </div>

    <div class="chip-group">
      <button class="chip active" data-review-state="all">All States</button>
      <button class="chip" data-review-state="pending_review">Pending</button>
      <button class="chip" data-review-state="applied">Applied</button>
      <button class="chip" data-review-state="dismissed">Dismissed</button>
    </div>

    <div class="chip-group">
      <button class="chip" data-window="24h">24h</button>
      <button class="chip active" data-window="72h">72h</button>
      <button class="chip" data-window="7d">7d</button>
      <button class="chip" data-window="30d">30d</button>
    </div>

    <div class="search">
      <svg class="search-icon" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="7"/>
        <path d="m20 20-3.5-3.5"/>
      </svg>
      <input type="text" placeholder="Search sites &middot; regions &middot; events &middot; bundles" id="console-search"/>
      <kbd class="search-kbd">&#8984;K</kbd>
    </div>

    <div class="sys-status">
      <div class="sys-item">
        <span class="pulse-dot" id="sys-pulse"></span>
        <span id="sys-sources">connecting&hellip;</span>
      </div>
      <div class="sys-item">
        <span style="color: var(--text-tertiary)">updated</span>
        <span id="sys-updated" style="color: var(--lime)">&#8212;</span>
      </div>
    </div>

    <div class="operator" title="Operator">OP</div>
  </header>

  <!-- ============================================================
       LEFT — INTELLIGENCE RAIL
       ============================================================ -->
  <aside class="left-rail">

    <!-- WHAT CHANGED -->
    <div class="panel">
      <div class="panel-header">
        <div class="panel-title">What Changed</div>
        <div class="panel-count" id="changes-count">&#8212;</div>
      </div>
      <div id="what-changed-list">
        <div class="change-item">
          <div class="change-marker active"></div>
          <div class="change-body">
            <div class="change-meta"><span class="change-type">SYSTEM</span><span class="change-time">now</span></div>
            <div class="change-title" style="color:var(--text-tertiary)">Connecting to intelligence feeds&hellip;</div>
          </div>
        </div>
      </div>
    </div>

    <!-- NARRATIVE -->
    <div class="panel">
      <div class="panel-header">
        <div class="panel-title violet">Narrative</div>
        <div class="panel-count" id="narrative-version">auto</div>
      </div>
      <div class="narrative">
        <div class="narrative-text" id="narrative-text" style="color:var(--text-tertiary)">
          Loading operational narrative&hellip;
        </div>
        <div class="narrative-meta" id="narrative-meta"></div>
      </div>
    </div>

    <!-- SOURCE HEALTH -->
    <div class="panel">
      <div class="panel-header">
        <div class="panel-title amber">Source Health</div>
        <div class="panel-count" id="source-count">&#8212;</div>
      </div>
      <div class="source-grid" id="source-health-list">
        <div class="loading-shimmer" style="width:100%;margin:6px 0"></div>
        <div class="loading-shimmer" style="width:85%;margin:6px 0"></div>
        <div class="loading-shimmer" style="width:92%;margin:6px 0"></div>
      </div>
    </div>

    <!-- PROVENANCE -->
    <div class="panel">
      <div class="panel-header">
        <div class="panel-title">Provenance</div>
        <div class="panel-count">latest</div>
      </div>
      <div id="provenance-list" style="display:flex;flex-direction:column;gap:8px">
        <div style="font-family:var(--font-mono);font-size:10px;color:var(--text-tertiary);padding:8px 0;">
          No ingest batches yet.
        </div>
      </div>
    </div>

  </aside>

  <!-- ============================================================
       CENTER — HERO OPERATIONAL SURFACE
       ============================================================ -->
  <main class="center">
    <div id="cesium-globe"></div>

    <svg class="crosshair" width="100%" height="100%">
      <line x1="50%" y1="0" x2="50%" y2="100%" stroke="white" stroke-width="0.5" stroke-dasharray="2 4"/>
      <line x1="0" y1="50%" x2="100%" y2="50%" stroke="white" stroke-width="0.5" stroke-dasharray="2 4"/>
    </svg>

    <div class="scanlines"></div>
    <div class="vignette"></div>

    <div class="corner-frame tl"></div>
    <div class="corner-frame tr"></div>
    <div class="corner-frame bl"></div>
    <div class="corner-frame br"></div>

    <div class="coord-badge" id="coord-badge">38.71&deg;N &middot; 9.14&deg;W &middot; IBERIAN WATCH</div>

    <!-- Telemetry ticker -->
    <div class="telemetry">
      <div class="tel-item"><span>LAT</span><span id="tel-lat">38.7139&deg;N</span></div>
      <div class="tel-item"><span>LON</span><span id="tel-lon">-009.1400&deg;</span></div>
      <div class="tel-item"><span>ALT</span><span id="fps">1,800 km</span></div>
    </div>

    <!-- Operational Picture -->
    <div class="op-picture">
      <div class="op-header">
        <span>Operational Picture</span>
        <span style="color: var(--teal)" id="live-indicator">&#9679; LIVE</span>
      </div>
      <div class="op-stats">
        <div>
          <div class="op-stat-value amber" id="op-regions">&#8212;</div>
          <div class="op-stat-label">Regions Active</div>
        </div>
        <div>
          <div class="op-stat-value coral" id="op-sites">&#8212;</div>
          <div class="op-stat-label">Sites Elevated</div>
        </div>
        <div>
          <div class="op-stat-value teal" id="op-events">&#8212;</div>
          <div class="op-stat-label">Events Live</div>
        </div>
      </div>
      <div class="op-message" id="op-message">
        Connecting to operational feeds&hellip;
      </div>
      <div class="op-review-strip" id="op-review-strip"></div>
    </div>

    <!-- Legend -->
    <div class="legend">
      <div class="legend-header">Layers</div>
      <div class="legend-row">
        <span class="legend-dot coral"></span>
        <span>Critical</span>
        <div class="legend-toggle on" data-layer="critical"></div>
      </div>
      <div class="legend-row">
        <span class="legend-dot amber"></span>
        <span>Elevated</span>
        <div class="legend-toggle on" data-layer="elevated"></div>
      </div>
      <div class="legend-row">
        <span class="legend-dot teal"></span>
        <span>Active</span>
        <div class="legend-toggle on" data-layer="active"></div>
      </div>
      <div class="legend-row">
        <span class="legend-dot lime"></span>
        <span>Healthy</span>
        <div class="legend-toggle on" data-layer="healthy"></div>
      </div>
      <div class="legend-row" style="margin-top: 4px; padding-top: 8px; border-top: 1px solid var(--line)">
        <span style="width: 8px; height: 8px; border: 1px solid var(--teal); display:inline-block"></span>
        <span>Forecast</span>
        <div class="legend-toggle" data-layer="forecast"></div>
      </div>
      <div class="legend-row">
        <span style="width: 8px; height: 2px; background: var(--violet); display:inline-block"></span>
        <span>Orbital</span>
        <div class="legend-toggle on" data-layer="orbital"></div>
      </div>
    </div>

    <!-- Focus Panel — appears when site/region selected -->
    <div id="focus-panel" class="focus-panel">
      <div class="focus-panel-header">
        <span class="focus-kind-badge" id="focus-kind">SITE</span>
        <button class="focus-close" id="focus-close-btn">&#215;</button>
      </div>
      <div class="focus-ftitle" id="focus-title">&mdash;</div>
      <div class="focus-reason" id="focus-reason"></div>
      <div class="focus-coords" id="focus-coords"></div>
      <div class="focus-detail" id="focus-detail"></div>
      <hr class="focus-divider">
      <div class="focus-actions">
        <button class="att-action" id="focus-primary-btn">Open &rarr;</button>
      </div>
      <div class="focus-links" id="focus-links"></div>
      <div class="focus-state" id="focus-state"></div>
      <div class="focus-review" id="focus-review"></div>
    </div>

  </main>

  <!-- ============================================================
       RIGHT — DECISION RAIL
       ============================================================ -->
  <aside class="right-rail">

    <!-- ATTENTION QUEUE -->
    <div class="panel">
      <div class="panel-header">
        <div class="panel-title coral">Attention Queue</div>
        <div class="panel-count" id="attention-count">&#8212;</div>
      </div>
      <div id="attention-queue-list">
        <div class="loading-shimmer" style="width:100%;margin:6px 0;height:14px"></div>
        <div class="loading-shimmer" style="width:90%;margin:6px 0;height:14px"></div>
        <div class="loading-shimmer" style="width:95%;margin:6px 0;height:14px"></div>
      </div>
    </div>

    <!-- RECOMMENDED ACTIONS -->
    <div class="panel">
      <div class="panel-header">
        <div class="panel-title violet">Recommended Actions</div>
        <div class="panel-count">impact-sorted</div>
      </div>
      <div id="recommended-actions-list" style="display:flex;flex-direction:column;gap:8px">
        <div style="font-family:var(--font-mono);font-size:10px;color:var(--text-tertiary);padding:6px 0;">
          Loading recommendations&hellip;
        </div>
      </div>
    </div>

    <!-- RISK TREND CHART -->
    <div class="panel">
      <div class="panel-header">
        <div class="panel-title" id="risk-trend-title">Region Risk</div>
        <div class="panel-count" id="risk-trend-window">72h</div>
      </div>
      <div class="chart-container">
        <div class="chart-title">
          <div>
            <div class="chart-metric"><span id="risk-metric">&#8212;</span><span class="unit">/ 1.0</span></div>
            <div style="font-family: var(--font-mono); font-size: 9.5px; color: var(--text-tertiary); letter-spacing: 0.1em; margin-top: 3px; text-transform: uppercase;">Pressure Index</div>
          </div>
          <div class="chart-delta" id="risk-delta">
            <svg width="10" height="10" viewBox="0 0 10 10"><path d="M 5 1 L 9 7 L 1 7 Z" fill="currentColor"/></svg>
            &#8212;
          </div>
        </div>
        <svg class="chart-svg" viewBox="0 0 300 64" preserveAspectRatio="none">
          <defs>
            <linearGradient id="riskGrad" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0" stop-color="#FF7C6E" stop-opacity="0.45"/>
              <stop offset="1" stop-color="#FF7C6E" stop-opacity="0"/>
            </linearGradient>
          </defs>
          <rect x="0" y="0" width="300" height="16" fill="rgba(255,124,110,0.05)"/>
          <rect x="0" y="16" width="300" height="24" fill="rgba(241,201,107,0.05)"/>
          <line x1="0" y1="16" x2="300" y2="16" stroke="rgba(160,190,220,0.1)" stroke-dasharray="2 3"/>
          <line x1="0" y1="40" x2="300" y2="40" stroke="rgba(160,190,220,0.1)" stroke-dasharray="2 3"/>
          <path id="risk-area" d="M 0 55 L 300 55 L 300 64 L 0 64 Z" fill="url(#riskGrad)"/>
          <path id="risk-line" d="M 0 55 L 300 55" fill="none" stroke="#FF7C6E" stroke-width="1.5"/>
        </svg>
        <div class="chart-x-labels">
          <span>-72H</span><span>-48H</span><span>-24H</span><span>NOW</span>
        </div>
        <div class="chart-context" id="risk-context">Waiting for focused analytics.</div>
      </div>
    </div>

    <!-- EVENT COMPOSITION -->
    <div class="panel">
      <div class="panel-header">
        <div class="panel-title">Event Composition</div>
        <div class="panel-count" id="event-total">&#8212;</div>
      </div>
      <div id="event-composition-list" style="display:flex;flex-direction:column;gap:9px">
        <div class="loading-shimmer" style="width:100%;margin:4px 0;height:3px"></div>
        <div class="loading-shimmer" style="width:75%;margin:4px 0;height:3px"></div>
        <div class="loading-shimmer" style="width:55%;margin:4px 0;height:3px"></div>
      </div>
    </div>

  </aside>

  <!-- ============================================================
       BOTTOM — TEMPORAL STRIP
       ============================================================ -->
  <footer class="bottom-bar">
    <div class="time-label">
      <span class="time-label-kicker" id="timeline-start-kicker">WINDOW START</span>
      <span class="time-label-time" id="timeline-start-time">&#8212;</span>
      <span class="time-label-date" id="timeline-start-date">&#8212;</span>
    </div>

    <div class="timeline-container" id="timeline">
      <div class="timeline-summary" id="timeline-summary">Event memory and forecast windows will appear here.</div>
      <div class="timeline-track"></div>
      <div class="future-window" id="timeline-future-window"></div>

      <div id="timeline-ticks"></div>

      <!-- Events populated by JS -->
      <div id="timeline-events"></div>

      <div class="now-marker" id="timeline-now-marker"></div>
    </div>

    <div class="time-label right">
      <span class="time-label-kicker" id="timeline-end-kicker">WINDOW END</span>
      <span class="time-label-time" id="timeline-end-time">&#8212;</span>
      <span class="time-label-date" id="timeline-end-date">&#8212;</span>
    </div>
  </footer>

</div>

<div id="data-drawer" class="drawer">
  <div class="drawer-header">
    <div class="drawer-title" id="drawer-title">Operational Detail</div>
    <button class="drawer-close" id="drawer-close-btn">&#215;</button>
  </div>
  <div class="drawer-path" id="drawer-path"></div>
  <div class="drawer-body">
    <pre class="drawer-pre" id="drawer-content">Select evidence, replay, or operational detail to inspect it here.</pre>
  </div>
</div>

<script>
/* ============================================================
   OPERATIONAL GLOBE — Three.js r128
   ============================================================ */

/* ============================================================
   OPERATIONAL GLOBE — CesiumJS 1.127
   Real satellite imagery + OSM tiles, dark theme
   ============================================================ */

// Suppress Cesium's default access token requirement for OSM tiles
Cesium.Ion.defaultAccessToken = '';

const viewer = new Cesium.Viewer('cesium-globe', {
  animation: false,
  timeline: false,
  selectionIndicator: false,
  navigationHelpButton: false,
  homeButton: false,
  sceneModePicker: false,
  baseLayerPicker: false,
  geocoder: false,
  fullscreenButton: false,
  infoBox: false,
  shouldAnimate: true,
  imageryProvider: new Cesium.OpenStreetMapImageryProvider({
    url: 'https://tile.openstreetmap.org/',
    credit: ''
  }),
});

// Dark scene settings
viewer.scene.backgroundColor = Cesium.Color.fromCssColorString('#07111B');
viewer.scene.skyBox.show = true;
viewer.scene.fog.enabled = true;
viewer.scene.fog.density = 0.0002;
viewer.scene.globe.enableLighting = false;
viewer.scene.globe.baseColor = Cesium.Color.fromCssColorString('#07111B');

// Tint OSM tiles darker to match console theme
const imageryLayers = viewer.imageryLayers;
if (imageryLayers.length > 0) {
  const baseLayer = imageryLayers.get(0);
  baseLayer.brightness = 0.5;
  baseLayer.contrast = 1.1;
  baseLayer.saturation = 0.7;
  baseLayer.hue = 0.55;
}

// Initial camera position — Iberian Peninsula
viewer.camera.setView({
  destination: Cesium.Cartesian3.fromDegrees(-9.14, 38.71, 1800000),
  orientation: { heading: 0, pitch: Cesium.Math.toRadians(-45), roll: 0 }
});

// Update telemetry display from camera
function updateCameraTelemetry() {
  const pos = viewer.camera.positionCartographic;
  if (!pos) return;
  const lat = Cesium.Math.toDegrees(pos.latitude).toFixed(4);
  const lon = Cesium.Math.toDegrees(pos.longitude).toFixed(4);
  const alt = Math.round(pos.height / 1000);
  const latEl = document.getElementById('tel-lat');
  const lonEl = document.getElementById('tel-lon');
  const fpsEl = document.getElementById('fps');
  if (latEl) latEl.textContent = lat + '\u00b0N';
  if (lonEl) lonEl.textContent = lon + '\u00b0';
  if (fpsEl) fpsEl.textContent = alt.toLocaleString() + ' km';
}
viewer.camera.changed.addEventListener(updateCameraTelemetry);

// Site colour mapping
const SITE_COLORS = {
  critical: Cesium.Color.fromCssColorString('#FF7C6E'),
  elevated: Cesium.Color.fromCssColorString('#F1C96B'),
  active:   Cesium.Color.fromCssColorString('#4FE0D0'),
  healthy:  Cesium.Color.fromCssColorString('#8EE59B'),
};
const SITE_SIZES = { critical: 14, elevated: 11, active: 9, healthy: 7 };

function addSiteToGlobe(site) {
  const color = SITE_COLORS[site.level] || SITE_COLORS.active;
  const size  = SITE_SIZES[site.level]  || 9;
  viewer.entities.add({
    position: Cesium.Cartesian3.fromDegrees(site.lng, site.lat),
    point: {
      pixelSize: size,
      color: color,
      outlineColor: Cesium.Color.BLACK.withAlpha(0.7),
      outlineWidth: 1,
      heightReference: Cesium.HeightReference.CLAMP_TO_GROUND,
    },
    label: site.label ? {
      text: site.label,
      font: '10px monospace',
      fillColor: Cesium.Color.WHITE.withAlpha(0.85),
      outlineColor: Cesium.Color.BLACK,
      outlineWidth: 2,
      style: Cesium.LabelStyle.FILL_AND_OUTLINE,
      pixelOffset: new Cesium.Cartesian2(0, -18),
      scale: 1.0,
      showBackground: false,
      heightReference: Cesium.HeightReference.CLAMP_TO_GROUND,
    } : undefined,
    properties: { level: site.level, label: site.label },
  });
}

// Globe entity click — open focus panel
const clickHandler = new Cesium.ScreenSpaceEventHandler(viewer.scene.canvas);
clickHandler.setInputAction(function(click) {
  const picked = viewer.scene.pick(click.position);
  if (!picked || !picked.id) return;
  const ent = picked.id;
  const props = ent.properties;
  if (!props) return;
  const pos = ent.position && ent.position.getValue(Cesium.JulianDate.now());
  let lat = null, lng = null;
  if (pos) {
    const carto = Cesium.Ellipsoid.WGS84.cartesianToCartographic(pos);
    lat = Cesium.Math.toDegrees(carto.latitude);
    lng = Cesium.Math.toDegrees(carto.longitude);
  }
  openFocusPanel({
    kind: propValue(props, 'kind') || 'site',
    title: propValue(props, 'label') || ent.name || 'Unknown',
    reason: propValue(props, 'reason') || '',
    lat,
    lng,
    siteId: propValue(props, 'site_id') || '',
    regionId: propValue(props, 'region_id') || '',
    actionPath: propValue(props, 'action_path') || '',
    narrativePath: propValue(props, 'narrative_path') || '',
    operatorState: propValue(props, 'operator_state') || '',
  });
}, Cesium.ScreenSpaceEventType.LEFT_CLICK);

document.getElementById('focus-close-btn').addEventListener('click', () => {
  document.getElementById('focus-panel').classList.remove('visible');
  _focusCoords = { lat: null, lng: null };
  FOCUS_STATE.kind = null;
  FOCUS_STATE.siteId = null;
  FOCUS_STATE.regionId = null;
  FOCUS_STATE.actionPath = null;
  FOCUS_STATE.narrativePath = null;
  FOCUS_STATE.lastRecommendation = null;
  setFocusStateText('');
  const focusReview = document.getElementById('focus-review');
  if (focusReview) focusReview.textContent = '';
  refreshGlobalAnalytics().catch(() => {});
});
document.getElementById('drawer-close-btn').addEventListener('click', closeDrawer);

document.getElementById('focus-primary-btn').addEventListener('click', () => {
  if (FOCUS_STATE.actionPath) {
    openDrawerFromPath('Primary Focus', FOCUS_STATE.actionPath).catch(() => {});
  } else if (_focusCoords.lat != null) {
    focusOnGlobe(_focusCoords.lat, _focusCoords.lng, 180000);
  }
});

// --- Globe data cache + focus system ---
const GLOBE_DATA = { regions: [], sites: [], events: [] };
let lastAttentionItems = [];
let lastRecommendedActions = [];
let _focusCoords = { lat: null, lng: null };
const FOCUS_STATE = {
  kind: null,
  siteId: null,
  regionId: null,
  actionPath: null,
  narrativePath: null,
  evidencePath: null,
  replayPath: null,
  title: null,
  lastRecommendation: null,
};
const PRESSURE_HISTORY = []; // circular buffer { t, score }
const FORECAST_ENTITIES = [];
const RECOMMENDATION_STATE = {};

function focusOnGlobe(lat, lng, altM) {
  viewer.camera.flyTo({
    destination: Cesium.Cartesian3.fromDegrees(lng, lat, altM || 450000),
    orientation: { heading: 0, pitch: Cesium.Math.toRadians(-50), roll: 0 },
    duration: 1.5,
  });
}

function propValue(props, key) {
  return props && props[key] ? props[key].getValue() : null;
}

function severityClassFromScore(score) {
  if (score >= 0.85) return 'critical';
  if (score >= 0.65) return 'elevated';
  if (score >= 0.35) return 'active';
  return 'resolved';
}

function setFocusPrimaryButton(label, path) {
  const btn = document.getElementById('focus-primary-btn');
  if (!btn) return;
  btn.textContent = label || 'Open →';
  btn.disabled = false;
  btn.style.opacity = '1';
  FOCUS_STATE.actionPath = path || null;
}

function setFocusStateText(text) {
  const stateEl = document.getElementById('focus-state');
  if (!stateEl) return;
  stateEl.textContent = text || '';
}

function currentTimelineWindowHours() {
  const mapping = { '24h': 24, '72h': 72, '7d': 24 * 7, '30d': 24 * 30 };
  return mapping[CONSOLE_STATE.window] || 72;
}

function currentTimelineRangeMs() {
  const totalHours = currentTimelineWindowHours();
  const halfWindowMs = (totalHours * 3600 * 1000) / 2;
  const nowMs = Date.now();
  return {
    nowMs: nowMs,
    startMs: nowMs - halfWindowMs,
    endMs: nowMs + halfWindowMs,
    totalMs: totalHours * 3600 * 1000,
    halfHours: totalHours / 2,
  };
}

function renderTimelineScale() {
  const ticksEl = document.getElementById('timeline-ticks');
  const futureWindowEl = document.getElementById('timeline-future-window');
  const nowMarkerEl = document.getElementById('timeline-now-marker');
  const startKickerEl = document.getElementById('timeline-start-kicker');
  const endKickerEl = document.getElementById('timeline-end-kicker');
  if (!ticksEl || !futureWindowEl || !nowMarkerEl) return;

  const range = currentTimelineRangeMs();
  const half = range.halfHours;
  const tickOffsets = [-0.66, -0.33, 0.33, 0.66];

  ticksEl.innerHTML = tickOffsets.map(function(multiplier) {
    const offsetHours = Math.round(half * multiplier);
    const pct = 50 + (multiplier * 50);
    const label = (offsetHours > 0 ? '+' : '') + offsetHours + 'H';
    return '<div class="timeline-tick" style="left:' + pct.toFixed(1) + '%"><div class="timeline-tick-label">' + label + '</div></div>';
  }).join('');

  futureWindowEl.style.left = '50%';
  futureWindowEl.style.right = '0';
  nowMarkerEl.style.left = '50%';

  if (startKickerEl) startKickerEl.textContent = 'WINDOW START · T-' + half + 'h';
  if (endKickerEl) endKickerEl.textContent = 'WINDOW END · T+' + half + 'h';
}

function formatReviewHistoryPayload(payload) {
  if (!Array.isArray(payload)) return null;
  if (!payload.length) return 'No review history recorded yet.';
  return payload.map(function(review, index) {
    const state = humanReviewState(review.state || '');
    const actor = review.actor ? 'ACTOR      ' + review.actor : 'ACTOR      operator';
    const rationale = review.rationale ? 'RATIONALE  ' + review.rationale : 'RATIONALE  none recorded';
    const snapshot = review.snapshot_id ? 'SNAPSHOT   ' + review.snapshot_id : '';
    const evidence = review.evidence_bundle_hash ? 'EVIDENCE   ' + review.evidence_bundle_hash : '';
    const age = review.age_bucket ? 'AGE        ' + String(review.age_bucket).toUpperCase() + ' · ' + tAgo(review.decided_at_unix_seconds) : '';
    return [
      '#' + String(index + 1).padStart(2, '0') + ' · ' + String(state || 'reviewed').toUpperCase(),
      age,
      actor,
      rationale,
      snapshot,
      evidence,
    ].filter(Boolean).join('\n');
  }).join('\n\n');
}

function formatReadinessPayload(payload) {
  if (!payload || typeof payload !== 'object') return null;
  const scheduler = payload.scheduler || {};
  const sources = Array.isArray(payload.sources) ? payload.sources : [];
  const ready = sources.filter(function(source) {
    return String(source.readiness || '').toLowerCase() === 'ready';
  }).length;
  const lines = [
    'SCHEDULER   ' + (scheduler.enabled ? 'ENABLED' : 'DISABLED'),
    scheduler.poll_seconds ? 'POLL        ' + scheduler.poll_seconds + 's' : '',
    scheduler.retry_seconds ? 'RETRY       ' + scheduler.retry_seconds + 's' : '',
    scheduler.window_hours ? 'WINDOW      ' + scheduler.window_hours + 'h' : '',
    'READY       ' + ready + ' / ' + sources.length,
    scheduler.force_discovery ? 'DISCOVERY   forced' : 'DISCOVERY   scheduled',
    scheduler.startup_discovery_enabled ? 'BOOT        startup discovery on' : 'BOOT        startup discovery off',
  ].filter(Boolean);
  if (!sources.length) return lines.join('\n');
  const sourceLines = sources.map(function(source) {
    const name = String(source.label || source.source_id || 'source').toUpperCase();
    const state = String(source.readiness || 'awaiting_data').replace(/_/g, ' ').toUpperCase();
    const reason = source.reason ? ' · ' + source.reason : '';
    const sampleText = source.sample_count ? ' · ' + source.sample_count + ' samples' : '';
    return name + ' · ' + state + sampleText + reason;
  });
  return lines.join('\n') + '\n\n' + sourceLines.join('\n');
}

function formatRegionOverviewPayload(payload) {
  if (!payload || typeof payload !== 'object' || !payload.region) return null;
  const region = payload.region || {};
  const topSites = Array.isArray(payload.top_sites) ? payload.top_sites : [];
  const siteTypes = Array.isArray(region.site_types) ? region.site_types.join(', ') : '';
  const lastDiscovery = region.last_discovered_at_unix_seconds
    ? ('LAST DISCOVERY   ' + tAgo(region.last_discovered_at_unix_seconds))
    : 'LAST DISCOVERY   none yet';
  const lastRun = region.last_scheduler_run_at_unix_seconds
    ? ('LAST RUN         ' + tAgo(region.last_scheduler_run_at_unix_seconds))
    : 'LAST RUN         scheduler has not run yet';
  const discovery = payload.discovery_due
    ? ('DISCOVERY        due · overdue by ' + formatDurationSeconds(payload.discovery_overdue_seconds || 0))
    : 'DISCOVERY        within cadence';
  const coords = [
    region.south != null ? 'S ' + Number(region.south).toFixed(2) : '',
    region.west != null ? 'W ' + Number(region.west).toFixed(2) : '',
    region.north != null ? 'N ' + Number(region.north).toFixed(2) : '',
    region.east != null ? 'E ' + Number(region.east).toFixed(2) : '',
  ].filter(Boolean).join(' · ');
  const topSiteLine = topSites.length
    ? 'TOP SITES        ' + topSites.slice(0, 3).map(function(site) {
        const name = site.site_name || site.name || site.seed_key || 'site';
        const status = site.recent_event_count > 0
          ? (site.recent_event_count + ' recent event' + (site.recent_event_count === 1 ? '' : 's'))
          : String(site.seed_status || 'candidate');
        return name + ' (' + status + ')';
      }).join(' · ')
    : 'TOP SITES        none ranked yet';
  return [
    'REGION           ' + (region.name || payload.region_id || 'Region'),
    coords ? 'BOUNDARY         ' + coords : '',
    siteTypes ? 'SITE TYPES       ' + siteTypes : '',
    'SEEDS            ' + Number(payload.seed_count || 0),
    'OBSERVED         ' + Number(payload.observed_seed_count || 0),
    'ELEVATED         ' + Number(payload.elevated_seed_count || 0),
    'RECENT EVENTS    ' + Number(payload.recent_event_count || 0),
    'CRITICAL EVENTS  ' + Number(payload.critical_event_count || 0),
    'MAX PRIORITY     ' + Number(payload.highest_scan_priority || 0).toFixed(2),
    'AVG CONFIDENCE   ' + Math.round(Number(payload.average_observation_confidence || 0) * 100) + '%',
    discovery,
    lastDiscovery,
    lastRun,
    topSiteLine,
    'SUMMARY          ' + (payload.narrative || 'Regional monitoring overview ready.'),
  ].filter(Boolean).join('\n');
}

function formatRegionSitesPayload(payload) {
  if (!payload || typeof payload !== 'object' || !Array.isArray(payload.sites)) return null;
  const sites = payload.sites || [];
  if (!sites.length) return 'No site candidates have been mapped for this region yet.';
  return [
    'SITE CANDIDATES  ' + sites.length,
    '',
    sites.slice(0, 8).map(function(site, index) {
      const state = site.recommendation_review_state
        ? humanReviewState(site.recommendation_review_state)
        : (site.has_recommendation ? 'pending review' : String(site.seed_status || 'candidate'));
      return [
        '#' + String(index + 1).padStart(2, '0') + ' · ' + (site.name || site.seed_key || 'Site'),
        'TYPE            ' + String(site.site_type || 'candidate'),
        'RISK            ' + Math.round(Number(site.risk_score || 0) * 100) + '% · ' + state,
        'STATUS          ' + (site.status_reason || site.priority_reason || 'Monitoring'),
      ].join('\n');
    }).join('\n\n'),
  ].join('\n');
}

function formatCommandSummaryPayload(payload) {
  if (!payload || typeof payload !== 'object' || !Array.isArray(payload.attention_queue)) return null;
  const queue = payload.attention_queue || [];
  const actions = payload.maintenance && Array.isArray(payload.maintenance.suggested_actions)
    ? payload.maintenance.suggested_actions
    : [];
  const dashboard = payload.dashboard || {};
  const highlightRegion = payload.highlights && payload.highlights.top_region ? payload.highlights.top_region.name : '';
  const summary = payload.summary || dashboard.summary || 'Command center summary ready.';
  const queueLine = queue.length
    ? 'QUEUE            ' + queue.slice(0, 3).map(function(item) {
        return (item.title || item.kind || 'item') + ' [' + String(item.urgency_label || 'watch').toUpperCase() + ']';
      }).join(' · ')
    : 'QUEUE            clear';
  const actionLine = actions.length
    ? 'ACTIONS          ' + actions.slice(0, 3).map(function(item) { return item.title || 'action'; }).join(' · ')
    : 'ACTIONS          none suggested';
  return [
    'FOCUS REGION      ' + (payload.focus_region_id || highlightRegion || 'global'),
    'REGIONS ACTIVE    ' + Number(dashboard.region_count || 0),
    'SITES ELEVATED    ' + Number(dashboard.elevated_site_count || 0),
    'EVENTS LIVE       ' + Number(dashboard.live_event_count || 0),
    queueLine,
    actionLine,
    'SUMMARY          ' + summary,
  ].join('\n');
}

function formatEvidencePayload(payload) {
  if (!payload || typeof payload !== 'object') return null;
  const finding = payload.finding || payload.summary || payload.description || ('Evidence bundle for ' + (payload.object_id || 'object') + '.');
  const confidence = payload.confidence != null ? ('CONFIDENCE  ' + Math.round(Number(payload.confidence) * 100) + '%') : '';
  const bundleHash = payload.bundle_hash ? ('BUNDLE      ' + payload.bundle_hash) : '';
  const objectId = payload.object_id ? ('OBJECT      ' + payload.object_id) : '';
  const events = Array.isArray(payload.events) ? ('EVENTS      ' + payload.events.length) : '';
  const observations = Array.isArray(payload.observations) ? ('OBSERVED    ' + payload.observations.length) : '';
  const sources = Array.isArray(payload.sources) ? ('SOURCES     ' + payload.sources.length) : '';
  const signals = Array.isArray(payload.signals) && payload.signals.length
    ? ('SIGNALS     ' + payload.signals.slice(0, 3).map(function(signal) {
        return signal.name || signal.signal_type || signal.reason || 'signal';
      }).join(' · '))
    : '';
  const contributors = Array.isArray(payload.confidence_contributors) && payload.confidence_contributors.length
    ? ('DRIVERS     ' + payload.confidence_contributors.slice(0, 2).map(function(entry) {
        return (entry.name || 'factor') + ' ' + Math.round(Number(entry.weight || 0) * 100) + '%';
      }).join(' · '))
    : '';
  return [
    'FINDING     ' + finding,
    confidence,
    objectId,
    bundleHash,
    events,
    observations,
    sources,
    signals,
    contributors,
  ].filter(Boolean).join('\n');
}

function formatReplayPayload(payload) {
  if (!payload || typeof payload !== 'object') return null;
  const manifest = payload.manifest || payload;
  const executionId = payload.execution_id || manifest.manifest_hash || payload.replay_manifest_hash || '';
  const status = payload.status || payload.result || (payload.drift_detected != null ? 'executed' : 'ready');
  const requestId = payload.request_id || '';
  const summary = payload.summary || payload.description || payload.outcome || (payload.drift_detected != null ? 'Replay execution loaded.' : 'Replay artifact loaded.');
  const drift = payload.drift_detected == null ? '' : ('DRIFT       ' + (payload.drift_detected ? 'detected' : 'no drift'));
  const assessmentDelta = payload.diff && payload.diff.assessment
    ? ('ASSESSMENT  Δconf ' + Number(payload.diff.assessment.confidence_delta || 0).toFixed(2)
      + ' · Δpred ' + Number(payload.diff.assessment.prediction_probability_delta || 0).toFixed(2))
    : '';
  const decisionDelta = payload.diff && payload.diff.decision
    ? ('DECISION    Δrisk ' + Number(payload.diff.decision.risk_score_delta || 0).toFixed(2))
    : '';
  const changedFields = payload.diff && payload.diff.assessment && Array.isArray(payload.diff.assessment.changed_fields) && payload.diff.assessment.changed_fields.length
    ? ('FIELDS      ' + payload.diff.assessment.changed_fields.join(', '))
    : '';
  return [
    executionId ? 'REPLAY      ' + executionId : 'REPLAY      ready',
    'STATUS      ' + status,
    requestId ? 'REQUEST     ' + requestId : '',
    manifest.object_id ? 'OBJECT      ' + manifest.object_id : '',
    Array.isArray(manifest.event_ids) ? 'EVENTS      ' + manifest.event_ids.length : '',
    drift,
    assessmentDelta,
    decisionDelta,
    changedFields,
    'SUMMARY     ' + summary,
  ].filter(Boolean).join('\n');
}

function formatDrawerPayload(path, payload) {
  if (path && path.indexOf('/source-health/readiness') >= 0) {
    const formattedReadiness = formatReadinessPayload(payload);
    if (formattedReadiness) return formattedReadiness;
  }
  if (path && path.indexOf('/v1/passive/regions/') >= 0 && path.indexOf('/overview') >= 0) {
    const formattedRegionOverview = formatRegionOverviewPayload(payload);
    if (formattedRegionOverview) return formattedRegionOverview;
  }
  if (path && path.indexOf('/v1/passive/map/sites') >= 0) {
    const formattedRegionSites = formatRegionSitesPayload(payload);
    if (formattedRegionSites) return formattedRegionSites;
  }
  if (path && path.indexOf('/v1/passive/command-center/summary') >= 0) {
    const formattedCommandSummary = formatCommandSummaryPayload(payload);
    if (formattedCommandSummary) return formattedCommandSummary;
  }
  if (path && path.indexOf('/reviews') >= 0) {
    const formattedReviews = formatReviewHistoryPayload(payload);
    if (formattedReviews) return formattedReviews;
  }
  if (path && path.indexOf('/evidence/') >= 0) {
    const formattedEvidence = formatEvidencePayload(payload);
    if (formattedEvidence) return formattedEvidence;
  }
  if (path && path.indexOf('/replay/') >= 0) {
    const formattedReplay = formatReplayPayload(payload);
    if (formattedReplay) return formattedReplay;
  }
  return typeof payload === 'string' ? payload : JSON.stringify(payload, null, 2);
}

function openDrawer(title, path, payload) {
  const drawer = document.getElementById('data-drawer');
  const titleEl = document.getElementById('drawer-title');
  const pathEl = document.getElementById('drawer-path');
  const contentEl = document.getElementById('drawer-content');
  if (!drawer || !titleEl || !pathEl || !contentEl) return;
  const focusState = FOCUS_STATE.siteId && RECOMMENDATION_STATE[FOCUS_STATE.siteId]
    ? ' · ' + String(RECOMMENDATION_STATE[FOCUS_STATE.siteId]).toUpperCase()
    : '';
  titleEl.textContent = (title || 'Operational Detail') + focusState;
  pathEl.textContent = path || '';
  contentEl.textContent = formatDrawerPayload(path, payload);
  drawer.classList.add('visible');
}

async function openDrawerFromPath(title, path) {
  if (!path) return;
  try {
    const payload = await API.get(path);
    openDrawer(title, path, payload);
  } catch (error) {
    openDrawer(title, path, 'Failed to load operational detail: ' + String(error.message || error));
  }
}

function closeDrawer() {
  const drawer = document.getElementById('data-drawer');
  if (!drawer) return;
  drawer.classList.remove('visible');
}

function setFocusQuickLinks(links) {
  const container = document.getElementById('focus-links');
  if (!container) return;
  const validLinks = (links || []).filter(function(link) { return link && link.label && link.path; });
  if (!validLinks.length) {
    container.innerHTML = '';
    return;
  }
  container.innerHTML = validLinks.map(function(link) {
    const cls = link.warn ? 'att-action warn' : 'att-action subtle';
    const label = String(link.label).replace(/</g, '&lt;');
    const safeTitle = label.replace(/'/g, '&#39;');
    const path = String(link.path).replace(/'/g, '&#39;');
    return '<button class="' + cls + '" onclick="openDrawerFromPath(\'' + safeTitle + '\', \'' + path + '\')">' + label + '</button>';
  }).join('');
}

function ageBucketLabel(bucket) {
  const value = String(bucket || '').toLowerCase();
  if (!value) return '';
  if (value === 'new') return 'fresh';
  return value.replace(/_/g, ' ');
}

function formatDurationSeconds(totalSeconds) {
  const value = Math.max(0, Number(totalSeconds || 0));
  if (value < 60) return Math.round(value) + 's';
  if (value < 3600) return Math.round(value / 60) + 'm';
  if (value < 86400) return Math.round(value / 3600) + 'h';
  return Math.round(value / 86400) + 'd';
}

function setRiskContext(text) {
  const contextEl = document.getElementById('risk-context');
  if (!contextEl) return;
  contextEl.textContent = text || 'Waiting for focused analytics.';
}

function setTimelineSummary(text) {
  const summaryEl = document.getElementById('timeline-summary');
  if (!summaryEl) return;
  summaryEl.textContent = text || 'Event memory and forecast windows will appear here.';
}

function renderOperationalReviewStrip(counts) {
  const strip = document.getElementById('op-review-strip');
  if (!strip) return;
  const pending = Number((counts && counts.pending) || 0);
  const applied = Number((counts && counts.applied) || 0);
  const dismissed = Number((counts && counts.dismissed) || 0);
  const reviewed = Number((counts && counts.reviewed) || 0);
  const pills = [];
  if (pending > 0) pills.push('<span class="op-review-pill pending">' + pending + ' pending review</span>');
  if (applied > 0) pills.push('<span class="op-review-pill applied">' + applied + ' applied</span>');
  if (reviewed > 0) pills.push('<span class="op-review-pill">' + reviewed + ' reviewed</span>');
  if (dismissed > 0) pills.push('<span class="op-review-pill dismissed">' + dismissed + ' dismissed</span>');
  strip.innerHTML = pills.length
    ? pills.join('')
    : '<span class="op-review-pill">No operator review backlog</span>';
}

function forecastLayerEnabled() {
  const toggle = document.querySelector('.legend-toggle[data-layer="forecast"]');
  return !toggle || toggle.classList.contains('on');
}

function entityVisibleForFilter(kind, operatorState) {
  if (CONSOLE_STATE.filter === 'site') return kind === 'site';
  if (CONSOLE_STATE.filter === 'regional') return kind === 'region';
  if (CONSOLE_STATE.filter === 'incident') return kind === 'canonical_event';
  if (!reviewStateMatches(operatorState) && (kind === 'site' || kind === 'region')) return false;
  return true;
}

function clearForecastOverlay() {
  while (FORECAST_ENTITIES.length) {
    const entity = FORECAST_ENTITIES.pop();
    if (entity) viewer.entities.remove(entity);
  }
}

function renderForecastOverlaySite(siteId, lat, lng, points, siteLabel) {
  if (lat == null || lng == null || !points || !points.length) return;
  const stressed = points
    .filter(function(point) {
      return Number(point.reserve_stress_index || 0) >= 0.45 || Number(point.modeled_price_index || 0) >= 0.7;
    })
    .slice(0, 4);
  stressed.forEach(function(point, index) {
    const stress = Number(point.reserve_stress_index || 0);
    const price = Number(point.modeled_price_index || 0);
    const level = stress >= 0.75 ? 'critical' : stress >= 0.55 ? 'elevated' : 'active';
    const color = SITE_COLORS[level] || SITE_COLORS.active;
    const entity = viewer.entities.add({
      position: Cesium.Cartesian3.fromDegrees(lng, lat),
      point: {
        pixelSize: 7 + index,
        color: color.withAlpha(0.95),
        outlineColor: Cesium.Color.WHITE.withAlpha(0.4),
        outlineWidth: 1,
        heightReference: Cesium.HeightReference.CLAMP_TO_GROUND,
      },
      label: {
        text: 'T+' + point.offset_hours + 'H',
        font: '9px monospace',
        fillColor: color,
        outlineColor: Cesium.Color.BLACK,
        outlineWidth: 2,
        style: Cesium.LabelStyle.FILL_AND_OUTLINE,
        pixelOffset: new Cesium.Cartesian2(14, -10 - (index * 12)),
        showBackground: true,
        backgroundColor: Cesium.Color.fromCssColorString('#07111B').withAlpha(0.75),
        scale: 0.9,
        heightReference: Cesium.HeightReference.CLAMP_TO_GROUND,
      },
      show: forecastLayerEnabled(),
      properties: {
        kind: 'site',
        layer: 'forecast',
        site_id: siteId,
        label: (siteLabel ? siteLabel + ' · ' : '') + 'Forecast T+' + point.offset_hours + 'H',
        action_path: '/v1/forecast/sites/' + siteId + '/scenario?horizon_hours=24',
        reason: 'Reserve stress ' + (stress * 100).toFixed(0) + '% · price index ' + (price * 100).toFixed(0) + '%.',
      },
    });
    FORECAST_ENTITIES.push(entity);
  });
}

function renderForecastOverlay(siteId, lat, lng, points, siteLabel) {
  clearForecastOverlay();
  renderForecastOverlaySite(siteId, lat, lng, points, siteLabel);
}

async function renderRegionalForecastOverlay(regionId) {
  clearForecastOverlay();
  if (!regionId) return;
  try {
    const siteData = await API.get('/v1/passive/map/sites?region_id=' + encodeURIComponent(regionId));
    const sites = (siteData.sites || [])
      .filter(function(site) { return site.site_id && site.coordinates; })
      .sort(function(left, right) { return Number(right.risk_score || 0) - Number(left.risk_score || 0); })
      .slice(0, 3);
    const scenarios = await Promise.all(sites.map(function(site) {
      return API.get('/v1/forecast/sites/' + encodeURIComponent(site.site_id) + '/scenario?horizon_hours=24')
        .then(function(scenario) { return { site, scenario }; })
        .catch(function() { return null; });
    }));
    scenarios.filter(Boolean).forEach(function(entry) {
      renderForecastOverlaySite(
        entry.site.site_id,
        entry.site.coordinates.lat,
        entry.site.coordinates.lon,
        entry.scenario.points || [],
        entry.site.name || entry.site.seed_key || 'Site'
      );
    });
  } catch (_) { /* leave */ }
}

async function setRecommendationState(siteId, stateLabel) {
  if (!siteId || !stateLabel) return;
  const snapshotId = FOCUS_STATE.lastRecommendation && FOCUS_STATE.lastRecommendation.snapshot_id
    ? FOCUS_STATE.lastRecommendation.snapshot_id
    : null;
  await API.post('/v1/orchestrator/sites/' + encodeURIComponent(siteId) + '/reviews', {
    state: String(stateLabel).charAt(0).toUpperCase() + String(stateLabel).slice(1),
    actor: 'operator_console',
    rationale: 'Recorded from command center focus panel',
    snapshot_id: snapshotId,
  });
  RECOMMENDATION_STATE[siteId] = stateLabel;
  setFocusStateText('RECOMMENDATION ' + String(stateLabel).toUpperCase());
  if (FOCUS_STATE.siteId === siteId) {
    refreshFocusedSiteAnalytics().catch(() => {});
  }
}

async function executeRecommendedAction(action) {
  if (!action || !action.path) return;
  const list = document.getElementById('recommended-actions-list');
  if (list) list.innerHTML = '<div style="font-family:var(--font-mono);font-size:10px;color:var(--text-tertiary);padding:6px 0;line-height:1.6;">Executing action…</div>';
  const method = String(action.method || 'POST').toUpperCase();
  let payload = action.payload || {};
  if (!action.payload && action.path.indexOf('/worker/heartbeats/prune') >= 0) {
    payload = { older_than_seconds: 86400 };
  } else if (!action.payload && action.path.indexOf('/source-health/samples/prune') >= 0) {
    payload = { older_than_seconds: 2592000, dry_run: false };
  }
  try {
    const result = method === 'GET'
      ? await API.get(action.path)
      : await API.post(action.path, payload);
    openDrawer(action.title || 'Action result', action.path, result);
  } catch (error) {
    openDrawer(action.title || 'Action failed', action.path, 'Failed to execute: ' + String(error.message || error));
  }
  await refreshAll();
}

async function executeRecommendedActionByIndex(index) {
  const action = lastRecommendedActions[index];
  if (!action) return;
  await executeRecommendedAction(action);
}

function inspectRecommendedActionByIndex(index) {
  const action = lastRecommendedActions[index];
  if (!action) return;
  openDrawer(action.title || 'Recommended action', action.path || '', action);
}

function pushPressureHistory(points) {
  PRESSURE_HISTORY.length = 0;
  (points || []).slice(-12).forEach(function(point) {
    PRESSURE_HISTORY.push(point);
  });
}

function renderTimelineEntries(entries) {
  const strip = document.getElementById('timeline-events');
  if (!strip) return;
  if (!entries || !entries.length) {
    strip.innerHTML = '';
    return;
  }
  const range = currentTimelineRangeMs();
  const start = range.startMs;
  const total = range.totalMs;
  strip.innerHTML = entries
    .map(function(entry) {
      const ts = (entry.target_epoch_unix_seconds || entry.last_observed_at_unix_seconds || 0) * 1000;
      const pct = Math.max(2, Math.min(97, ((ts - start) / total) * 100));
      const cls = entry.forecast ? 'forecast' : (entry.className || sevCls(entry.severity));
      const title = (entry.title || 'event').replace(/'/g, '');
      const reason = (entry.reason || '').replace(/'/g, ' ').substring(0, 120);
      const kind = entry.kind || (entry.forecast ? 'site' : 'canonical_event');
      const lat = entry.lat != null ? entry.lat : 'null';
      const lng = entry.lng != null ? entry.lng : 'null';
      const siteId = entry.siteId ? String(entry.siteId).replace(/'/g, '') : '';
      const regionId = entry.regionId ? String(entry.regionId).replace(/'/g, '') : '';
      const actionPath = entry.actionPath ? String(entry.actionPath).replace(/'/g, '') : '';
      const evidencePath = entry.evidencePath ? String(entry.evidencePath).replace(/'/g, '') : '';
      const replayPath = entry.replayPath ? String(entry.replayPath).replace(/'/g, '') : '';
      const style = cls === 'forecast'
        ? 'left:' + pct.toFixed(1) + '%;cursor:pointer;'
        : 'left:' + pct.toFixed(1) + '%;cursor:pointer;';
      return '<div class="timeline-event ' + cls + '" '
        + 'style="' + style + '" '
        + 'title="' + title + '" '
        + 'onclick="openFocusPanel({kind:\'' + kind + '\',title:\'' + title + '\',reason:\'' + reason + '\',lat:' + lat + ',lng:' + lng + ',siteId:\'' + siteId + '\',regionId:\'' + regionId + '\',actionPath:\'' + actionPath + '\',evidencePath:\'' + evidencePath + '\',replayPath:\'' + replayPath + '\'})">'
        + '<div class="timeline-event-label">' + title + '</div></div>';
    })
    .join('');
}

async function refreshGlobalAnalytics() {
  try {
    clearForecastOverlay();
    const data = await API.get('/v1/passive/command-center/summary');
    const topSite = data.dashboard && data.dashboard.top_sites ? data.dashboard.top_sites[0] : null;
    const topRegion = data.highlights ? data.highlights.top_region : null;
    const pressure = topSite ? Number(topSite.risk_score || 0) : 0;
    const reviewCounts = (data.dashboard && data.dashboard.regions ? data.dashboard.regions : []).reduce(function(acc, region) {
      acc.pending += Number(region.recommendation_pending_review_count || 0);
      acc.reviewed += Number(region.recommendation_reviewed_count || 0);
      acc.applied += Number(region.recommendation_applied_count || 0);
      acc.dismissed += Number(region.recommendation_dismissed_count || 0);
      return acc;
    }, { pending: 0, reviewed: 0, applied: 0, dismissed: 0 });
    PRESSURE_HISTORY.push({ t: Date.now(), score: pressure });
    if (PRESSURE_HISTORY.length > 12) PRESSURE_HISTORY.shift();
    const titleEl = document.getElementById('risk-trend-title');
    if (titleEl) titleEl.textContent = topRegion ? topRegion.name + ' Risk' : 'Region Risk';
    const windowEl = document.getElementById('risk-trend-window');
    if (windowEl) windowEl.textContent = CONSOLE_STATE.window.toUpperCase();
    updateRiskTrend(pressure);
    setRiskContext(topRegion
      ? semanticRiskDeltaSentence(
          topRegion.risk_delta_classification,
          topRegion.risk_delta_explanation || (topRegion.name + ' is carrying the dominant regional pressure in the current ' + CONSOLE_STATE.window.toUpperCase() + ' window.')
        )
      : (topSite
        ? semanticRiskDeltaSentence(
            topSite.risk_delta_classification,
            topSite.risk_delta_explanation || ('Top site ' + topSite.name + ' is carrying ' + (pressure * 100).toFixed(0) + '% modeled pressure in the current ' + CONSOLE_STATE.window.toUpperCase() + ' window.')
          )
        : 'No focused site selected. Global pressure reflects the most elevated monitored surface.'));
    renderTimelineEntries((data.dashboard && data.dashboard.top_canonical_events) || []);
    setTimelineSummary('Global timeline showing canonical event memory across the current command window.');
    setFocusPrimaryButton('Open →', null);
    setFocusQuickLinks([]);
    setFocusStateText('');
    renderOperationalReviewStrip(reviewCounts);
  } catch (_) { /* leave */ }
}

async function refreshFocusedSiteAnalytics() {
  if (FOCUS_STATE.kind !== 'site' || !FOCUS_STATE.siteId) {
    if (FOCUS_STATE.kind === 'region' && FOCUS_STATE.regionId) {
      try {
        const regionId = FOCUS_STATE.regionId;
        const [overview, siteData, commandCenter] = await Promise.all([
          API.get('/v1/passive/regions/' + encodeURIComponent(regionId) + '/overview'),
          API.get('/v1/passive/map/sites?region_id=' + encodeURIComponent(regionId)),
          API.get('/v1/passive/command-center/summary'),
        ]);
        const sites = (siteData.sites || []).slice();
        const pendingSites = sites.filter(function(site) {
          return !site.recommendation_review_state && site.has_recommendation;
        });
        const appliedSites = sites.filter(function(site) {
          return String(site.recommendation_review_state || '').toLowerCase() === 'applied';
        });
        const reviewedSites = sites.filter(function(site) {
          return String(site.recommendation_review_state || '').toLowerCase() === 'reviewed';
        });
        const dismissedSites = sites.filter(function(site) {
          return String(site.recommendation_review_state || '').toLowerCase() === 'dismissed';
        });
        const observedSites = sites.filter(function(site) { return !!site.observed; });
        const elevatedSites = sites.filter(function(site) { return !!site.elevated; });
        const topRiskSite = sites
          .slice()
          .sort(function(left, right) { return Number(right.risk_score || 0) - Number(left.risk_score || 0); })[0];
        const focusDetail = document.getElementById('focus-detail');
        if (focusDetail) {
          focusDetail.textContent =
            'SITES ' + sites.length
            + ' · OBSERVED ' + observedSites.length
            + ' · ELEVATED ' + elevatedSites.length
            + ' · PENDING REVIEW ' + pendingSites.length;
        }
        const focusReason = document.getElementById('focus-reason');
        if (focusReason) {
          focusReason.textContent = overview.narrative || ('Regional monitoring active across ' + sites.length + ' site candidates.');
        }
        const narEl = document.getElementById('narrative-text');
        const metaEl = document.getElementById('narrative-meta');
        if (narEl) {
          narEl.textContent = overview.narrative || 'Regional monitoring active.';
          narEl.style.color = '';
        }
        if (metaEl) {
          metaEl.innerHTML =
            '<span>SITES · ' + sites.length + '</span>' +
            '<span>PENDING REVIEW · ' + pendingSites.length + '</span>' +
            '<span>ELEVATED · ' + elevatedSites.length + '</span>';
        }
        setFocusQuickLinks([
          { label: 'Region Overview', path: '/v1/passive/regions/' + encodeURIComponent(regionId) + '/overview' },
          { label: 'Region Sites', path: '/v1/passive/map/sites?region_id=' + encodeURIComponent(regionId) },
          { label: 'Command Summary', path: '/v1/passive/command-center/summary' },
        ]);
        setFocusPrimaryButton('Open Region →', '/v1/passive/regions/' + encodeURIComponent(regionId) + '/overview');
        setFocusStateText(pendingSites.length > 0 ? 'REGION NEEDS REVIEW' : 'REGION FORECAST OVERLAY ACTIVE');
        const focusReview = document.getElementById('focus-review');
        if (focusReview) {
          focusReview.textContent =
            'PENDING · ' + pendingSites.length
            + '\nREVIEWED · ' + reviewedSites.length
            + '\nAPPLIED · ' + appliedSites.length
            + '\nDISMISSED · ' + dismissedSites.length;
        }
        setRiskContext(
          semanticRiskDeltaSentence(
            overview.risk_delta_classification || (topRiskSite && topRiskSite.risk_delta_classification),
            overview.risk_delta_explanation || (topRiskSite && topRiskSite.risk_delta_explanation) || (topRiskSite ? ('Top regional site ' + topRiskSite.name + ' is carrying ' + (Number(topRiskSite.risk_score || 0) * 100).toFixed(0) + '% modeled pressure.') : 'Regional risk context will appear as monitored sites accumulate live signals.')
          )
        );
        renderOperationalReviewStrip({
          pending: pendingSites.length,
          reviewed: reviewedSites.length,
          applied: appliedSites.length,
          dismissed: dismissedSites.length,
        });
        const topEvents = (((commandCenter.dashboard || {}).top_canonical_events) || [])
          .filter(function(event) { return event.region_id === regionId; })
          .slice(0, 6);
        renderTimelineEntries(topEvents);
        setTimelineSummary(
          pendingSites.length > 0
            ? (pendingSites.length + ' site recommendation' + (pendingSites.length === 1 ? '' : 's') + ' waiting on operator review in this region.')
            : 'Regional timeline showing canonical memory for the selected command surface.'
        );
        const recList = document.getElementById('recommended-actions-list');
        if (recList) {
          const focusSites = (pendingSites.length ? pendingSites : sites)
            .slice()
            .sort(function(left, right) { return Number(right.risk_score || 0) - Number(left.risk_score || 0); })
            .slice(0, 3);
          recList.innerHTML = focusSites.length
            ? focusSites.map(function(site) {
              const state = site.recommendation_review_state
                ? humanReviewState(site.recommendation_review_state)
                : (site.has_recommendation ? 'pending review' : 'monitoring');
              const path = site.overview_path || ('/v1/passive/sites/' + encodeURIComponent(site.site_id || '') + '/overview');
              const narrativePath = site.narrative_path || '';
              return '<div style="padding:11px 12px;background:var(--bg-elevated);border-left:2px solid var(--amber);border-radius:0 2px 2px 0;">'
                + '<div style="font-size:12px;color:var(--text-primary);line-height:1.4;">' + escapeHtml(site.name || site.seed_key || 'Site') + '</div>'
                + '<div style="font-size:11px;color:var(--text-secondary);margin-top:3px;line-height:1.4;">' + escapeHtml(site.status_reason || site.priority_reason || 'Site candidate under regional watch.') + '</div>'
                + '<div style="font-family:var(--font-mono);font-size:9px;color:var(--text-tertiary);margin-top:6px;letter-spacing:0.08em;">STATE ' + escapeHtml(String(state).toUpperCase()) + ' · RISK ' + (Number(site.risk_score || 0) * 100).toFixed(0) + '%</div>'
                + '<div style="display:flex;gap:8px;flex-wrap:wrap;margin-top:10px;">'
                + '<button class="att-action subtle" onclick="openDrawerFromPath(\'Site Overview\', \'' + path.replace(/'/g, '&#39;') + '\')">Inspect</button>'
                + (narrativePath ? '<button class="att-action" onclick="openDrawerFromPath(\'Site Narrative\', \'' + narrativePath.replace(/'/g, '&#39;') + '\')">Narrative</button>' : '')
                + '</div></div>';
            }).join('')
            : '<div style="font-family:var(--font-mono);font-size:10px;color:var(--text-tertiary);padding:6px 0;line-height:1.6;">No region-specific actions required in the current window.</div>';
        }
      } catch (_) {
        await refreshGlobalAnalytics();
      }
      await renderRegionalForecastOverlay(FOCUS_STATE.regionId);
      return;
    }
    await refreshGlobalAnalytics();
    return;
  }
  try {
    const siteId = FOCUS_STATE.siteId;
    const [overview, scenario, recommendation, decisions, reviews] = await Promise.all([
      API.get('/v1/passive/sites/' + encodeURIComponent(siteId) + '/overview?limit=12'),
      API.get('/v1/forecast/sites/' + encodeURIComponent(siteId) + '/scenario?horizon_hours=24'),
      API.post('/v1/orchestrator/sites/' + encodeURIComponent(siteId) + '/recommend', {
        horizon_hours: 24,
        battery_state_of_charge: 0.45,
        reserve_margin_ratio: 0.18,
        price_signal_bias: 0.0,
      }),
      API.get('/v1/orchestrator/sites/' + encodeURIComponent(siteId) + '/decisions?limit=5').catch(function() { return []; }),
      API.get('/v1/orchestrator/sites/' + encodeURIComponent(siteId) + '/reviews?limit=5').catch(function() { return []; }),
    ]);

    FOCUS_STATE.lastRecommendation = recommendation;
    FOCUS_STATE.narrativePath = '/v1/passive/sites/' + encodeURIComponent(siteId) + '/narrative?days=30';
    FOCUS_STATE.actionPath = '/v1/passive/sites/' + encodeURIComponent(siteId) + '/overview';

    const narEl = document.getElementById('narrative-text');
    const metaEl = document.getElementById('narrative-meta');
    if (narEl) {
      narEl.textContent = (overview.risk_history_narrative && overview.risk_history_narrative.narrative)
        || recommendation.operational_reason
        || 'Focused site analytics active.';
      narEl.style.color = '';
    }
    if (metaEl) {
      metaEl.innerHTML =
        '<span>EVENTS · ' + ((overview.recent_events || []).length) + '</span>' +
        '<span>PATTERNS · ' + ((overview.recurring_patterns || []).length) + '</span>' +
        '<span>FORECAST · ' + (scenario.horizon_hours || 24) + 'H</span>';
    }
    const focusDetail = document.getElementById('focus-detail');
    if (focusDetail) {
      const confidence = Number(recommendation.confidence || 0) * 100;
      const sourceCount = overview.seed_lifecycle ? Number(overview.seed_lifecycle.source_count || 0) : 0;
      focusDetail.textContent =
        'CONFIDENCE ' + confidence.toFixed(0) + '% · SOURCES ' + sourceCount + ' · ACTION ' + String(recommendation.action || 'monitor_only').replace(/_/g, ' ');
    }

    const focusReason = document.getElementById('focus-reason');
    if (focusReason) {
      focusReason.textContent = recommendation.operational_reason + ' ' + recommendation.expected_benefit;
    }
    setFocusPrimaryButton('Open Overview →', FOCUS_STATE.actionPath);
    const provenance = overview.risk_history_narrative ? overview.risk_history_narrative.provenance : null;
    setFocusQuickLinks([
      { label: 'Narrative', path: '/v1/passive/sites/' + encodeURIComponent(siteId) + '/narrative?days=30' },
      { label: 'Forecast', path: '/v1/forecast/sites/' + encodeURIComponent(siteId) + '/scenario?horizon_hours=24' },
      { label: 'Reviews', path: '/v1/orchestrator/sites/' + encodeURIComponent(siteId) + '/reviews?limit=10' },
      provenance && provenance.bundle_hashes && provenance.bundle_hashes[0]
        ? { label: 'Evidence', path: '/v1/evidence/' + provenance.bundle_hashes[0], warn: true }
        : null,
      provenance && provenance.manifest_hashes && provenance.manifest_hashes[0]
        ? { label: 'Replay', path: '/v1/replay/' + provenance.manifest_hashes[0], warn: true }
        : null,
    ]);
    const latestReview = Array.isArray(reviews) && reviews.length ? reviews[0] : null;
    const recommendationState = latestReview && latestReview.state
      ? String(latestReview.state).toLowerCase()
      : RECOMMENDATION_STATE[siteId];
    if (recommendationState) {
      RECOMMENDATION_STATE[siteId] = recommendationState;
    }
    setFocusStateText(recommendationState ? ('RECOMMENDATION ' + String(recommendationState).toUpperCase()) : 'RECOMMENDATION PENDING REVIEW');
    const focusReview = document.getElementById('focus-review');
    if (focusReview) {
      if (latestReview) {
        focusReview.textContent =
          'LAST REVIEW · ' + tAgo(latestReview.decided_at_unix_seconds)
          + '\nSTATE · ' + humanReviewState(latestReview.state || recommendationState || '')
          + (latestReview.actor ? '\nACTOR · ' + latestReview.actor : '')
          + (latestReview.rationale ? '\nRATIONALE · ' + latestReview.rationale : '');
      } else {
        focusReview.textContent = recommendationState === 'pending_review'
          ? 'LAST REVIEW · Awaiting operator decision'
          : '';
      }
    }
    renderOperationalReviewStrip({
      pending: recommendationState === 'pending_review' ? 1 : 0,
      reviewed: recommendationState === 'reviewed' ? 1 : 0,
      applied: recommendationState === 'applied' ? 1 : 0,
      dismissed: recommendationState === 'dismissed' ? 1 : 0,
    });

    const riskTitleEl = document.getElementById('risk-trend-title');
    const riskWindowEl = document.getElementById('risk-trend-window');
    if (riskTitleEl) riskTitleEl.textContent = (overview.site && overview.site.site && overview.site.site.name ? overview.site.site.name : FOCUS_STATE.title || 'Site') + ' Risk';
    if (riskWindowEl) riskWindowEl.textContent = (scenario.horizon_hours || 24) + 'H';
    setRiskContext(semanticRiskDeltaSentence(
      overview.risk_history_narrative && overview.risk_history_narrative.risk_direction,
      (overview.site && overview.site.risk_delta_explanation)
        || (overview.risk_history_narrative && overview.risk_history_narrative.narrative)
        || recommendation.expected_benefit
    ));

    const historyPoints = ((overview.temporal_evolution || []).slice(-12)).map(function(point) {
      return {
        t: point.window_end_unix_seconds,
        score: Math.max(Number(point.peak_risk || 0), Number(point.cumulative_risk || 0)),
      };
    });
    if (historyPoints.length) {
      pushPressureHistory(historyPoints);
      updateRiskTrend(historyPoints[historyPoints.length - 1].score);
    } else {
      const forecastPoints = (scenario.points || []).slice(0, 12).map(function(point) {
        return { t: point.target_epoch_unix_seconds, score: Number(point.reserve_stress_index || 0) };
      });
      pushPressureHistory(forecastPoints);
      updateRiskTrend(forecastPoints.length ? forecastPoints[forecastPoints.length - 1].score : 0);
    }

    const recentEntries = (overview.recent_events || []).map(function(event) {
      return {
        kind: 'canonical_event',
        title: (event.site_name || event.threat_type || 'event').toUpperCase(),
        reason: event.summary || '',
        severity: severityClassFromScore(Number(event.risk_score || 0)),
        last_observed_at_unix_seconds: event.observed_at_unix_seconds,
        lat: _focusCoords.lat,
        lng: _focusCoords.lng,
        siteId: siteId,
        actionPath: '/v1/passive/sites/' + encodeURIComponent(siteId) + '/overview',
      };
    });
    const forecastEntries = (scenario.points || [])
      .filter(function(point) {
        return Number(point.reserve_stress_index || 0) >= 0.45 || Number(point.modeled_price_index || 0) >= 0.7;
      })
      .slice(0, 6)
      .map(function(point) {
        const stress = Number(point.reserve_stress_index || 0);
        return {
          kind: 'site',
          title: 'FORECAST T+' + point.offset_hours + 'H',
          reason: 'Reserve stress ' + (stress * 100).toFixed(0) + '% · price index ' + (Number(point.modeled_price_index || 0) * 100).toFixed(0) + '%.',
          forecast: true,
          className: 'forecast',
          target_epoch_unix_seconds: point.target_epoch_unix_seconds,
          lat: _focusCoords.lat,
          lng: _focusCoords.lng,
          siteId: siteId,
          actionPath: '/v1/forecast/sites/' + encodeURIComponent(siteId) + '/scenario?horizon_hours=24',
        };
      });
    renderTimelineEntries(recentEntries.concat(forecastEntries));
    setTimelineSummary(
      ((overview.recent_events || []).length) + ' recent event' + (((overview.recent_events || []).length) === 1 ? '' : 's')
      + ' · ' + forecastEntries.length + ' forecast window' + (forecastEntries.length === 1 ? '' : 's')
      + ' promoted into the next ' + (scenario.horizon_hours || 24) + 'h.'
    );
    renderForecastOverlay(
      siteId,
      _focusCoords.lat,
      _focusCoords.lng,
      scenario.points || [],
      overview.site && overview.site.site ? overview.site.site.name : ''
    );

    const recList = document.getElementById('recommended-actions-list');
    if (recList) {
      const decisionCount = Array.isArray(decisions) ? decisions.length : 0;
      const reviewed = RECOMMENDATION_STATE[siteId];
      const reviewMeta = latestReview
        ? ('LAST REVIEW ' + tAgo(latestReview.decided_at_unix_seconds)
          + (latestReview.actor ? ' · ' + escapeHtml(latestReview.actor) : '')
          + (latestReview.rationale ? '<br>' + escapeHtml(latestReview.rationale) : ''))
        : 'AWAITING OPERATOR DECISION';
      recList.innerHTML =
        '<div style="padding:11px 12px;background:var(--bg-elevated);border-left:2px solid var(--teal);border-radius:0 2px 2px 0;">'
        + '<div style="font-size:12px;color:var(--text-primary);line-height:1.4;">' + escapeHtml(String(recommendation.action || 'monitor_only').replace(/_/g, ' ').toUpperCase()) + '</div>'
        + '<div style="font-size:11px;color:var(--text-secondary);margin-top:3px;line-height:1.4;">' + escapeHtml(recommendation.expected_benefit || 'Operational recommendation ready.') + '</div>'
        + '<div style="font-family:var(--font-mono);font-size:9px;color:var(--text-tertiary);margin-top:6px;letter-spacing:0.08em;">'
        + 'CONFIDENCE ' + (Number(recommendation.confidence || 0) * 100).toFixed(0) + '% · DECISIONS ' + decisionCount
        + (reviewed ? ' · STATE ' + String(reviewed).toUpperCase() : '')
        + '</div>'
        + '<div style="font-family:var(--font-mono);font-size:9px;color:var(--text-secondary);margin-top:8px;line-height:1.7;">' + reviewMeta + '</div>'
        + '<div style="display:flex;gap:8px;flex-wrap:wrap;margin-top:10px;">'
        + '<button class="att-action subtle" onclick="setRecommendationState(\'' + siteId + '\', \'reviewed\').catch(() => {})">Review</button>'
        + '<button class="att-action" onclick="setRecommendationState(\'' + siteId + '\', \'applied\').catch(() => {})">Apply</button>'
        + '<button class="att-action warn" onclick="setRecommendationState(\'' + siteId + '\', \'dismissed\').catch(() => {})">Dismiss</button>'
        + '<button class="att-action subtle" onclick="openDrawerFromPath(\'Recommendation Reviews\', \'/v1/orchestrator/sites/' + encodeURIComponent(siteId) + '/reviews?limit=10\')">History</button>'
        + '</div></div>';
    }
  } catch (_) { /* leave */ }
}

function openFocusPanel({ kind, title, reason, lat, lng, siteId, regionId, actionPath, narrativePath, evidencePath, replayPath, operatorState }) {
  const panel   = document.getElementById('focus-panel');
  const fKind   = document.getElementById('focus-kind');
  const fTitle  = document.getElementById('focus-title');
  const fReason = document.getElementById('focus-reason');
  const fCoords = document.getElementById('focus-coords');
  const focusReview = document.getElementById('focus-review');
  if (!panel) return;
  const kindLabel = { site: 'SITE', region: 'REGION', canonical_event: 'EVENT', neo: 'NEO', maintenance: 'MAINTENANCE' }[kind] || (kind || 'SITE').toUpperCase();
  const kindCls   = kind === 'region' ? 'region' : kind === 'canonical_event' ? 'event' : '';
  fKind.textContent   = kindLabel;
  fKind.className     = 'focus-kind-badge' + (kindCls ? ' ' + kindCls : '');
  fTitle.textContent  = title || '\u2014';
  fReason.textContent = reason || '';
  fCoords.textContent = lat != null ? lat.toFixed(4) + '\u00b0N \u00b7 ' + lng.toFixed(4) + '\u00b0' : '';
  const focusDetail = document.getElementById('focus-detail');
  if (focusDetail) focusDetail.textContent = '';
  if (focusReview) focusReview.textContent = '';
  _focusCoords = { lat, lng };
  FOCUS_STATE.kind = kind || null;
  FOCUS_STATE.siteId = siteId || null;
  FOCUS_STATE.regionId = regionId || null;
  FOCUS_STATE.actionPath = actionPath || null;
  FOCUS_STATE.narrativePath = narrativePath || null;
  FOCUS_STATE.evidencePath = evidencePath || null;
  FOCUS_STATE.replayPath = replayPath || null;
  FOCUS_STATE.title = title || null;
  panel.classList.add('visible');
  if (lat != null) focusOnGlobe(lat, lng);
  setFocusPrimaryButton(actionPath ? 'Open →' : 'Recenter →', actionPath || null);
  setFocusQuickLinks([
    narrativePath ? { label: 'Narrative', path: narrativePath } : null,
    evidencePath ? { label: 'Evidence', path: evidencePath, warn: true } : null,
    replayPath ? { label: 'Replay', path: replayPath, warn: true } : null,
    replayPath && replayPath.indexOf('/replay/') >= 0 && replayPath.indexOf('/execute') < 0
      ? { label: 'Run Replay', path: replayPath + '/execute', warn: true }
      : null,
  ].filter(Boolean));
  setFocusStateText(operatorState ? ('RECOMMENDATION ' + String(operatorState).replace(/_/g, ' ').toUpperCase()) : '');
  refreshFocusedSiteAnalytics().catch(() => {});
}

// --- Console state (filter + time window) ---
const CONSOLE_STATE = { filter: 'global', reviewState: 'all', window: '72h' };
function windowAfterUnix() {
  const secs = { '24h': 86400, '72h': 259200, '7d': 604800, '30d': 2592000 };
  return Math.floor(Date.now() / 1000) - (secs[CONSOLE_STATE.window] || 259200);
}

// --- Chip interactions ---
document.querySelectorAll('.chip-group').forEach(group => {
  group.querySelectorAll('.chip').forEach(chip => {
    chip.addEventListener('click', () => {
      group.querySelectorAll('.chip').forEach(c => c.classList.remove('active'));
      chip.classList.add('active');
      if (chip.dataset.filter) CONSOLE_STATE.filter = chip.dataset.filter;
      if (chip.dataset.reviewState) CONSOLE_STATE.reviewState = chip.dataset.reviewState;
      if (chip.dataset.window) CONSOLE_STATE.window = chip.dataset.window;
      refreshAll();
    });
  });
});

// --- Legend toggles — wire to Cesium entity visibility ---
document.querySelectorAll('.legend-toggle').forEach(t => {
  t.addEventListener('click', () => {
    t.classList.toggle('on');
    const layer = t.dataset.layer;
    const isOn  = t.classList.contains('on');
    if (!layer || !viewer) return;
    viewer.entities.values.forEach(ent => {
      const props = ent.properties;
      if (!props) return;
      const entLayer = props.layer && props.layer.getValue();
      if (entLayer === layer) ent.show = isOn;
    });
  });
});

// --- Search ---
(function() {
  const input = document.getElementById('console-search');
  if (!input) return;
  let timeout = null;
  input.addEventListener('input', function() {
    clearTimeout(timeout);
    timeout = setTimeout(function() {
      const q = input.value.trim().toLowerCase();
      if (!q || q.length < 2) return;
      const r = GLOBE_DATA.regions.find(function(r) { return (r.name || r.region_id || '').toLowerCase().includes(q); });
      if (r && r.bbox) {
        const lat = (r.bbox.south + r.bbox.north) / 2;
        const lng = (r.bbox.west  + r.bbox.east)  / 2;
        openFocusPanel({
          kind: 'region',
          title: r.name || r.region_id,
          reason: (r.seeds_elevated || 0) + ' seeds elevated \u00b7 ' + (r.seeds_known || 0) + ' indexed',
          lat: lat,
          lng: lng,
          regionId: r.region_id || '',
          actionPath: '/v1/passive/regions/' + r.region_id + '/overview',
          operatorState: r.recommendation_pending_review_count > 0 ? 'pending_review' : (r.recommendation_applied_count > 0 ? 'applied' : (r.recommendation_reviewed_count > 0 ? 'reviewed' : (r.recommendation_dismissed_count > 0 ? 'dismissed' : ''))),
        });
        input.value = '';
        return;
      }
      const s = GLOBE_DATA.sites.find(function(s) { return ((s.name || '') + ' ' + (s.seed_key || '')).toLowerCase().includes(q); });
      if (s && s.coordinates) {
        openFocusPanel({
          kind: 'site',
          title: s.name || s.seed_key || 'Site',
          reason: s.status_reason || s.priority_reason || '',
          lat: s.coordinates.lat,
          lng: s.coordinates.lon,
          siteId: s.site_id || '',
          regionId: s.region_id || '',
          actionPath: s.overview_path || '',
          narrativePath: s.narrative_path || '',
          operatorState: s.recommendation_review_state ? String(s.recommendation_review_state).toLowerCase() : (s.has_recommendation ? 'pending_review' : ''),
        });
        input.value = '';
        return;
      }
      const ev = GLOBE_DATA.events.find(function(e) { return ((e.site_name || '') + ' ' + (e.event_type || '') + ' ' + (e.summary || '')).toLowerCase().includes(q); });
      if (ev && ev.coordinates) {
        openFocusPanel({
          kind: 'canonical_event',
          title: ev.site_name || ev.event_type || 'Event',
          reason: ev.summary || ev.operational_readout || '',
          lat: ev.coordinates.lat,
          lng: ev.coordinates.lon,
          siteId: ev.site_id || '',
          regionId: ev.region_id || '',
          actionPath: ev.site_id ? '/v1/passive/sites/' + ev.site_id + '/overview' : '',
          evidencePath: ev.bundle_hashes && ev.bundle_hashes[0] ? '/v1/evidence/' + ev.bundle_hashes[0] : '',
          replayPath: ev.manifest_hashes && ev.manifest_hashes[0] ? '/v1/replay/' + ev.manifest_hashes[0] : '',
        });
        input.value = '';
      }
    }, 350);
  });
  input.addEventListener('keydown', function(e) {
    if (e.key === 'Escape') { input.value = ''; input.blur(); }
  });
}());

/* ============================================================
   API INTEGRATION
   Field mapping against actual Rust response structs:
   - CanonicalEventsResponse { events: CanonicalPassiveEvent[] }
     CanonicalPassiveEvent: event_type, severity (critical/high/medium/low),
     summary, operational_readout, last_observed_at_unix_seconds, coordinates: {lat,lon}
   - PassiveCommandCenterSummary { summary, attention_queue, maintenance, highlights }
     maintenance.suggested_actions: [{ action_id, title, reason, method, path }]
     attention_queue: [{ item_id, kind, priority, title, reason, primary_action_label }]
   - OperationalVisibilitySummary { regions: OperationalVisibilityRegion[], total_regions }
     OperationalVisibilityRegion: { name, state: healthy|pressured|degraded|failing }
   - RegionMapResponse { regions: RegionMapItem[] }
     RegionMapItem: { region_id, name, enabled, seeds_known, seeds_elevated, max_priority }
   - SiteMapResponse { sites: SiteMapItem[] }  (requires ?region_id=...)
     SiteMapItem: { coordinates:{lat,lon}, elevated, scan_priority, name, seed_key }
   - IngestBatchLog: { request_id, source, timestamp_unix_seconds, records_received }
   ============================================================ */

const API = {
  async get(path) {
    const res  = await fetch(path);
    const json = await res.json();
    if (!res.ok) throw new Error((json.error && json.error.message) || 'HTTP ' + res.status);
    return json.data !== undefined ? json.data : json;
  },
  async post(path, payload) {
    const res = await fetch(path, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(payload || {}),
    });
    const json = await res.json();
    if (!res.ok) throw new Error((json.error && json.error.message) || 'HTTP ' + res.status);
    return json.data !== undefined ? json.data : json;
  }
};

// unix seconds → "Xm ago"
function tAgo(ts) {
  if (!ts) return '\u2014';
  const d = Math.floor(Date.now() / 1000 - ts);
  if (d < 60)    return d + 's ago';
  if (d < 3600)  return Math.floor(d / 60) + 'm ago';
  if (d < 86400) return Math.floor(d / 3600) + 'h ago';
  return Math.floor(d / 86400) + 'd ago';
}

function escapeHtml(value) {
  return String(value == null ? '' : value)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

function humanReviewState(stateValue) {
  const value = String(stateValue || '').toLowerCase();
  if (!value) return '';
  return value.replace(/_/g, ' ');
}

function semanticRiskDeltaLabel(deltaValue) {
  const value = String(deltaValue || '').toLowerCase();
  if (!value) return '';
  if (value === 'spike') return 'spiking';
  if (value === 'increase') return 'rising';
  if (value === 'decrease') return 'cooling';
  return 'stable';
}

function semanticRiskDeltaSentence(label, explanation) {
  const cleanLabel = semanticRiskDeltaLabel(label);
  if (!cleanLabel && !explanation) return 'Pressure is steady in the current window.';
  if (!cleanLabel) return explanation;
  return 'Risk is ' + cleanLabel + (explanation ? '. ' + explanation : '.');
}

function reviewStateMatches(stateValue) {
  if (!CONSOLE_STATE.reviewState || CONSOLE_STATE.reviewState === 'all') return true;
  return String(stateValue || '') === CONSOLE_STATE.reviewState;
}

// CanonicalPassiveEvent.severity → CSS marker class
function sevCls(sev) {
  const s = (sev || '').toLowerCase();
  if (s === 'critical') return 'critical';
  if (s === 'high')     return 'elevated';
  if (s === 'medium')   return 'active';
  return 'resolved';
}

function pClass(idx) {
  return idx <= 1 ? 'p1' : idx <= 3 ? 'p2' : 'p3';
}

// --- What Changed ---
// GET /v1/passive/canonical-events → CanonicalEventsResponse
async function refreshWhatChanged() {
  try {
    const data   = await API.get('/v1/passive/canonical-events?after_unix_seconds=' + windowAfterUnix());
    const events = data.events || [];
    const list   = document.getElementById('what-changed-list');
    const count  = document.getElementById('changes-count');
    if (!list) return;
    if (!events.length) {
      list.innerHTML = '<div style="font-family:var(--font-mono);font-size:10px;color:var(--text-tertiary);padding:8px 0;line-height:1.6;">No active changes detected in the current ' + CONSOLE_STATE.window + ' horizon.</div>';
      if (count) count.textContent = '0';
      return;
    }
    if (count) count.textContent = events.length + ' new';
    list.innerHTML = events.slice(0, 7).map(ev => {
      const cls  = sevCls(ev.severity);
      const type = (ev.event_type || 'event').toUpperCase().replace(/_/g, ' ');
      const text = ev.summary || ev.operational_readout || 'Event detected';
      const when = tAgo(ev.last_observed_at_unix_seconds);
      return '<div class="change-item">'
        + '<div class="change-marker ' + cls + '"></div>'
        + '<div class="change-body">'
        + '<div class="change-meta"><span class="change-type">' + type + '</span><span class="change-time">' + when + '</span></div>'
        + '<div class="change-title">' + text + '</div>'
        + '</div></div>';
    }).join('');
  } catch (_) { /* leave */ }
}

// --- Narrative ---
// GET /v1/passive/command-center/summary → PassiveCommandCenterSummary
async function refreshNarrative() {
  try {
    const data  = await API.get('/v1/passive/command-center/summary');
    const narEl = document.getElementById('narrative-text');
    const metaEl = document.getElementById('narrative-meta');
    if (!narEl) return;
    const attn  = data.attention_queue || [];
    const maint = data.maintenance || {};
    const acts  = (maint.suggested_actions || []).length;
    const stale = maint.stale_heartbeat_count || 0;
    // Build operational briefing language from server data
    let nar = '';
    if (!attn.length && !acts && !stale) {
      nar = 'Observation stable in the selected region. No live operational escalations detected in the current window. Maintenance queue is clear.';
    } else if (data.summary) {
      nar = data.summary
        .replace(/command center focused on ([^.\n]+)/i, 'Monitoring active across $1.')
        .replace(/^command center/i, 'Operational monitor');
      if (attn.length > 0) nar += ' ' + attn.length + ' item' + (attn.length === 1 ? '' : 's') + ' requiring attention.';
      if (acts > 0) nar += ' ' + acts + ' recommended action' + (acts === 1 ? '' : 's') + ' queued.';
    } else {
      nar = 'Monitoring active. ' + attn.length + ' attention item' + (attn.length === 1 ? '' : 's') + ' in queue.';
      if (stale > 0) nar += ' ' + stale + ' stale heartbeat' + (stale === 1 ? '' : 's') + ' detected.';
    }
    narEl.textContent = nar;
    narEl.style.color = '';
    if (metaEl) {
      metaEl.innerHTML =
        '<span>QUEUE \u00b7 ' + attn.length + '</span>' +
        '<span>STALE \u00b7 ' + stale + '</span>' +
        '<span>ACTIONS \u00b7 ' + acts + '</span>';
    }
  } catch (_) { /* leave */ }
}

// --- Source Health ---
// GET /v1/passive/operational-visibility → OperationalVisibilitySummary
// regions: [{ name, state: 'healthy'|'pressured'|'degraded'|'failing' }]
async function refreshSourceHealth() {
  try {
    const grid    = document.getElementById('source-health-list');
    const cnt     = document.getElementById('source-count');
    const sysEl   = document.getElementById('sys-sources');
    if (!grid) return;
    const [readiness, samplePayload] = await Promise.all([
      API.get('/v1/passive/source-health/readiness'),
      API.get('/v1/passive/source-health/samples?limit=250').catch(function() { return []; }),
    ]);
    const readinessSources = Array.isArray(readiness.sources) ? readiness.sources : [];
    const samples = Array.isArray(samplePayload) ? samplePayload : [];
    const scheduler = readiness.scheduler || {};
    const readinessById = {};
    readinessSources.forEach(function(source) {
      readinessById[String(source.source_id || '').toLowerCase()] = source;
    });
    const sourceIdForKind = {
      weather: 'open_meteo',
      firesmoke: 'firms',
      adsb: 'opensky',
    };
    const groupedSamples = {};
    samples.forEach(function(sample) {
      const rawKind = String(sample.source_kind || '').toLowerCase();
      const sourceId = sourceIdForKind[rawKind];
      if (!sourceId) return;
      if (!groupedSamples[sourceId]) {
        groupedSamples[sourceId] = {
          sampleCount: 0,
          successCount: 0,
          failureCount: 0,
          latestGeneratedAt: null,
          lastDetail: '',
        };
      }
      const bucket = groupedSamples[sourceId];
      bucket.sampleCount += 1;
      if (sample.fetched) bucket.successCount += 1;
      else bucket.failureCount += 1;
      if (!bucket.latestGeneratedAt || Number(sample.generated_at_unix_seconds || 0) > bucket.latestGeneratedAt) {
        bucket.latestGeneratedAt = Number(sample.generated_at_unix_seconds || 0);
        bucket.lastDetail = String(sample.detail || '');
      }
    });

    const scoreByReadiness = { ready: 94, awaiting_data: 46, needs_config: 12, degraded: 24 };
    const classByReadiness = { ready: 'healthy', awaiting_data: 'degraded', needs_config: 'failing', degraded: 'failing' };
    const scoreByStatus = { healthy: 97, watch: 74, degraded: 38, stale: 16 };
    const classByStatus = { healthy: 'healthy', watch: 'degraded', degraded: 'degraded', stale: 'failing' };
    const nowUnix = Math.floor(Date.now() / 1000);

    const rows = readinessSources.map(function(source) {
      const sourceId = String(source.source_id || '').toLowerCase();
      const sampleStats = groupedSamples[sourceId];
      const readinessState = String(source.readiness || 'awaiting_data').toLowerCase();
      let sc = scoreByReadiness[readinessState] || 40;
      let cl = classByReadiness[readinessState] || 'degraded';
      let meta = readinessState.replace(/_/g, ' ');
      let hint = source.reason || 'No readiness detail yet.';

      if (sampleStats && sampleStats.sampleCount > 0) {
        const successRate = sampleStats.successCount / Math.max(sampleStats.sampleCount, 1);
        const latestAgeSeconds = sampleStats.latestGeneratedAt ? nowUnix - sampleStats.latestGeneratedAt : null;
        let status = 'healthy';
        if (latestAgeSeconds != null && latestAgeSeconds > 21600) status = 'stale';
        else if (sampleStats.failureCount > 0 || successRate < 0.8) status = 'degraded';
        else if (successRate < 0.95) status = 'watch';
        sc = scoreByStatus[status] || Math.round(successRate * 100);
        cl = classByStatus[status] || 'degraded';
        meta = status + ' · ' + sampleStats.sampleCount + ' samples';
        hint = latestAgeSeconds != null
          ? Math.floor(latestAgeSeconds / 60) + 'm since latest sample'
          : (sampleStats.lastDetail || hint);
        if (sampleStats.failureCount > 0 && sampleStats.lastDetail) {
          hint += ' · ' + sampleStats.failureCount + ' failures';
        }
      } else if (String(source.source_id || '') === 'nasa_briefings') {
        meta = 'briefings only';
      }

      return {
        name: String(source.label || source.source_id || 'SOURCE').toUpperCase().substring(0, 16),
        score: sc,
        klass: cl,
        meta: meta,
        hint: hint,
        ready: readinessState === 'ready'
      };
    });

    const readyCount = rows.filter(function(row) { return row.ready; }).length;
    if (cnt) cnt.textContent = readyCount + ' / ' + rows.length;
    if (sysEl) {
      sysEl.textContent = scheduler.enabled
        ? readyCount + ' / ' + rows.length + ' feeds'
        : 'scheduler off';
    }
    grid.innerHTML = rows.slice(0, 6).map(function(row) {
      return '<div class="source-row">'
        + '<div class="source-name">' + escapeHtml(row.name) + '</div>'
        + '<div class="source-bar"><div class="source-bar-fill ' + row.klass + '" style="width:' + row.score + '%"></div></div>'
        + '<div class="source-value">' + row.score + '</div>'
        + '<div style="grid-column: 1 / span 3; font-family:var(--font-mono);font-size:9px;color:var(--text-tertiary);letter-spacing:0.08em;margin-top:4px;">' + escapeHtml(row.meta.toUpperCase()) + '</div>'
        + '<div style="grid-column: 1 / span 3; font-size:10px;color:var(--text-secondary);margin-top:4px;line-height:1.5;">' + escapeHtml(row.hint) + '</div>'
        + '</div>';
    }).join('');
  } catch (_) {
    const grid = document.getElementById('source-health-list');
    if (grid) grid.innerHTML = '<div style="font-family:var(--font-mono);font-size:10px;color:var(--text-tertiary);padding:8px 0;">Source health unavailable.</div>';
  }
}

// --- Provenance ---
// GET /v1/ingest/batches → Vec<IngestBatchLog>
// IngestBatchLog: { request_id, source, timestamp_unix_seconds, records_received }
async function refreshProvenance() {
  try {
    const data    = await API.get('/v1/ingest/batches');
    const list    = document.getElementById('provenance-list');
    if (!list) return;
    const batches = Array.isArray(data) ? data : [];
    if (!batches.length) {
      list.innerHTML = '<div style="font-family:var(--font-mono);font-size:10px;color:var(--text-tertiary);padding:8px 0;">No ingest batches yet.</div>';
      return;
    }
    list.innerHTML = batches.slice(0, 3).map(b => {
      const id   = (b.request_id || 'unknown').substring(0, 20);
      const kind = (b.source || 'INGEST').toUpperCase();
      const when = tAgo(b.timestamp_unix_seconds);
      const recs = b.records_received != null ? b.records_received + ' records' : '';
      return '<div style="display:flex;justify-content:space-between;padding:8px 0;border-bottom:1px dashed var(--line);">'
        + '<div>'
        + '<div style="font-family:var(--font-mono);font-size:10px;color:var(--text-tertiary);letter-spacing:0.08em;">' + kind + (recs ? ' \u00b7 ' + recs : '') + '</div>'
        + '<div style="font-size:12px;color:var(--text-primary);margin-top:2px;">' + id + '</div>'
        + '</div>'
        + '<div style="font-family:var(--font-mono);font-size:10px;color:var(--text-tertiary);align-self:center;">' + when + '</div>'
        + '</div>';
    }).join('');
  } catch (_) { /* leave */ }
}

// --- Op Picture + Globe ---
// GET /v1/passive/map/regions  → RegionMapResponse { regions: RegionMapItem[] }
//   RegionMapItem: { region_id, name, enabled, seeds_known, seeds_elevated,
//                   recent_events, critical_events, bbox: {south,west,north,east} }
// GET /v1/passive/canonical-events → CanonicalEventsResponse { events: [...] }
//   Uses region-level aggregates (seeds_elevated, recent_events) to avoid
//   hanging per-region site fetches which have no timeout guarantee.
async function refreshOpPicture() {
  try {
    const [regData, evData] = await Promise.all([
      API.get('/v1/passive/map/regions'),
      API.get('/v1/passive/canonical-events'),
    ]);
    const regions = regData.regions || [];
    const events  = evData.events   || [];

    const enabledRegions = regions.filter(r => r.enabled);
    const activeRegions  = enabledRegions.length;
    // seeds_elevated is the per-region count of elevated seeds (observed sites at risk)
    const elevSites  = enabledRegions.reduce((s, r) => s + (r.seeds_elevated || 0), 0);
    const liveEvents = events.length;
    // total seeds known across all regions
    const seedsKnown = enabledRegions.reduce((s, r) => s + (r.seeds_known || 0), 0);
    const reviewCounts = enabledRegions.reduce(function(acc, region) {
      acc.pending += Number(region.recommendation_pending_review_count || 0);
      acc.reviewed += Number(region.recommendation_reviewed_count || 0);
      acc.applied += Number(region.recommendation_applied_count || 0);
      acc.dismissed += Number(region.recommendation_dismissed_count || 0);
      return acc;
    }, { pending: 0, reviewed: 0, applied: 0, dismissed: 0 });

    const opR   = document.getElementById('op-regions');
    const opS   = document.getElementById('op-sites');
    const opEv  = document.getElementById('op-events');
    const opMsg = document.getElementById('op-message');
    if (opR)  opR.textContent  = activeRegions || '\u2014';
    if (opS)  opS.textContent  = elevSites;
    if (opEv) opEv.textContent = liveEvents;
    if (opMsg) {
      if (!regions.length) {
        opMsg.innerHTML = 'No regions configured. Add regions to begin monitoring.';
      } else if (!liveEvents && !elevSites) {
        opMsg.innerHTML = activeRegions + ' region' + (activeRegions === 1 ? '' : 's') + ' monitored'
          + (seedsKnown > 0 ? ' \u00b7 ' + seedsKnown.toLocaleString() + ' seeds indexed' : '')
          + (reviewCounts.pending > 0 ? ' \u00b7 ' + reviewCounts.pending + ' recommendation review' + (reviewCounts.pending === 1 ? '' : 's') + ' pending.' : ' \u00b7 awaiting scan results.');
      } else {
        opMsg.innerHTML = '<strong>' + elevSites + ' site' + (elevSites === 1 ? '' : 's') + '</strong> elevated \u00b7 '
          + liveEvents + ' event' + (liveEvents === 1 ? '' : 's') + ' live across '
          + activeRegions + ' region' + (activeRegions === 1 ? '' : 's') + '.'
          + (reviewCounts.pending > 0 ? ' ' + reviewCounts.pending + ' recommendation review' + (reviewCounts.pending === 1 ? '' : 's') + ' waiting on operator action.' : '');
      }
    }
    renderOperationalReviewStrip(reviewCounts);

    const siteResponses = await Promise.all(
      enabledRegions.map(function(r) {
        return API.get('/v1/passive/map/sites?region_id=' + encodeURIComponent(r.region_id))
          .catch(function() { return { sites: [] }; });
      })
    );
    const sites = siteResponses.flatMap(function(resp) { return resp.sites || []; });

    // Store in global cache for focus lookups
    GLOBE_DATA.regions = enabledRegions;
    GLOBE_DATA.sites   = sites;
    GLOBE_DATA.events  = events;

    // Rebuild globe entities from live API data (clear previous)
    viewer.entities.removeAll();
    enabledRegions.forEach(r => {
      if (r.bbox) {
        const lat   = (r.bbox.south + r.bbox.north) / 2;
        const lng   = (r.bbox.west  + r.bbox.east)  / 2;
        const level = (r.seeds_elevated || 0) > 0 ? 'elevated' : 'active';
        const color = SITE_COLORS[level] || SITE_COLORS.active;
        viewer.entities.add({
          position: Cesium.Cartesian3.fromDegrees(lng, lat),
          point: { pixelSize: SITE_SIZES[level] || 9, color, outlineColor: Cesium.Color.BLACK.withAlpha(0.7), outlineWidth: 1, heightReference: Cesium.HeightReference.CLAMP_TO_GROUND },
          label: { text: r.name || r.region_id, font: '10px monospace', fillColor: Cesium.Color.WHITE.withAlpha(0.85), outlineColor: Cesium.Color.BLACK, outlineWidth: 2, style: Cesium.LabelStyle.FILL_AND_OUTLINE, pixelOffset: new Cesium.Cartesian2(0, -18), scale: 1.0, showBackground: false, heightReference: Cesium.HeightReference.CLAMP_TO_GROUND },
          show: entityVisibleForFilter('region', r.recommendation_pending_review_count > 0 ? 'pending_review' : (r.recommendation_applied_count > 0 ? 'applied' : (r.recommendation_reviewed_count > 0 ? 'reviewed' : (r.recommendation_dismissed_count > 0 ? 'dismissed' : '')))),
          properties: { kind: 'region', layer: level, label: r.name || r.region_id, region_id: r.region_id, action_path: '/v1/passive/regions/' + r.region_id + '/overview', reason: (r.seeds_elevated || 0) + ' seeds elevated · ' + (r.seeds_known || 0) + ' indexed', operator_state: r.recommendation_pending_review_count > 0 ? 'pending_review' : (r.recommendation_applied_count > 0 ? 'applied' : (r.recommendation_reviewed_count > 0 ? 'reviewed' : (r.recommendation_dismissed_count > 0 ? 'dismissed' : ''))) },
        });
        // Region outline rectangle
        viewer.entities.add({
          rectangle: {
            coordinates: Cesium.Rectangle.fromDegrees(r.bbox.west, r.bbox.south, r.bbox.east, r.bbox.north),
            material: color.withAlpha(0.08),
            outline: true,
            outlineColor: color.withAlpha(0.5),
            outlineWidth: 1.5,
            height: 0,
          },
          show: entityVisibleForFilter('region', r.recommendation_pending_review_count > 0 ? 'pending_review' : (r.recommendation_applied_count > 0 ? 'applied' : (r.recommendation_reviewed_count > 0 ? 'reviewed' : (r.recommendation_dismissed_count > 0 ? 'dismissed' : '')))),
          properties: { kind: 'region', layer: level, label: r.name || r.region_id, region_id: r.region_id, action_path: '/v1/passive/regions/' + r.region_id + '/overview', reason: r.narrative_summary || ((r.seeds_elevated || 0) + ' seeds elevated'), operator_state: r.recommendation_pending_review_count > 0 ? 'pending_review' : (r.recommendation_applied_count > 0 ? 'applied' : (r.recommendation_reviewed_count > 0 ? 'reviewed' : (r.recommendation_dismissed_count > 0 ? 'dismissed' : ''))) },
        });
      }
    });
    sites.forEach(site => {
      if (site.coordinates) {
        const level = site.elevated ? 'elevated' : (site.observed ? 'active' : 'healthy');
        const color = SITE_COLORS[level] || SITE_COLORS.active;
        viewer.entities.add({
          position: Cesium.Cartesian3.fromDegrees(site.coordinates.lon, site.coordinates.lat),
          point: {
            pixelSize: site.elevated ? 10 : (site.observed ? 8 : 6),
            color,
            outlineColor: Cesium.Color.BLACK.withAlpha(0.7),
            outlineWidth: site.elevated ? 2 : 1,
            heightReference: Cesium.HeightReference.CLAMP_TO_GROUND
          },
          label: {
            text: site.name || site.seed_key || 'Site',
            font: '9px monospace',
            fillColor: Cesium.Color.WHITE.withAlpha(0.72),
            outlineColor: Cesium.Color.BLACK,
            outlineWidth: 2,
            style: Cesium.LabelStyle.FILL_AND_OUTLINE,
            pixelOffset: new Cesium.Cartesian2(0, 14),
            scale: 0.9,
            showBackground: false,
            heightReference: Cesium.HeightReference.CLAMP_TO_GROUND
          },
          show: entityVisibleForFilter('site', site.recommendation_review_state ? String(site.recommendation_review_state).toLowerCase() : (site.has_recommendation ? 'pending_review' : '')),
          properties: {
            kind: 'site',
            layer: level,
            label: site.name || site.seed_key || 'Site',
            site_id: site.site_id || '',
            region_id: site.region_id || '',
            action_path: site.overview_path || '',
            narrative_path: site.narrative_path || '',
            reason: site.status_reason || site.priority_reason || '',
            operator_state: site.recommendation_review_state ? String(site.recommendation_review_state).toLowerCase() : (site.has_recommendation ? 'pending_review' : ''),
            age_bucket: site.recommendation_age_bucket || '',
          },
        });
      }
    });
    events.forEach(ev => {
      if (ev.coordinates) {
        const level = sevCls(ev.severity);
        const color = SITE_COLORS[level] || SITE_COLORS.active;
        viewer.entities.add({
          position: Cesium.Cartesian3.fromDegrees(ev.coordinates.lon, ev.coordinates.lat),
          point: { pixelSize: SITE_SIZES[level] || 9, color, outlineColor: Cesium.Color.BLACK.withAlpha(0.7), outlineWidth: 1, heightReference: Cesium.HeightReference.CLAMP_TO_GROUND },
          show: entityVisibleForFilter('canonical_event', ''),
          properties: {
            kind: 'canonical_event',
            layer: level,
            label: ev.site_name || ev.event_type || 'Event',
            site_id: ev.site_id || '',
            region_id: ev.region_id || '',
            action_path: '/v1/passive/sites/' + ev.site_id + '/overview',
            reason: ev.summary || ev.operational_readout || '',
            evidence_path: ev.bundle_hashes && ev.bundle_hashes[0] ? '/v1/evidence/' + ev.bundle_hashes[0] : '',
            replay_path: ev.manifest_hashes && ev.manifest_hashes[0] ? '/v1/replay/' + ev.manifest_hashes[0] : '',
          },
        });
      }
    });
  } catch (e) {
    const opMsg = document.getElementById('op-message');
    if (opMsg) opMsg.innerHTML = 'Operational feed error — retrying…';
  }
}

// --- Attention Queue ---
// PassiveCommandCenterAttentionItem: { item_id, kind, priority, title, reason, primary_action_label }
// kind values: site | region | neo | canonical_event | maintenance
async function refreshAttentionQueue() {
  try {
    const data  = await API.get('/v1/passive/command-center/summary');
    const list  = document.getElementById('attention-queue-list');
    const cnt   = document.getElementById('attention-count');
    if (!list) return;
    const filterKind = { regional: 'region', site: 'site', incident: 'canonical_event' }[CONSOLE_STATE.filter];
    let items = data.attention_queue || [];
    if (filterKind) items = items.filter(i => i.kind === filterKind);
    items = items.filter(i => reviewStateMatches(i.operator_state));
    if (!items.length) {
      list.innerHTML = '<div style="font-family:var(--font-mono);font-size:10px;color:var(--text-tertiary);padding:8px 0;">Queue is clear.</div>';
      if (cnt) cnt.textContent = '0';
      return;
    }
    if (cnt) cnt.textContent = 'top ' + Math.min(items.length, 5);
    const kindMap = {
      site:            { cls: 'focus',    label: 'Site' },
      region:          { cls: 'region',   label: 'Region' },
      neo:             { cls: 'focus',    label: 'NEO Alert' },
      canonical_event: { cls: 'evidence', label: 'Event' },
      maintenance:     { cls: 'replay',   label: 'Maintenance' },
    };
    // Update Risk Trend window label
    const rtwEl = document.getElementById('risk-trend-window');
    if (rtwEl) rtwEl.textContent = CONSOLE_STATE.window.toUpperCase();

    lastAttentionItems = items.slice(0, 5);
    list.innerHTML = lastAttentionItems.map((item, idx) => {
      const pc  = pClass(idx);
      const km  = kindMap[item.kind] || { cls: 'evidence', label: String(item.kind || 'Item') };
      const num = String(idx + 1).padStart(2, '0');
      const lbl = item.primary_action_label || 'Open \u2192';
      const operatorState = item.operator_state
        ? '<span class="att-kind evidence" style="margin-left:8px;">' + String(item.operator_state).replace(/_/g, ' ') + '</span>'
        : '';
      const urgency = item.urgency_label
        ? '<span class="att-kind replay" style="margin-left:8px;">' + escapeHtml(item.urgency_label) + '</span>'
        : '';
      const ageBucket = item.age_bucket
        ? '<span class="att-kind subtle" style="margin-left:8px;">' + escapeHtml(ageBucketLabel(item.age_bucket)) + '</span>'
        : '';
      const age = item.age_seconds != null
        ? '<span class="change-time">' + tAgo(Math.floor(Date.now() / 1000) - Number(item.age_seconds || 0)) + '</span>'
        : '';
      return '<div class="attention-item">'
        + '<div class="priority-number ' + pc + '">' + num + '</div>'
        + '<div class="att-body">'
        + '<div class="att-head"><span class="att-kind ' + km.cls + '">' + km.label + '</span>' + operatorState + urgency + ageBucket + age + '</div>'
        + '<div class="att-title">' + (item.title || 'Unnamed') + '</div>'
        + (item.reason ? '<div class="att-reason">' + item.reason + '</div>' : '')
        + '<button class="att-action" onclick="handleAttentionClick(' + idx + ')">' + lbl + '</button>'
        + '</div></div>';
    }).join('');
  } catch (_) { /* leave */ }
}

function handleAttentionClick(idx) {
  const item = lastAttentionItems[idx];
  if (!item) return;
  // Resolve coordinates from cached globe data
  let lat = null, lng = null;
  if (item.kind === 'region') {
    const r = GLOBE_DATA.regions.find(r => r.name === item.title || r.region_id === item.title
      || (r.name || '').toLowerCase() === item.title.toLowerCase()
      || (r.region_id || '').toLowerCase() === item.title.toLowerCase());
    if (r && r.bbox) {
      lat = (r.bbox.south + r.bbox.north) / 2;
      lng = (r.bbox.west  + r.bbox.east)  / 2;
    } else if (GLOBE_DATA.regions.length) {
      // best effort: centroid of first region
      const first = GLOBE_DATA.regions[0];
      if (first.bbox) { lat = (first.bbox.south + first.bbox.north) / 2; lng = (first.bbox.west + first.bbox.east) / 2; }
    }
  } else if (item.kind === 'site') {
    const site = GLOBE_DATA.sites.find(s =>
      (item.site_id && s.site_id === item.site_id) ||
      (s.name || '').toLowerCase() === item.title.toLowerCase());
    if (site && site.coordinates) { lat = site.coordinates.lat; lng = site.coordinates.lon; }
  } else if (item.kind === 'canonical_event' || item.kind === 'neo') {
    const ev = GLOBE_DATA.events.find(e =>
      (e.summary || e.event_type || '').toLowerCase().includes(item.title.toLowerCase()) ||
      (e.site_name || '').toLowerCase().includes(item.title.toLowerCase()));
    if (ev && ev.coordinates) { lat = ev.coordinates.lat; lng = ev.coordinates.lon; }
    // No dangerous fallback — lat stays null, panel opens without fly-to
  }
  const evidencePath = Array.isArray(item.confirmation_read_paths)
    ? item.confirmation_read_paths.find(function(path) { return String(path).indexOf('/v1/evidence/') === 0; }) || ''
    : '';
  const replayPath = Array.isArray(item.confirmation_read_paths)
    ? item.confirmation_read_paths.find(function(path) { return String(path).indexOf('/v1/replay/') === 0; }) || ''
    : '';
  openFocusPanel({
    kind: item.kind,
    title: item.title,
    reason: item.reason,
    lat,
    lng,
    siteId: item.site_id || '',
    regionId: item.region_id || '',
    actionPath: item.action_path || (item.site_id ? '/v1/passive/sites/' + item.site_id + '/overview' : (item.region_id ? '/v1/passive/regions/' + item.region_id + '/overview' : '')),
    operatorState: item.operator_state || '',
    evidencePath: evidencePath,
    replayPath: replayPath,
  });
}

// --- Recommended Actions ---
// maintenance.suggested_actions: [{ action_id, title, reason, method, path }]
async function refreshRecommendedActions() {
  try {
    const data    = await API.get('/v1/passive/command-center/summary');
    const list    = document.getElementById('recommended-actions-list');
    if (!list) return;
    const actions = (data.maintenance || {}).suggested_actions || [];
    lastRecommendedActions = actions.slice(0, 4);
    if (!actions.length) {
      list.innerHTML = '<div style="font-family:var(--font-mono);font-size:10px;color:var(--text-tertiary);padding:6px 0;line-height:1.6;">No operator actions required in the current window. Monitoring continues.</div>';
      return;
    }
    const colors = ['var(--coral)', 'var(--amber)', 'var(--teal)', 'var(--lime)'];
    list.innerHTML = lastRecommendedActions.map((a, i) => {
      const color = colors[i % colors.length];
      return '<div style="padding:11px 12px;background:var(--bg-elevated);border-left:2px solid ' + color + ';border-radius:0 2px 2px 0;">'
        + '<div style="font-size:12px;color:var(--text-primary);line-height:1.4;">' + (a.title || 'Action required') + '</div>'
        + (a.reason ? '<div style="font-size:11px;color:var(--text-secondary);margin-top:3px;line-height:1.4;">' + a.reason + '</div>' : '')
        + '<div style="font-family:var(--font-mono);font-size:9px;color:var(--text-tertiary);margin-top:6px;letter-spacing:0.08em;">'
        + (a.method || 'POST') + ' ' + (a.path || '') + '</div>'
        + '<div style="display:flex;gap:8px;flex-wrap:wrap;margin-top:10px;">'
        + '<button class="att-action subtle" onclick="inspectRecommendedActionByIndex(' + i + ')">Inspect</button>'
        + '<button class="att-action" onclick="executeRecommendedActionByIndex(' + i + ')">Run</button>'
        + '</div></div>';
    }).join('');
  } catch (_) { /* leave */ }
}

// --- Event Composition ---
// CanonicalPassiveEvent: { event_type, severity, priority }
async function refreshEventComposition() {
  try {
    const data   = await API.get('/v1/passive/canonical-events?after_unix_seconds=' + windowAfterUnix());
    const events = data.events || [];
    const list   = document.getElementById('event-composition-list');
    const total  = document.getElementById('event-total');
    if (!list) return;
    if (!events.length) {
      list.innerHTML = '<div style="font-family:var(--font-mono);font-size:10px;color:var(--text-tertiary);padding:4px 0;">No events.</div>';
      if (total) total.textContent = '0 live';
      return;
    }
    if (total) total.textContent = events.length + ' live';
    const counts = {};
    events.forEach(ev => {
      const t = (ev.event_type || 'other').toUpperCase().replace(/_/g, ' ');
      counts[t] = (counts[t] || 0) + 1;
    });
    const sorted = Object.entries(counts).sort((a, b) => b[1] - a[1]);
    const max    = sorted[0][1];
    const colors = ['var(--coral)', 'var(--amber)', 'var(--teal)', 'var(--violet)', 'var(--lime)'];
    list.innerHTML = sorted.slice(0, 5).map(([type, n], i) => {
      const pct   = Math.round((n / max) * 100);
      const color = colors[i % colors.length];
      return '<div>'
        + '<div style="display:flex;justify-content:space-between;margin-bottom:4px;font-family:var(--font-mono);font-size:10.5px;">'
        + '<span style="color:var(--text-secondary);">' + type + '</span><span style="color:' + color + '">' + n + '</span>'
        + '</div>'
        + '<div style="height:3px;background:var(--bg-deep);border-radius:1px;overflow:hidden;">'
        + '<div style="width:' + pct + '%;height:100%;background:' + color + ';"></div>'
        + '</div></div>';
    }).join('');
  } catch (_) { /* leave */ }
}

// --- Risk Trend ---
function updateRiskTrend(pressure) {
  const metricEl = document.getElementById('risk-metric');
  const deltaEl  = document.getElementById('risk-delta');
  const lineEl   = document.getElementById('risk-line');
  const areaEl   = document.getElementById('risk-area');
  if (metricEl) metricEl.textContent = pressure.toFixed(2);
  if (deltaEl && PRESSURE_HISTORY.length > 1) {
    const prev = PRESSURE_HISTORY[PRESSURE_HISTORY.length - 2].score;
    const diff = pressure - prev;
    const sign = diff > 0.005 ? '\u2191' : diff < -0.005 ? '\u2193' : '\u2192';
    const col  = diff > 0.005 ? 'var(--coral)' : diff < -0.005 ? 'var(--lime)' : 'var(--text-tertiary)';
    deltaEl.style.color = col;
    deltaEl.innerHTML   = sign + ' ' + (diff >= 0 ? '+' : '') + (diff * 100).toFixed(1) + '%';
  }
  if (lineEl && areaEl && PRESSURE_HISTORY.length > 1) {
    const h = PRESSURE_HISTORY;
    const n = h.length;
    const pts = h.map(function(p, i) {
      const x = (i / (n - 1)) * 300;
      const y = 62 - Math.min(p.score, 1) * 56; // map 0-1 to y 62-6
      return x.toFixed(1) + ' ' + y.toFixed(1);
    });
    const lineD = 'M ' + pts.join(' L ');
    const first = pts[0].split(' ');
    const last  = pts[pts.length - 1].split(' ');
    const areaD = 'M ' + pts[0] + ' L ' + pts.slice(1).join(' L ') + ' L ' + last[0] + ' 64 L ' + first[0] + ' 64 Z';
    lineEl.setAttribute('d', lineD);
    areaEl.setAttribute('d', areaD);
  }
}

// --- Timeline strip ---
// --- Sys status ---
function refreshSysStatus() {
  const upd = document.getElementById('sys-updated');
  if (upd) upd.textContent = 'just now';
}

// --- Timeline labels ---
function updateTimelineLabels() {
  const range = currentTimelineRangeMs();
  const start = new Date(range.startMs);
  const end   = new Date(range.endMs);
  const pad   = n => String(n).padStart(2, '0');
  const fmtTime = d => pad(d.getUTCHours()) + ':' + pad(d.getUTCMinutes()) + ' UTC';
  const months  = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'];
  const fmtDate = d => months[d.getUTCMonth()] + ' ' + d.getUTCDate() + ', ' + d.getUTCFullYear();
  const el = id => document.getElementById(id);
  renderTimelineScale();
  if (el('timeline-start-time')) el('timeline-start-time').textContent = fmtTime(start);
  if (el('timeline-start-date')) el('timeline-start-date').textContent = fmtDate(start);
  if (el('timeline-end-time'))   el('timeline-end-time').textContent   = fmtTime(end);
  if (el('timeline-end-date'))   el('timeline-end-date').textContent   = fmtDate(end);
}

// --- Master refresh ---
async function refreshAll() {
  refreshSysStatus();
  updateTimelineLabels();
  await Promise.allSettled([
    refreshWhatChanged(),
    refreshNarrative(),
    refreshSourceHealth(),
    refreshProvenance(),
    refreshOpPicture(),
    refreshAttentionQueue(),
    refreshRecommendedActions(),
    refreshEventComposition(),
  ]);
  await refreshFocusedSiteAnalytics();
}

// Initial load + 30s polling
refreshAll();
setInterval(refreshAll, 30000);
</script>

</body>
</html>
"##;

const LANDING_PAGE_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>ShieldSky | Operational Intelligence Layer</title>
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Bebas+Neue&family=Barlow:ital,wght@0,300;0,400;0,500;0,600;1,300&family=Barlow+Condensed:wght@300;400;500;600;700&family=IBM+Plex+Mono:wght@300;400;500&display=swap" rel="stylesheet">

<style>
:root {
  --bg: #04090f;
  --bg2: #060f1a;
  --bg3: #071220;
  --teal: #00d4b8;
  --teal-dim: rgba(0,212,184,0.15);
  --teal-glow: rgba(0,212,184,0.4);
  --amber: #f0a500;
  --amber-dim: rgba(240,165,0,0.12);
  --coral: #ff6b6b;
  --white: #ffffff;
  --white-90: rgba(255,255,255,0.9);
  --white-60: rgba(255,255,255,0.6);
  --white-30: rgba(255,255,255,0.3);
  --white-10: rgba(255,255,255,0.07);
  --blue-mist: #8db4d8;
  --border: rgba(255,255,255,0.08);
  --border-teal: rgba(0,212,184,0.25);
}

*, *::before, *::after { margin: 0; padding: 0; box-sizing: border-box; }

html { scroll-behavior: smooth; }

body {
  background: var(--bg);
  color: var(--white);
  font-family: 'Barlow', sans-serif;
  overflow-x: hidden;
  cursor: default;
}

/* --- NOISE TEXTURE OVERLAY --- */
body::before {
  content: '';
  position: fixed;
  inset: 0;
  background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)' opacity='0.04'/%3E%3C/svg%3E");
  background-size: 200px 200px;
  pointer-events: none;
  z-index: 1000;
  opacity: 0.4;
}

/* --- SCANLINE --- */
body::after {
  content: '';
  position: fixed;
  inset: 0;
  background: repeating-linear-gradient(0deg, transparent, transparent 2px, rgba(0,0,0,0.03) 2px, rgba(0,0,0,0.03) 4px);
  pointer-events: none;
  z-index: 999;
}

/* --- GRID OVERLAY --- */
.grid-bg {
  position: fixed;
  inset: 0;
  background-image:
    linear-gradient(rgba(0,212,184,0.025) 1px, transparent 1px),
    linear-gradient(90deg, rgba(0,212,184,0.025) 1px, transparent 1px);
  background-size: 80px 80px;
  pointer-events: none;
  z-index: 0;
}

/* --- NAV --- */
nav {
  position: fixed;
  top: 0; left: 0; right: 0;
  z-index: 900;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 48px;
  background: linear-gradient(180deg, rgba(4,9,15,0.95) 0%, transparent 100%);
  backdrop-filter: blur(2px);
}

.nav-logo {
  display: flex;
  align-items: center;
  gap: 12px;
  text-decoration: none;
}

.nav-logo-dot {
  width: 10px; height: 10px;
  background: var(--teal);
  border-radius: 50%;
  box-shadow: 0 0 12px var(--teal), 0 0 24px var(--teal-glow);
  animation: pulse-dot 2s ease-in-out infinite;
}

@keyframes pulse-dot {
  0%, 100% { opacity: 1; box-shadow: 0 0 12px var(--teal), 0 0 24px var(--teal-glow); }
  50% { opacity: 0.7; box-shadow: 0 0 6px var(--teal), 0 0 12px var(--teal-glow); }
}

.nav-logo-text {
  font-family: 'Barlow Condensed', sans-serif;
  font-size: 18px;
  font-weight: 600;
  letter-spacing: 0.12em;
  color: var(--white);
  text-transform: uppercase;
}

.nav-links {
  display: flex;
  align-items: center;
  gap: 8px;
}

.nav-link {
  font-family: 'IBM Plex Mono', monospace;
  font-size: 11px;
  font-weight: 400;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--white-60);
  text-decoration: none;
  padding: 8px 16px;
  border: 1px solid var(--border);
  border-radius: 2px;
  transition: all 0.2s;
}

.nav-link:hover { color: var(--white); border-color: var(--white-30); }

.nav-cta {
  background: var(--teal);
  color: var(--bg) !important;
  font-weight: 500;
  border-color: var(--teal) !important;
}

.nav-cta:hover { background: #00f0d0; box-shadow: 0 0 20px var(--teal-glow); }

/* --- HERO --- */
#hero {
  position: relative;
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  overflow: hidden;
}

#globe-canvas {
  position: absolute;
  top: 50%;
  right: -5%;
  transform: translateY(-50%);
  width: 65vw;
  height: 65vw;
  max-width: 900px;
  max-height: 900px;
  opacity: 0.85;
  z-index: 1;
}

.hero-gradient {
  position: absolute;
  inset: 0;
  background:
    radial-gradient(ellipse 60% 80% at 20% 50%, rgba(0,212,184,0.04) 0%, transparent 70%),
    radial-gradient(ellipse 40% 60% at 75% 50%, rgba(0,212,184,0.06) 0%, transparent 70%),
    linear-gradient(90deg, var(--bg) 35%, transparent 70%);
  z-index: 2;
  pointer-events: none;
}

.hero-bottom-fade {
  position: absolute;
  bottom: 0; left: 0; right: 0;
  height: 200px;
  background: linear-gradient(0deg, var(--bg) 0%, transparent 100%);
  z-index: 3;
}

.hero-content {
  position: relative;
  z-index: 10;
  padding: 0 48px;
  max-width: 800px;
  animation: hero-in 1.2s cubic-bezier(0.16, 1, 0.3, 1) both;
}

@keyframes hero-in {
  from { opacity: 0; transform: translateY(30px); }
  to { opacity: 1; transform: translateY(0); }
}

.hero-tag {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  font-family: 'IBM Plex Mono', monospace;
  font-size: 11px;
  font-weight: 400;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: var(--teal);
  margin-bottom: 32px;
  animation: hero-in 1.2s 0.1s cubic-bezier(0.16, 1, 0.3, 1) both;
}

.hero-tag::before {
  content: '';
  display: block;
  width: 32px; height: 1px;
  background: var(--teal);
}

.hero-headline {
  font-family: 'Bebas Neue', sans-serif;
  font-size: clamp(72px, 9vw, 140px);
  font-weight: 400;
  line-height: 0.92;
  letter-spacing: 0.01em;
  color: var(--white);
  margin-bottom: 28px;
  animation: hero-in 1.2s 0.2s cubic-bezier(0.16, 1, 0.3, 1) both;
}

.hero-headline em {
  font-style: normal;
  color: transparent;
  -webkit-text-stroke: 1px rgba(255,255,255,0.4);
}

.hero-subline {
  font-size: 18px;
  font-weight: 300;
  line-height: 1.6;
  color: var(--white-60);
  max-width: 520px;
  margin-bottom: 48px;
  animation: hero-in 1.2s 0.3s cubic-bezier(0.16, 1, 0.3, 1) both;
}

.hero-subline strong {
  color: var(--white-90);
  font-weight: 500;
}

.hero-actions {
  display: flex;
  align-items: center;
  gap: 16px;
  flex-wrap: wrap;
  animation: hero-in 1.2s 0.4s cubic-bezier(0.16, 1, 0.3, 1) both;
}

.btn-primary {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  background: var(--teal);
  color: var(--bg);
  font-family: 'Barlow Condensed', sans-serif;
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  text-decoration: none;
  padding: 14px 32px;
  border-radius: 2px;
  transition: all 0.25s;
  border: none;
  cursor: pointer;
}

.btn-primary:hover {
  background: #00f0d0;
  box-shadow: 0 0 40px rgba(0,212,184,0.5), 0 8px 24px rgba(0,0,0,0.4);
  transform: translateY(-1px);
}

.btn-primary svg { width: 16px; height: 16px; }

.btn-secondary {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  background: transparent;
  color: var(--white-60);
  font-family: 'IBM Plex Mono', monospace;
  font-size: 11px;
  font-weight: 400;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  text-decoration: none;
  padding: 14px 24px;
  border-radius: 2px;
  border: 1px solid var(--border);
  transition: all 0.25s;
}

.btn-secondary:hover { color: var(--white); border-color: var(--white-30); background: var(--white-10); }

/* LIVE INDICATOR */
.live-badge {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-family: 'IBM Plex Mono', monospace;
  font-size: 10px;
  letter-spacing: 0.15em;
  color: var(--teal);
  text-transform: uppercase;
  padding: 6px 14px;
  border: 1px solid var(--border-teal);
  border-radius: 2px;
  background: var(--teal-dim);
  margin-left: 8px;
}

.live-dot {
  width: 6px; height: 6px;
  background: var(--teal);
  border-radius: 50%;
  animation: pulse-dot 1.5s ease-in-out infinite;
}

/* STATS BAR */
.hero-stats {
  position: absolute;
  bottom: 48px;
  left: 48px;
  right: 48px;
  z-index: 10;
  display: flex;
  align-items: center;
  gap: 48px;
  padding-top: 24px;
  border-top: 1px solid var(--border);
  animation: hero-in 1.2s 0.6s cubic-bezier(0.16, 1, 0.3, 1) both;
}

.stat-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.stat-value {
  font-family: 'Bebas Neue', sans-serif;
  font-size: 28px;
  color: var(--white);
  letter-spacing: 0.05em;
}

.stat-label {
  font-family: 'IBM Plex Mono', monospace;
  font-size: 10px;
  letter-spacing: 0.15em;
  text-transform: uppercase;
  color: var(--white-30);
}

/* --- SECTIONS --- */
section {
  position: relative;
  z-index: 10;
}

.section-inner {
  max-width: 1280px;
  margin: 0 auto;
  padding: 0 48px;
}

.section-tag {
  font-family: 'IBM Plex Mono', monospace;
  font-size: 11px;
  font-weight: 400;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: var(--teal);
  margin-bottom: 20px;
  display: flex;
  align-items: center;
  gap: 16px;
}

.section-tag::after {
  content: '';
  flex: 1;
  max-width: 64px;
  height: 1px;
  background: var(--border-teal);
}

.section-title {
  font-family: 'Bebas Neue', sans-serif;
  font-size: clamp(48px, 6vw, 88px);
  font-weight: 400;
  line-height: 0.94;
  letter-spacing: 0.01em;
  color: var(--white);
  margin-bottom: 24px;
}

/* --- SURFACES SECTION --- */
#surfaces {
  padding: 120px 0;
  background: linear-gradient(180deg, transparent 0%, rgba(0,212,184,0.015) 50%, transparent 100%);
}

.surfaces-header {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 48px;
  align-items: end;
  margin-bottom: 64px;
}

.surfaces-desc {
  font-size: 17px;
  font-weight: 300;
  line-height: 1.7;
  color: var(--white-60);
  align-self: end;
}

.surfaces-grid {
  display: grid;
  grid-template-columns: 2fr 1fr 1fr;
  grid-template-rows: auto auto;
  gap: 2px;
  background: var(--border);
}

.surface-card {
  background: var(--bg2);
  padding: 32px;
  position: relative;
  overflow: hidden;
  transition: background 0.3s;
  cursor: default;
}

.surface-card:hover { background: var(--bg3); }

.surface-card::before {
  content: '';
  position: absolute;
  top: 0; left: 0; right: 0;
  height: 1px;
  background: linear-gradient(90deg, var(--teal) 0%, transparent 60%);
  opacity: 0;
  transition: opacity 0.3s;
}

.surface-card:hover::before { opacity: 1; }

.surface-card.large {
  grid-column: 1;
  grid-row: 1 / 3;
}

.surface-card-tag {
  font-family: 'IBM Plex Mono', monospace;
  font-size: 10px;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: var(--teal);
  margin-bottom: 16px;
}

.surface-card-title {
  font-family: 'Barlow Condensed', sans-serif;
  font-size: 22px;
  font-weight: 600;
  letter-spacing: 0.04em;
  color: var(--white);
  margin-bottom: 12px;
}

.surface-card-desc {
  font-size: 14px;
  font-weight: 300;
  line-height: 1.6;
  color: var(--white-60);
}

/* MOCK SCREEN */
.surface-mock {
  margin-top: 28px;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 2px;
  padding: 16px;
  font-family: 'IBM Plex Mono', monospace;
  font-size: 11px;
}

.mock-bar {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 14px;
  padding-bottom: 10px;
  border-bottom: 1px solid var(--border);
}

.mock-dot { width: 6px; height: 6px; border-radius: 50%; }
.mock-dot.red { background: #ff6b6b; }
.mock-dot.yellow { background: #f0a500; }
.mock-dot.green { background: var(--teal); }

.mock-title { font-size: 10px; color: var(--white-30); letter-spacing: 0.1em; flex: 1; text-align: center; }

.mock-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 5px 0;
  border-bottom: 1px solid rgba(255,255,255,0.03);
  font-size: 10px;
}

.mock-row-label { color: var(--white-60); }
.mock-row-val { color: var(--teal); }
.mock-row-val.amber { color: var(--amber); }
.mock-row-val.coral { color: var(--coral); }
.mock-row-val.dim { color: var(--white-30); }

.mock-alert {
  margin-top: 10px;
  padding: 8px 10px;
  background: rgba(255,107,107,0.08);
  border-left: 2px solid var(--coral);
  font-size: 10px;
  color: var(--white-60);
  line-height: 1.5;
}

.mock-alert strong { color: var(--coral); }

/* ATT QUEUE MOCK */
.mock-queue-item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 8px 0;
  border-bottom: 1px solid rgba(255,255,255,0.04);
}

.mock-queue-dot {
  width: 6px; height: 6px;
  border-radius: 50%;
  margin-top: 3px;
  flex-shrink: 0;
}

.mock-queue-dot.high { background: var(--coral); box-shadow: 0 0 6px var(--coral); }
.mock-queue-dot.med { background: var(--amber); }
.mock-queue-dot.low { background: var(--teal); }

.mock-queue-text { font-size: 10px; color: var(--white-60); line-height: 1.5; }
.mock-queue-text strong { color: var(--white); }
.mock-queue-time { font-size: 9px; color: var(--white-30); margin-left: auto; white-space: nowrap; }

/* --- THINKING SECTION --- */
#thinking {
  padding: 140px 0;
  background: var(--bg2);
  overflow: hidden;
}

.thinking-header {
  margin-bottom: 80px;
}

.thinking-flow {
  display: flex;
  align-items: stretch;
  gap: 0;
  position: relative;
}

.thinking-flow::before {
  content: '';
  position: absolute;
  top: 50px;
  left: 50px;
  right: 50px;
  height: 1px;
  background: linear-gradient(90deg, transparent, var(--border-teal), var(--border-teal), transparent);
}

.thinking-step {
  flex: 1;
  padding: 40px 32px;
  border-left: 1px solid var(--border);
  position: relative;
  transition: background 0.3s;
}

.thinking-step:first-child { border-left: none; }
.thinking-step:hover { background: rgba(0,212,184,0.03); }

.thinking-step-num {
  font-family: 'IBM Plex Mono', monospace;
  font-size: 10px;
  letter-spacing: 0.2em;
  color: var(--white-30);
  margin-bottom: 24px;
}

.thinking-step-icon {
  width: 48px; height: 48px;
  border: 1px solid var(--border-teal);
  border-radius: 2px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 24px;
  background: var(--teal-dim);
  position: relative;
  z-index: 1;
}

.thinking-step-icon svg { width: 22px; height: 22px; stroke: var(--teal); fill: none; stroke-width: 1.5; }

.thinking-step-title {
  font-family: 'Bebas Neue', sans-serif;
  font-size: 32px;
  color: var(--white);
  letter-spacing: 0.04em;
  margin-bottom: 14px;
}

.thinking-step-desc {
  font-size: 14px;
  font-weight: 300;
  line-height: 1.7;
  color: var(--white-60);
}

/* --- WHY MATTERS --- */
#why {
  padding: 140px 0;
  position: relative;
  overflow: hidden;
}

#why::before {
  content: '';
  position: absolute;
  top: 0; left: 0; right: 0; bottom: 0;
  background: radial-gradient(ellipse 80% 60% at 80% 50%, rgba(240,165,0,0.03) 0%, transparent 70%);
}

.why-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1px;
  background: var(--border);
  margin-top: 64px;
}

.why-item {
  background: var(--bg);
  padding: 48px;
  position: relative;
  overflow: hidden;
}

.why-item::after {
  content: attr(data-num);
  position: absolute;
  bottom: -20px;
  right: -10px;
  font-family: 'Bebas Neue', sans-serif;
  font-size: 120px;
  color: rgba(255,255,255,0.02);
  line-height: 1;
  pointer-events: none;
}

.why-item-icon {
  width: 40px; height: 2px;
  background: var(--amber);
  margin-bottom: 28px;
}

.why-item-title {
  font-family: 'Barlow Condensed', sans-serif;
  font-size: 24px;
  font-weight: 600;
  letter-spacing: 0.04em;
  color: var(--white);
  margin-bottom: 14px;
}

.why-item-text {
  font-size: 15px;
  font-weight: 300;
  line-height: 1.7;
  color: var(--white-60);
}

/* --- OPERATORS SECTION --- */
#operators {
  padding: 140px 0;
  background: var(--bg2);
}

.operators-layout {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 100px;
  align-items: center;
  margin-top: 64px;
}

.operators-left { }

.operators-statement {
  font-family: 'Barlow', sans-serif;
  font-size: clamp(24px, 3vw, 36px);
  font-weight: 300;
  line-height: 1.5;
  color: var(--white-90);
  margin-bottom: 40px;
}

.operators-statement strong {
  color: var(--white);
  font-weight: 600;
}

.operators-metrics {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1px;
  background: var(--border);
}

.metric-cell {
  background: var(--bg3);
  padding: 28px;
}

.metric-num {
  font-family: 'Bebas Neue', sans-serif;
  font-size: 52px;
  color: var(--white);
  line-height: 1;
  margin-bottom: 6px;
}

.metric-num span { color: var(--teal); }

.metric-desc {
  font-family: 'IBM Plex Mono', monospace;
  font-size: 11px;
  color: var(--white-30);
  letter-spacing: 0.1em;
  text-transform: uppercase;
  line-height: 1.5;
}

.operators-right { }

.capability-list {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.capability-item {
  display: flex;
  align-items: flex-start;
  gap: 20px;
  padding: 24px 28px;
  background: rgba(255,255,255,0.02);
  border-left: 2px solid transparent;
  transition: all 0.3s;
}

.capability-item:hover {
  background: rgba(0,212,184,0.04);
  border-left-color: var(--teal);
}

.cap-index {
  font-family: 'IBM Plex Mono', monospace;
  font-size: 10px;
  color: var(--white-30);
  letter-spacing: 0.1em;
  padding-top: 3px;
  flex-shrink: 0;
}

.cap-content { }
.cap-title {
  font-family: 'Barlow Condensed', sans-serif;
  font-size: 16px;
  font-weight: 600;
  letter-spacing: 0.05em;
  color: var(--white);
  margin-bottom: 6px;
}

.cap-desc {
  font-size: 13px;
  font-weight: 300;
  color: var(--white-60);
  line-height: 1.6;
}

/* --- CTA SECTION --- */
#cta {
  padding: 160px 0;
  position: relative;
  overflow: hidden;
  text-align: center;
}

#cta::before {
  content: '';
  position: absolute;
  inset: 0;
  background:
    radial-gradient(ellipse 70% 70% at 50% 50%, rgba(0,212,184,0.07) 0%, transparent 70%);
}

#cta-canvas {
  position: absolute;
  inset: 0;
  z-index: 0;
  opacity: 0.6;
}

.cta-inner {
  position: relative;
  z-index: 10;
  max-width: 800px;
  margin: 0 auto;
  padding: 0 48px;
}

.cta-tag {
  font-family: 'IBM Plex Mono', monospace;
  font-size: 11px;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: var(--teal);
  margin-bottom: 28px;
}

.cta-headline {
  font-family: 'Bebas Neue', sans-serif;
  font-size: clamp(64px, 8vw, 120px);
  font-weight: 400;
  line-height: 0.9;
  color: var(--white);
  margin-bottom: 24px;
}

.cta-sub {
  font-size: 18px;
  font-weight: 300;
  color: var(--white-60);
  margin-bottom: 48px;
  line-height: 1.6;
}

.cta-actions {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  flex-wrap: wrap;
}

/* --- FOOTER --- */
footer {
  padding: 48px;
  border-top: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  position: relative;
  z-index: 10;
}

.footer-logo {
  display: flex;
  align-items: center;
  gap: 10px;
  text-decoration: none;
}

.footer-logo-text {
  font-family: 'Barlow Condensed', sans-serif;
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.15em;
  text-transform: uppercase;
  color: var(--white-60);
}

.footer-meta {
  font-family: 'IBM Plex Mono', monospace;
  font-size: 10px;
  color: var(--white-30);
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.footer-links {
  display: flex;
  gap: 24px;
}

.footer-link {
  font-family: 'IBM Plex Mono', monospace;
  font-size: 10px;
  color: var(--white-30);
  text-decoration: none;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  transition: color 0.2s;
}

.footer-link:hover { color: var(--teal); }

/* --- SCROLL REVEAL --- */
.reveal {
  opacity: 0;
  transform: translateY(24px);
  transition: opacity 0.8s cubic-bezier(0.16, 1, 0.3, 1), transform 0.8s cubic-bezier(0.16, 1, 0.3, 1);
}

.reveal.visible { opacity: 1; transform: translateY(0); }
.reveal-delay-1 { transition-delay: 0.1s; }
.reveal-delay-2 { transition-delay: 0.2s; }
.reveal-delay-3 { transition-delay: 0.3s; }
.reveal-delay-4 { transition-delay: 0.4s; }

/* --- RESPONSIVE --- */
@media (max-width: 1024px) {
  nav { padding: 16px 24px; }
  .hero-content { padding: 0 24px; }
  .hero-stats { left: 24px; right: 24px; gap: 32px; }
  .section-inner { padding: 0 24px; }
  #globe-canvas { width: 80vw; height: 80vw; right: -20%; }
  .surfaces-header { grid-template-columns: 1fr; }
  .surfaces-grid { grid-template-columns: 1fr 1fr; }
  .surface-card.large { grid-column: 1 / 3; grid-row: 1; }
  .thinking-flow { flex-wrap: wrap; }
  .thinking-flow::before { display: none; }
  .thinking-step { min-width: 45%; }
  .why-grid { grid-template-columns: 1fr; }
  .operators-layout { grid-template-columns: 1fr; gap: 48px; }
  footer { flex-direction: column; gap: 24px; text-align: center; }
}

@media (max-width: 640px) {
  .hero-headline { font-size: 60px; }
  .hero-stats { flex-wrap: wrap; gap: 20px; }
  #globe-canvas { width: 100vw; height: 100vw; right: -30%; opacity: 0.4; }
  .surfaces-grid { grid-template-columns: 1fr; }
  .surface-card.large { grid-column: 1; grid-row: 1; }
  .thinking-step { min-width: 100%; }
  .operators-metrics { grid-template-columns: 1fr; }
  .nav-links { gap: 4px; }
  .nav-link { padding: 6px 10px; font-size: 10px; }
}
</style>
</head>
<body>

<div class="grid-bg"></div>

<!-- NAV -->
<nav>
  <a href="/" class="nav-logo">
    <div class="nav-logo-dot"></div>
    <span class="nav-logo-text">ShieldSky</span>
  </a>
  <div class="nav-links">
    <a href="/health" class="nav-link">Health</a>
    <a href="/v1/health" class="nav-link">API</a>
    <a href="/console" class="nav-link nav-cta">Operator Console →</a>
  </div>
</nav>

<!-- HERO -->
<section id="hero">
  <canvas id="globe-canvas"></canvas>
  <div class="hero-gradient"></div>
  <div class="hero-bottom-fade"></div>

  <div class="hero-content">
    <div class="hero-tag">Operational Intelligence Layer</div>
    <h1 class="hero-headline">
      Observe<br>
      before<br>
      <em>escalation.</em>
    </h1>
    <p class="hero-subline">
      ShieldSky turns <strong>passive signals, orbital passes, and site anomalies</strong> into ranked intelligence, narrative briefings, and auditable decisions -- before an incident becomes an event.
    </p>
    <div class="hero-actions">
      <a href="/console" class="btn-primary">
        <svg viewBox="0 0 16 16" fill="currentColor"><path d="M3 8h10M9 4l4 4-4 4"/></svg>
        Enter Command Center
      </a>
      <a href="/v1/health" class="btn-secondary">API Surface →</a>
      <span class="live-badge">
        <span class="live-dot"></span>
        System Live
      </span>
    </div>
  </div>

  <div class="hero-stats">
    <div class="stat-item">
      <span class="stat-value">24/7</span>
      <span class="stat-label">Passive Observation</span>
    </div>
    <div class="stat-item">
      <span class="stat-value">∞</span>
      <span class="stat-label">Orbital Context</span>
    </div>
    <div class="stat-item">
      <span class="stat-value">ms</span>
      <span class="stat-label">Event Latency</span>
    </div>
    <div class="stat-item">
      <span class="stat-value">100%</span>
      <span class="stat-label">Auditability</span>
    </div>
  </div>
</section>

<!-- COMMAND SURFACES -->
<section id="surfaces">
  <div class="section-inner">
    <div class="surfaces-header">
      <div class="reveal">
        <div class="section-tag">Command Surfaces</div>
        <h2 class="section-title">What the<br>operator sees.</h2>
      </div>
      <p class="surfaces-desc reveal reveal-delay-2">
        Not charts. Not dashboards. A living intelligence surface — attention queue, narrative context, source health, and evidence — assembled in real time.
      </p>
    </div>

    <div class="surfaces-grid reveal">
      <!-- Large card: Command Center -->
      <div class="surface-card large">
        <div class="surface-card-tag">Primary Surface</div>
        <div class="surface-card-title">Operator Command Center</div>
        <p class="surface-card-desc">The full intelligence surface. Attention queue, narrative briefings, site status, anomaly detection, and evidence replay — unified in a single operational view.</p>
        <div class="surface-mock">
          <div class="mock-bar">
            <span class="mock-dot red"></span>
            <span class="mock-dot yellow"></span>
            <span class="mock-dot green"></span>
            <span class="mock-title">SHIELDSKY / CONSOLE</span>
          </div>
          <div class="mock-row"><span class="mock-row-label">OP PICTURE</span><span class="mock-row-val">● LIVE</span></div>
          <div class="mock-row"><span class="mock-row-label">REGIONS ACTIVE</span><span class="mock-row-val">7</span></div>
          <div class="mock-row"><span class="mock-row-label">CANONICAL EVENTS</span><span class="mock-row-val amber">3 UNREVIEWED</span></div>
          <div class="mock-row"><span class="mock-row-label">EVIDENCE BUNDLES</span><span class="mock-row-val">12 CAPTURED</span></div>
          <div class="mock-row"><span class="mock-row-label">INGEST LATENCY</span><span class="mock-row-val dim">4s</span></div>
          <div class="mock-alert"><strong>CANONICAL EVENT:</strong> Passive signal cluster aligned with orbital pass -- REGION-04. Evidence bundle captured. Narrative generated.</div>
        </div>
      </div>

      <!-- Attention Queue -->
      <div class="surface-card">
        <div class="surface-card-tag">Intelligence Layer</div>
        <div class="surface-card-title">Attention Queue</div>
        <p class="surface-card-desc">Prioritized signals, not raw feeds. Every item ranked, contextualized, actionable.</p>
        <div class="surface-mock" style="margin-top:20px">
          <div class="mock-bar">
            <span class="mock-dot red"></span><span class="mock-dot yellow"></span><span class="mock-dot green"></span>
            <span class="mock-title">ATTENTION / QUEUE</span>
          </div>
          <div class="mock-queue-item">
            <span class="mock-queue-dot high"></span>
            <div class="mock-queue-text"><strong>CANONICAL EVENT -- REGION-04</strong><br>Passive cluster + orbital pass. Evidence bundle active.</div>
            <span class="mock-queue-time">NOW</span>
          </div>
          <div class="mock-queue-item">
            <span class="mock-queue-dot med"></span>
            <div class="mock-queue-text"><strong>SITE DELTA-09</strong><br>Operational pressure above threshold. Narrative briefing ready.</div>
            <span class="mock-queue-time">06m</span>
          </div>
          <div class="mock-queue-item">
            <span class="mock-queue-dot low"></span>
            <div class="mock-queue-text"><strong>NEO FLYBY BRIEF</strong><br>Updated orbital context. Replay available.</div>
            <span class="mock-queue-time">14m</span>
          </div>
        </div>
      </div>

      <!-- Source Health -->
      <div class="surface-card">
        <div class="surface-card-tag">System Awareness</div>
        <div class="surface-card-title">Source Health</div>
        <p class="surface-card-desc">Signal quality, ingest latency, and source reliability — always visible.</p>
        <div class="surface-mock" style="margin-top:20px">
          <div class="mock-bar">
            <span class="mock-dot red"></span><span class="mock-dot yellow"></span><span class="mock-dot green"></span>
            <span class="mock-title">SOURCE / HEALTH</span>
          </div>
          <div class="mock-row"><span class="mock-row-label">PASSIVE INGEST</span><span class="mock-row-val">● NOMINAL</span></div>
          <div class="mock-row"><span class="mock-row-label">ORBITAL / NEOWS</span><span class="mock-row-val">● NOMINAL</span></div>
          <div class="mock-row"><span class="mock-row-label">SITE CORPUS</span><span class="mock-row-val amber">◐ PARTIAL</span></div>
          <div class="mock-row"><span class="mock-row-label">CANONICAL BUS</span><span class="mock-row-val">● NOMINAL</span></div>
        </div>
      </div>

      <!-- Narrative -->
      <div class="surface-card">
        <div class="surface-card-tag">Intelligence Output</div>
        <div class="surface-card-title">Narrative Briefings</div>
        <p class="surface-card-desc">Synthesized context, not raw data. Each event arrives with explanation, evidence, and confidence.</p>
      </div>

      <!-- Timeline -->
      <div class="surface-card">
        <div class="surface-card-tag">Evidence Layer</div>
        <div class="surface-card-title">Timeline & Replay</div>
        <p class="surface-card-desc">Full audit trail. Every decision traceable. Every signal captured. Replay any moment.</p>
      </div>
    </div>
  </div>
</section>

<!-- HOW SHIELDSKY THINKS -->
<section id="thinking">
  <div class="section-inner">
    <div class="thinking-header reveal">
      <div class="section-tag">Cognitive Architecture</div>
      <h2 class="section-title">How ShieldSky<br>thinks.</h2>
    </div>

    <div class="thinking-flow">
      <div class="thinking-step reveal">
        <div class="thinking-step-num">01</div>
        <div class="thinking-step-icon">
          <svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M2 12h2M20 12h2M5.6 5.6l1.4 1.4M17 17l1.4 1.4M5.6 18.4l1.4-1.4M17 7l1.4-1.4"/></svg>
        </div>
        <div class="thinking-step-title">Observe</div>
        <p class="thinking-step-desc">Continuous passive ingestion of regional signals, orbital passes, environmental data, and site anomalies — without interruption.</p>
      </div>

      <div class="thinking-step reveal reveal-delay-1">
        <div class="thinking-step-num">02</div>
        <div class="thinking-step-icon">
          <svg viewBox="0 0 24 24"><path d="M9 3H5a2 2 0 0 0-2 2v4m6-6h10a2 2 0 0 1 2 2v4M9 3v18m0 0h10a2 2 0 0 0 2-2V9M9 21H5a2 2 0 0 1-2-2V9m0 0h18"/></svg>
        </div>
        <div class="thinking-step-title">Interpret</div>
        <p class="thinking-step-desc">Raw signals become structured events. Canonical detection normalizes sources into comparable, traceable intelligence units.</p>
      </div>

      <div class="thinking-step reveal reveal-delay-2">
        <div class="thinking-step-num">03</div>
        <div class="thinking-step-icon">
          <svg viewBox="0 0 24 24"><polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"/></svg>
        </div>
        <div class="thinking-step-title">Prioritize</div>
        <p class="thinking-step-desc">Attention queue surfaces what matters now. Risk, recency, and operational context weight each signal automatically.</p>
      </div>

      <div class="thinking-step reveal reveal-delay-3">
        <div class="thinking-step-num">04</div>
        <div class="thinking-step-icon">
          <svg viewBox="0 0 24 24"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
        </div>
        <div class="thinking-step-title">Explain</div>
        <p class="thinking-step-desc">Every event arrives with narrative context, evidence references, and confidence levels — not just an alert.</p>
      </div>

      <div class="thinking-step reveal reveal-delay-4">
        <div class="thinking-step-num">05</div>
        <div class="thinking-step-icon">
          <svg viewBox="0 0 24 24"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>
        </div>
        <div class="thinking-step-title">Act</div>
        <p class="thinking-step-desc">Decision surfaces, triage workflows, and evidence-backed actions close the loop from observation to response.</p>
      </div>
    </div>
  </div>
</section>

<!-- WHY IT MATTERS -->
<section id="why">
  <div class="section-inner">
    <div class="reveal">
      <div class="section-tag">Operational Value</div>
      <h2 class="section-title">Not data.<br>Antecipation.</h2>
    </div>
    <div class="why-grid">
      <div class="why-item reveal" data-num="1">
        <div class="why-item-icon"></div>
        <div class="why-item-title">Passive Site Discovery</div>
        <p class="why-item-text">Sites reveal themselves through signal patterns before any active probe. ShieldSky builds the map from what it hears, not what it asks.</p>
      </div>
      <div class="why-item reveal reveal-delay-1" data-num="2">
        <div class="why-item-icon" style="background:var(--coral)"></div>
        <div class="why-item-title">Risk Narratives, Not Alerts</div>
        <p class="why-item-text">Alerts create noise. Narratives create understanding. Every anomaly arrives with context, trajectory, and recommended priority — already written.</p>
      </div>
      <div class="why-item reveal reveal-delay-2" data-num="3">
        <div class="why-item-icon" style="background:var(--teal)"></div>
        <div class="why-item-title">Orbital Context Fused</div>
        <p class="why-item-text">Ground-level signals and overhead orbital passes analyzed together. Canonical event detection sees what single-layer systems miss.</p>
      </div>
      <div class="why-item reveal reveal-delay-3" data-num="4">
        <div class="why-item-icon" style="background:#8db4d8"></div>
        <div class="why-item-title">Fully Auditable Decisions</div>
        <p class="why-item-text">Every action traces back to a signal, a source, a timestamp, a confidence level. Nothing is unexplained. Nothing is unrecorded.</p>
      </div>
    </div>
  </div>
</section>

<!-- BUILT FOR OPERATORS -->
<section id="operators">
  <div class="section-inner">
    <div class="operators-layout">
      <div class="operators-left reveal">
        <div class="section-tag">Built for Operators</div>
        <h2 class="section-title" style="font-size: clamp(40px,5vw,72px)">Know<br>before<br>it moves.</h2>
        <p class="operators-statement" style="font-size:17px; margin-top:28px">
          Built for <strong>regional intelligence operators and critical site teams</strong> who cannot afford to discover a threat in the after-action report.
        </p>
        <div class="operators-metrics">
          <div class="metric-cell">
            <div class="metric-num">7<span>+</span></div>
            <div class="metric-desc">Signal<br>source types</div>
          </div>
          <div class="metric-cell">
            <div class="metric-num">1<span>ms</span></div>
            <div class="metric-desc">Canonical<br>event latency</div>
          </div>
          <div class="metric-cell">
            <div class="metric-num">∞</div>
            <div class="metric-desc">Replay &<br>audit depth</div>
          </div>
          <div class="metric-cell">
            <div class="metric-num">0</div>
            <div class="metric-desc">Hidden<br>decisions</div>
          </div>
        </div>
      </div>

      <div class="operators-right reveal reveal-delay-2">
        <div class="capability-list">
          <div class="capability-item">
            <span class="cap-index">01</span>
            <div class="cap-content">
              <div class="cap-title">Regional Monitoring</div>
              <p class="cap-desc">Define regions, set thresholds, receive structured intelligence — not raw sensor dumps.</p>
            </div>
          </div>
          <div class="capability-item">
            <span class="cap-index">02</span>
            <div class="cap-content">
              <div class="cap-title">Anomaly Detection & Triage</div>
              <p class="cap-desc">Passive detection surfaces deviations automatically. Triage workflows guide response without ambiguity.</p>
            </div>
          </div>
          <div class="capability-item">
            <span class="cap-index">03</span>
            <div class="cap-content">
              <div class="cap-title">Evidence-Backed Workflows</div>
              <p class="cap-desc">Every decision surfaces supporting evidence. Every action records itself for future audit.</p>
            </div>
          </div>
          <div class="capability-item">
            <span class="cap-index">04</span>
            <div class="cap-content">
              <div class="cap-title">Orbital & Environmental Context</div>
              <p class="cap-desc">Ground signals enriched with orbital passes and environmental data — automatically fused.</p>
            </div>
          </div>
          <div class="capability-item">
            <span class="cap-index">05</span>
            <div class="cap-content">
              <div class="cap-title">API-First Integration</div>
              <p class="cap-desc">Full intelligence surface available via API. Connect systems. Extend coverage. Own the pipeline.</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</section>

<!-- FINAL CTA -->
<section id="cta">
  <canvas id="cta-canvas"></canvas>
  <div class="cta-inner reveal">
    <div class="cta-tag">— System Ready</div>
    <h2 class="cta-headline">The world<br>is already<br>moving.</h2>
    <p class="cta-sub">The command center is live. Every signal is being observed.<br>The question is whether you're watching.</p>
    <div class="cta-actions">
      <a href="/console" class="btn-primary" style="font-size:15px; padding: 16px 40px;">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor"><path d="M3 8h10M9 4l4 4-4 4"/></svg>
        Open Command Center
      </a>
      <a href="/v1/health" class="btn-secondary">System Health</a>
      <a href="/v1/briefing/apod" class="btn-secondary">Orbital Briefing</a>
    </div>
  </div>
</section>

<!-- FOOTER -->
<footer>
  <a href="/" class="footer-logo">
    <svg width="16" height="16" viewBox="0 0 28 28" fill="none"><path d="M14 2L3 6.5V14c0 5.8 4.5 11.2 11 13 6.5-1.8 11-7.2 11-13V6.5L14 2z" fill="rgba(0,212,184,0.10)" stroke="#00d4b8" stroke-width="1.5"/><circle cx="14" cy="13" r="1.8" fill="#00d4b8"/></svg>
    <span class="footer-logo-text">ShieldSky</span>
  </a>
  <div class="footer-meta">Signals become decisions here. Always on.</div>
  <div class="footer-links">
    <a href="/console" class="footer-link">Console</a>
    <a href="/v1/health" class="footer-link">API</a>
    <a href="/v1/briefing/neows" class="footer-link">Orbital Brief</a>
  </div>
</footer>

<script>
/* ==============================================
   GLOBE CANVAS
============================================== */
(function() {
  const canvas = document.getElementById('globe-canvas');
  const ctx = canvas.getContext('2d');
  let W, H, cx, cy, radius, animId;
  let rotY = 0;

  function resize() {
    const rect = canvas.getBoundingClientRect();
    W = canvas.width = rect.width * window.devicePixelRatio;
    H = canvas.height = rect.height * window.devicePixelRatio;
    cx = W / 2; cy = H / 2;
    radius = Math.min(W, H) * 0.42;
  }

  window.addEventListener('resize', resize);
  resize();

  // Generate globe points
  const DOTS = [];
  const NUM_DOTS = 2800;

  // Lat/lon regions with higher density (simulate landmasses roughly)
  const landRegions = [
    {lat:[20,70], lon:[-10,40]},    // Europe/Africa
    {lat:[30,70], lon:[40,140]},    // Asia
    {lat:[-35,35], lon:[-80,-35]},  // S.America
    {lat:[20,70], lon:[-130,-60]},  // N.America
    {lat:[-40,-10], lon:[115,155]}, // Australia
  ];

  function isLand(latDeg, lonDeg) {
    for (const r of landRegions) {
      if (latDeg >= r.lat[0] && latDeg <= r.lat[1] && lonDeg >= r.lon[0] && lonDeg <= r.lon[1]) return true;
    }
    return false;
  }

  for (let i = 0; i < NUM_DOTS; i++) {
    const u = Math.random();
    const v = Math.random();
    const theta = 2 * Math.PI * u;
    const phi = Math.acos(2 * v - 1);
    const x = Math.sin(phi) * Math.cos(theta);
    const y = Math.sin(phi) * Math.sin(theta);
    const z = Math.cos(phi);
    const latDeg = (Math.PI / 2 - phi) * 180 / Math.PI;
    const lonDeg = theta * 180 / Math.PI - 180;
    const land = isLand(latDeg, lonDeg);
    DOTS.push({ x, y, z, land, size: land ? (Math.random() * 1.5 + 0.8) : (Math.random() * 0.8 + 0.3) });
  }

  // Latitude lines
  const LAT_LINES = [];
  for (let lat = -60; lat <= 60; lat += 30) {
    const pts = [];
    const phi = (90 - lat) * Math.PI / 180;
    for (let lon = 0; lon <= 360; lon += 3) {
      const theta = lon * Math.PI / 180;
      pts.push({
        x: Math.sin(phi) * Math.cos(theta),
        y: Math.sin(phi) * Math.sin(theta),
        z: Math.cos(phi)
      });
    }
    LAT_LINES.push(pts);
  }

  // Longitude lines
  const LON_LINES = [];
  for (let lon = 0; lon < 360; lon += 30) {
    const pts = [];
    const theta = lon * Math.PI / 180;
    for (let lat = -90; lat <= 90; lat += 3) {
      const phi = (90 - lat) * Math.PI / 180;
      pts.push({
        x: Math.sin(phi) * Math.cos(theta),
        y: Math.sin(phi) * Math.sin(theta),
        z: Math.cos(phi)
      });
    }
    LON_LINES.push(pts);
  }

  function project(px, py, pz, ry) {
    // Rotate around Y axis
    const cosY = Math.cos(ry), sinY = Math.sin(ry);
    const rx = px * cosY + pz * sinY;
    const rz = -px * sinY + pz * cosY;
    return { sx: cx + rx * radius, sy: cy - py * radius, z: rz };
  }

  function draw() {
    ctx.clearRect(0, 0, W, H);

    // Draw lat/lon grid lines
    for (const line of [...LAT_LINES, ...LON_LINES]) {
      ctx.beginPath();
      let first = true;
      for (const p of line) {
        const {sx, sy, z} = project(p.x, p.y, p.z, rotY);
        if (z < -0.2) { first = true; continue; }
        const alpha = Math.max(0, (z + 0.2) / 1.2) * 0.12;
        if (first) { ctx.moveTo(sx, sy); first = false; }
        else ctx.lineTo(sx, sy);
      }
      ctx.strokeStyle = `rgba(0,212,184,0.12)`;
      ctx.lineWidth = 0.5;
      ctx.stroke();
    }

    // Sort dots by Z (painter's algorithm)
    const projected = DOTS.map(d => {
      const p = project(d.x, d.y, d.z, rotY);
      return {...d, ...p};
    });
    projected.sort((a, b) => a.z - b.z);

    for (const d of projected) {
      if (d.z < -0.1) continue;
      const depth = (d.z + 1) / 2;
      const alpha = depth * (d.land ? 0.95 : 0.35);
      const size = d.size * (0.5 + depth * 0.5) * (W / 800);

      if (d.land) {
        ctx.beginPath();
        ctx.arc(d.sx, d.sy, size, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(0,212,184,${alpha * 0.9})`;
        ctx.fill();
        if (depth > 0.7 && d.size > 1.2) {
          ctx.shadowBlur = 6 * (W / 800);
          ctx.shadowColor = 'rgba(0,212,184,0.6)';
          ctx.fill();
          ctx.shadowBlur = 0;
        }
      } else {
        ctx.beginPath();
        ctx.arc(d.sx, d.sy, size * 0.7, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(140,180,220,${alpha * 0.5})`;
        ctx.fill();
      }
    }

    // Draw globe outline
    const grad = ctx.createRadialGradient(cx - radius * 0.3, cy - radius * 0.3, 0, cx, cy, radius);
    grad.addColorStop(0, 'rgba(0,212,184,0.04)');
    grad.addColorStop(0.7, 'rgba(0,50,80,0.02)');
    grad.addColorStop(1, 'rgba(0,212,184,0.08)');
    ctx.beginPath();
    ctx.arc(cx, cy, radius, 0, Math.PI * 2);
    ctx.strokeStyle = 'rgba(0,212,184,0.15)';
    ctx.lineWidth = 1;
    ctx.stroke();

    // Atmosphere glow
    const atmoGrad = ctx.createRadialGradient(cx, cy, radius * 0.9, cx, cy, radius * 1.15);
    atmoGrad.addColorStop(0, 'rgba(0,212,184,0.0)');
    atmoGrad.addColorStop(0.6, 'rgba(0,212,184,0.04)');
    atmoGrad.addColorStop(1, 'rgba(0,212,184,0.0)');
    ctx.beginPath();
    ctx.arc(cx, cy, radius * 1.15, 0, Math.PI * 2);
    ctx.fillStyle = atmoGrad;
    ctx.fill();

    rotY += 0.0025;
    animId = requestAnimationFrame(draw);
  }

  draw();
})();

/* ==============================================
   CTA CANVAS — PARTICLE FIELD
============================================== */
(function() {
  const canvas = document.getElementById('cta-canvas');
  const ctx = canvas.getContext('2d');
  let W, H;

  function resize() {
    W = canvas.width = canvas.offsetWidth;
    H = canvas.height = canvas.offsetHeight;
  }
  window.addEventListener('resize', resize);
  resize();

  const particles = Array.from({length: 80}, () => ({
    x: Math.random() * 1200,
    y: Math.random() * 600,
    vx: (Math.random() - 0.5) * 0.3,
    vy: (Math.random() - 0.5) * 0.3,
    r: Math.random() * 1.5 + 0.3,
    alpha: Math.random() * 0.5 + 0.1,
  }));

  function drawCta() {
    ctx.clearRect(0, 0, W, H);
    for (const p of particles) {
      p.x += p.vx; p.y += p.vy;
      if (p.x < 0) p.x = W; if (p.x > W) p.x = 0;
      if (p.y < 0) p.y = H; if (p.y > H) p.y = 0;
      ctx.beginPath();
      ctx.arc(p.x, p.y, p.r, 0, Math.PI * 2);
      ctx.fillStyle = `rgba(0,212,184,${p.alpha})`;
      ctx.fill();
    }
    // Draw connections
    for (let i = 0; i < particles.length; i++) {
      for (let j = i + 1; j < particles.length; j++) {
        const dx = particles[i].x - particles[j].x;
        const dy = particles[i].y - particles[j].y;
        const dist = Math.sqrt(dx*dx + dy*dy);
        if (dist < 120) {
          ctx.beginPath();
          ctx.moveTo(particles[i].x, particles[i].y);
          ctx.lineTo(particles[j].x, particles[j].y);
          ctx.strokeStyle = `rgba(0,212,184,${(1 - dist/120) * 0.06})`;
          ctx.lineWidth = 0.5;
          ctx.stroke();
        }
      }
    }
    requestAnimationFrame(drawCta);
  }
  drawCta();
})();

/* ==============================================
   SCROLL REVEAL
============================================== */
const reveals = document.querySelectorAll('.reveal');
const observer = new IntersectionObserver((entries) => {
  entries.forEach(e => { if (e.isIntersecting) { e.target.classList.add('visible'); } });
}, { threshold: 0.1, rootMargin: '0px 0px -40px 0px' });
reveals.forEach(el => observer.observe(el));

/* ==============================================
   NAV SCROLL
============================================== */
const nav = document.querySelector('nav');
window.addEventListener('scroll', () => {
  if (window.scrollY > 60) {
    nav.style.background = 'rgba(4,9,15,0.97)';
    nav.style.backdropFilter = 'blur(12px)';
    nav.style.borderBottom = '1px solid rgba(255,255,255,0.05)';
  } else {
    nav.style.background = 'linear-gradient(180deg, rgba(4,9,15,0.95) 0%, transparent 100%)';
    nav.style.backdropFilter = 'blur(2px)';
    nav.style.borderBottom = 'none';
  }
});
</script>
</body>
</html>
"##;
