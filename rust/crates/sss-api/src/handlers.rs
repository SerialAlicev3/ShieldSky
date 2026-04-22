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
pub struct PassiveObservationsQuery {
    pub limit: Option<usize>,
    pub source_kind: Option<sss_passive_scanner::PassiveSourceKind>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PassiveSiteCorpusQuery {
    pub limit: Option<usize>,
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
<script src="https://cdnjs.cloudflare.com/ajax/libs/three.js/r128/three.min.js"></script>
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

  #globe-canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }

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

  /* Risk trend mini chart */
  .chart-container {
    padding: 14px 0 4px;
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
    <canvas id="globe-canvas"></canvas>

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
      <div class="tel-item"><span>ALT</span><span>12,400 km</span></div>
      <div class="tel-item"><span>FPS</span><span id="fps">60</span></div>
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
      <span class="time-label-kicker">T &minus; 36h</span>
      <span class="time-label-time" id="timeline-start-time">&#8212;</span>
      <span class="time-label-date" id="timeline-start-date">&#8212;</span>
    </div>

    <div class="timeline-container" id="timeline">
      <div class="timeline-track"></div>
      <div class="future-window" style="left: 60%; right: 0;"></div>

      <!-- Static ticks -->
      <div class="timeline-tick" style="left: 16.6%"><div class="timeline-tick-label">-30H</div></div>
      <div class="timeline-tick" style="left: 33.3%"><div class="timeline-tick-label">-24H</div></div>
      <div class="timeline-tick" style="left: 50%"><div class="timeline-tick-label">-12H</div></div>
      <div class="timeline-tick" style="left: 75%"><div class="timeline-tick-label">+12H</div></div>
      <div class="timeline-tick" style="left: 91.6%"><div class="timeline-tick-label">+24H</div></div>

      <!-- Events populated by JS -->
      <div id="timeline-events"></div>

      <!-- Now marker at 60% -->
      <div class="now-marker" style="left: 60%"></div>
    </div>

    <div class="time-label right">
      <span class="time-label-kicker">T + 36h</span>
      <span class="time-label-time" id="timeline-end-time">&#8212;</span>
      <span class="time-label-date" id="timeline-end-date">&#8212;</span>
    </div>
  </footer>

</div>

<script>
/* ============================================================
   OPERATIONAL GLOBE — Three.js r128
   ============================================================ */

const canvas = document.getElementById('globe-canvas');
const container = canvas.parentElement;

const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(
  35,
  container.clientWidth / container.clientHeight,
  0.1, 1000
);
camera.position.set(0, 0.2, 4.2);

const renderer = new THREE.WebGLRenderer({
  canvas: canvas,
  antialias: true,
  alpha: true
});
renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
renderer.setSize(container.clientWidth, container.clientHeight);
renderer.setClearColor(0x000000, 0);

// --- Globe sphere ---
const globeGeo = new THREE.SphereGeometry(1.5, 96, 96);
const globeMat = new THREE.MeshBasicMaterial({
  color: 0x0a1825,
  transparent: true,
  opacity: 0.85
});
const globe = new THREE.Mesh(globeGeo, globeMat);
scene.add(globe);

// --- Wireframe overlay ---
const wireGeo = new THREE.SphereGeometry(1.505, 48, 32);
const wireMat = new THREE.MeshBasicMaterial({
  color: 0x4FE0D0,
  wireframe: true,
  transparent: true,
  opacity: 0.09
});
const wireframe = new THREE.Mesh(wireGeo, wireMat);
scene.add(wireframe);

// --- Atmospheric glow ---
const glowGeo = new THREE.SphereGeometry(1.58, 64, 64);
const glowMat = new THREE.ShaderMaterial({
  uniforms: {},
  vertexShader: `
    varying vec3 vNormal;
    varying vec3 vPos;
    void main() {
      vNormal = normalize(normalMatrix * normal);
      vPos = position;
      gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
    }
  `,
  fragmentShader: `
    varying vec3 vNormal;
    varying vec3 vPos;
    void main() {
      float intensity = pow(0.7 - dot(vNormal, vec3(0.0, 0.0, 1.0)), 2.5);
      vec3 color = mix(vec3(0.31, 0.88, 0.82), vec3(0.61, 0.55, 1.0), smoothstep(-1.0, 1.0, vPos.y));
      gl_FragColor = vec4(color, 1.0) * intensity * 0.65;
    }
  `,
  blending: THREE.AdditiveBlending,
  side: THREE.BackSide,
  transparent: true
});
const glow = new THREE.Mesh(glowGeo, glowMat);
scene.add(glow);

// --- Utility: lat/lng to Three.js Vector3 ---
function latLngToVec3(lat, lng, r) {
  const phi   = (90 - lat) * (Math.PI / 180);
  const theta = (lng + 180) * (Math.PI / 180);
  return new THREE.Vector3(
    -r * Math.sin(phi) * Math.cos(theta),
     r * Math.cos(phi),
     r * Math.sin(phi) * Math.sin(theta)
  );
}

// --- Continental landmass dots (procedural) ---
function createLandmassDots() {
  const group = new THREE.Group();
  const landmasses = [
    { lat: [35,  60], lng: [-10,  40], density: 0.55 },
    { lat: [-35, 35], lng: [-18,  50], density: 0.4  },
    { lat: [10,  70], lng: [ 40, 140], density: 0.45 },
    { lat: [15,  70], lng: [-170,-55], density: 0.4  },
    { lat: [-55, 12], lng: [-82, -35], density: 0.45 },
    { lat: [-40,-10], lng: [112, 155], density: 0.5  },
  ];
  const dotGeo = new THREE.SphereGeometry(0.007, 6, 6);
  const dotMat = new THREE.MeshBasicMaterial({ color: 0x3a5d7a, transparent: true, opacity: 0.6 });
  for (const lm of landmasses) {
    const count = Math.floor((lm.lat[1]-lm.lat[0]) * (lm.lng[1]-lm.lng[0]) * lm.density * 0.15);
    for (let i = 0; i < count; i++) {
      if (Math.random() > 0.6) continue;
      const lat = lm.lat[0] + Math.random() * (lm.lat[1] - lm.lat[0]);
      const lng = lm.lng[0] + Math.random() * (lm.lng[1] - lm.lng[0]);
      const dot = new THREE.Mesh(dotGeo, dotMat);
      dot.position.copy(latLngToVec3(lat, lng, 1.51));
      group.add(dot);
    }
  }
  return group;
}

const landmassDots = createLandmassDots();
scene.add(landmassDots);

// --- Lat / longitude grid lines ---
function createGrid() {
  const group = new THREE.Group();
  const mat = new THREE.LineBasicMaterial({ color: 0x4FE0D0, transparent: true, opacity: 0.08 });
  for (let lat = -60; lat <= 60; lat += 30) {
    const pts = [];
    for (let lng = 0; lng <= 360; lng += 4) pts.push(latLngToVec3(lat, lng, 1.51));
    group.add(new THREE.Line(new THREE.BufferGeometry().setFromPoints(pts), mat));
  }
  for (let lng = 0; lng < 360; lng += 30) {
    const pts = [];
    for (let lat = -90; lat <= 90; lat += 4) pts.push(latLngToVec3(lat, lng, 1.51));
    group.add(new THREE.Line(new THREE.BufferGeometry().setFromPoints(pts), mat));
  }
  return group;
}
scene.add(createGrid());

// --- Equator highlight ---
(function() {
  const pts = [];
  for (let lng = 0; lng <= 360; lng += 2) pts.push(latLngToVec3(0, lng, 1.512));
  scene.add(new THREE.Line(
    new THREE.BufferGeometry().setFromPoints(pts),
    new THREE.LineBasicMaterial({ color: 0x4FE0D0, transparent: true, opacity: 0.25 })
  ));
})();

// --- Site rendering ---
const siteColors = { critical: 0xFF7C6E, elevated: 0xF1C96B, active: 0x4FE0D0, healthy: 0x8EE59B };
const siteMeshes = [];

function addSiteToGlobe(site) {
  const pos   = latLngToVec3(site.lat, site.lng, 1.515);
  const color = siteColors[site.level] || siteColors.active;
  const size  = site.level === 'critical' ? 0.025 : site.level === 'elevated' ? 0.02 : 0.014;

  const dot = new THREE.Mesh(
    new THREE.SphereGeometry(size, 10, 10),
    new THREE.MeshBasicMaterial({ color })
  );
  dot.position.copy(pos);
  scene.add(dot);
  siteMeshes.push({ mesh: dot, level: site.level, basePos: pos.clone() });

  // Halo ring
  const halo = new THREE.Mesh(
    new THREE.RingGeometry(size * 1.8, size * 2.2, 24),
    new THREE.MeshBasicMaterial({ color, transparent: true, opacity: 0.5, side: THREE.DoubleSide })
  );
  halo.position.copy(pos);
  halo.lookAt(0, 0, 0);
  scene.add(halo);

  // Beam + pulse for critical / elevated
  if (site.level === 'critical' || site.level === 'elevated') {
    const dir        = pos.clone().normalize();
    const beamLength = site.level === 'critical' ? 0.3 : 0.18;
    const beam = new THREE.Mesh(
      new THREE.CylinderGeometry(0.003, 0.008, beamLength, 8, 1, true),
      new THREE.MeshBasicMaterial({ color, transparent: true, opacity: 0.5, side: THREE.DoubleSide })
    );
    beam.position.copy(pos.clone().add(dir.clone().multiplyScalar(beamLength / 2)));
    beam.lookAt(pos.clone().multiplyScalar(2));
    beam.rotateX(Math.PI / 2);
    scene.add(beam);

    const pulse = new THREE.Mesh(
      new THREE.RingGeometry(size * 2, size * 2.3, 32),
      new THREE.MeshBasicMaterial({ color, transparent: true, opacity: 0.8, side: THREE.DoubleSide })
    );
    pulse.position.copy(pos);
    pulse.lookAt(0, 0, 0);
    scene.add(pulse);
    siteMeshes.push({ mesh: pulse, pulse: true, basePos: pos.clone(), level: site.level });
  }
}

// --- Default fallback sites (used if API returns nothing) ---
const defaultSites = [
  { lat: 38.71, lng: -9.14, level: 'active',   label: 'Lisbon Node' },
  { lat: 41.15, lng: -8.61, level: 'active',   label: 'Porto Grid' },
  { lat: 40.41, lng: -3.70, level: 'elevated', label: 'Madrid Cluster' },
  { lat: 48.85, lng:  2.35, level: 'healthy',  label: 'Paris Node' },
  { lat: 51.50, lng: -0.12, level: 'active',   label: 'London Ops' },
  { lat: 52.52, lng: 13.40, level: 'healthy',  label: 'Berlin Hub' },
  { lat: 40.63, lng:-73.93, level: 'healthy',  label: 'NY East' },
  { lat: 35.68, lng:139.69, level: 'healthy',  label: 'Tokyo Main' },
];

defaultSites.forEach(addSiteToGlobe);

// --- Arc connections ---
function createArc(startLat, startLng, endLat, endLng, color) {
  const start = latLngToVec3(startLat, startLng, 1.51);
  const end   = latLngToVec3(endLat,   endLng,   1.51);
  const mid   = start.clone().add(end).multiplyScalar(0.5);
  mid.normalize().multiplyScalar(mid.length() + 0.35);
  const curve = new THREE.QuadraticBezierCurve3(start, mid, end);
  const geo   = new THREE.BufferGeometry().setFromPoints(curve.getPoints(60));
  return new THREE.Line(geo, new THREE.LineBasicMaterial({ color, transparent: true, opacity: 0.5 }));
}

scene.add(createArc(38.71, -9.14, 40.41, -3.70, 0x4FE0D0));
scene.add(createArc(40.41, -3.70, 48.85,  2.35, 0xF1C96B));
scene.add(createArc(51.50, -0.12, 48.85,  2.35, 0x4FE0D0));

// --- Orbital ring + satellite ---
(function() {
  const orbit = new THREE.Mesh(
    new THREE.RingGeometry(2.0, 2.003, 128),
    new THREE.MeshBasicMaterial({ color: 0x9B8CFF, transparent: true, opacity: 0.25, side: THREE.DoubleSide })
  );
  orbit.rotation.x = Math.PI * 0.55;
  orbit.rotation.y = Math.PI * 0.15;
  scene.add(orbit);

  const satellite = new THREE.Mesh(
    new THREE.SphereGeometry(0.018, 8, 8),
    new THREE.MeshBasicMaterial({ color: 0x9B8CFF })
  );
  scene.add(satellite);
  window.__satellite = { mesh: satellite, orbit };
})();

// --- Particle field (stars) ---
(function() {
  const count     = 400;
  const geometry  = new THREE.BufferGeometry();
  const positions = new Float32Array(count * 3);
  for (let i = 0; i < count; i++) {
    const r     = 8 + Math.random() * 4;
    const theta = Math.random() * Math.PI * 2;
    const phi   = Math.acos(2 * Math.random() - 1);
    positions[i*3]   = r * Math.sin(phi) * Math.cos(theta);
    positions[i*3+1] = r * Math.sin(phi) * Math.sin(theta);
    positions[i*3+2] = r * Math.cos(phi);
  }
  geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3));
  scene.add(new THREE.Points(geometry, new THREE.PointsMaterial({
    color: 0x9EB3C8, size: 0.025, transparent: true, opacity: 0.6
  })));
})();

// --- Animation loop ---
const clock = new THREE.Clock();
let frames = 0, lastFpsUpdate = 0;

function animate() {
  requestAnimationFrame(animate);
  const t = clock.getElapsedTime();

  globe.rotation.y       += 0.0008;
  wireframe.rotation.y   += 0.0008;
  landmassDots.rotation.y += 0.0008;

  siteMeshes.forEach(s => {
    if (s.pulse) {
      const scale = 1 + Math.sin(t * 2.5 + (s.basePos ? s.basePos.x * 10 : 0)) * 0.35;
      s.mesh.scale.set(scale, scale, scale);
      s.mesh.material.opacity = 0.6 - Math.abs(Math.sin(t * 2.5)) * 0.4;
    }
  });

  if (window.__satellite) {
    const r     = 2.0;
    const angle = t * 0.12;
    const x = Math.cos(angle) * r;
    const y = Math.sin(angle) * r * 0.3;
    const z = Math.sin(angle) * r;
    window.__satellite.mesh.position.set(x, y, z);
    window.__satellite.mesh.position.applyAxisAngle(new THREE.Vector3(1, 0, 0), Math.PI * 0.55);
    window.__satellite.mesh.position.applyAxisAngle(new THREE.Vector3(0, 1, 0), Math.PI * 0.15);
  }

  camera.position.x = Math.sin(t * 0.1) * 0.08;
  camera.position.y = 0.2 + Math.cos(t * 0.13) * 0.04;
  camera.lookAt(0, 0, 0);

  renderer.render(scene, camera);

  frames++;
  if (t - lastFpsUpdate > 1) {
    const fpsel = document.getElementById('fps');
    if (fpsel) fpsel.textContent = frames;
    frames = 0;
    lastFpsUpdate = t;
  }
}
animate();

// --- Resize handling ---
window.addEventListener('resize', () => {
  camera.aspect = container.clientWidth / container.clientHeight;
  camera.updateProjectionMatrix();
  renderer.setSize(container.clientWidth, container.clientHeight);
});

// --- Mouse parallax ---
let targetRotX = 0, targetRotY = 0;
container.addEventListener('mousemove', e => {
  const rect = container.getBoundingClientRect();
  targetRotY = (e.clientX - rect.left) / rect.width  - 0.5;
  targetRotX = -((e.clientY - rect.top)  / rect.height - 0.5);
});

(function applyParallax() {
  requestAnimationFrame(applyParallax);
  camera.position.x += (targetRotY * 0.3 - camera.position.x) * 0.05;
  camera.position.y += (0.2 + targetRotX * 0.3 - camera.position.y) * 0.05;
})();

// --- Chip interactions ---
document.querySelectorAll('.chip-group').forEach(group => {
  group.querySelectorAll('.chip').forEach(chip => {
    chip.addEventListener('click', () => {
      group.querySelectorAll('.chip').forEach(c => c.classList.remove('active'));
      chip.classList.add('active');
    });
  });
});

// --- Legend toggles ---
document.querySelectorAll('.legend-toggle').forEach(t => {
  t.addEventListener('click', () => t.classList.toggle('on'));
});

// --- Simulated telemetry drift ---
setInterval(() => {
  const lat = (38.7139 + (Math.random() - 0.5) * 0.0004).toFixed(4);
  const lon = (-9.1400 + (Math.random() - 0.5) * 0.0004).toFixed(4);
  const latEl = document.getElementById('tel-lat');
  const lonEl = document.getElementById('tel-lon');
  if (latEl) latEl.textContent = lat + '\u00b0N';
  if (lonEl) lonEl.textContent = lon + '\u00b0';
}, 2000);

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
    const data   = await API.get('/v1/passive/canonical-events');
    const events = data.events || [];
    const list   = document.getElementById('what-changed-list');
    const count  = document.getElementById('changes-count');
    if (!list) return;
    if (!events.length) {
      list.innerHTML = '<div style="font-family:var(--font-mono);font-size:10px;color:var(--text-tertiary);padding:8px 0;">No recent events.</div>';
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
    if (data.summary) { narEl.textContent = data.summary; narEl.style.color = ''; }
    if (metaEl) {
      const attn  = (data.attention_queue || []).length;
      const maint = data.maintenance || {};
      const acts  = (maint.suggested_actions || []).length;
      metaEl.innerHTML =
        '<span>QUEUE \u00b7 ' + attn + '</span>' +
        '<span>STALE \u00b7 ' + (maint.stale_heartbeat_count || 0) + '</span>' +
        '<span>ACTIONS \u00b7 ' + acts + '</span>';
    }
  } catch (_) { /* leave */ }
}

// --- Source Health ---
// GET /v1/passive/operational-visibility → OperationalVisibilitySummary
// regions: [{ name, state: 'healthy'|'pressured'|'degraded'|'failing' }]
async function refreshSourceHealth() {
  try {
    const data    = await API.get('/v1/passive/operational-visibility');
    const grid    = document.getElementById('source-health-list');
    const cnt     = document.getElementById('source-count');
    const sysEl   = document.getElementById('sys-sources');
    if (!grid) return;
    const regions = data.regions || [];
    if (!regions.length) {
      grid.innerHTML = '<div style="font-family:var(--font-mono);font-size:10px;color:var(--text-tertiary);padding:8px 0;">No regions configured.</div>';
      if (cnt) cnt.textContent = '\u2014';
      if (sysEl) sysEl.textContent = 'no regions';
      return;
    }
    const score = { healthy: 97, pressured: 62, degraded: 35, failing: 14 };
    const cls   = { healthy: 'healthy', pressured: 'degraded', degraded: 'degraded', failing: 'failing' };
    const nHealthy = regions.filter(r => r.state === 'healthy').length;
    if (cnt)   cnt.textContent  = nHealthy + ' / ' + regions.length;
    if (sysEl) sysEl.textContent = nHealthy + ' / ' + regions.length + ' regions';
    grid.innerHTML = regions.map(r => {
      const sc = score[r.state] || 50;
      const cl = cls[r.state]   || 'degraded';
      const nm = (r.name || r.region_id || 'REGION').toUpperCase().substring(0, 14);
      return '<div class="source-row">'
        + '<div class="source-name">' + nm + '</div>'
        + '<div class="source-bar"><div class="source-bar-fill ' + cl + '" style="width:' + sc + '%"></div></div>'
        + '<div class="source-value">' + sc + '</div>'
        + '</div>';
    }).join('');
  } catch (_) {
    const grid = document.getElementById('source-health-list');
    if (grid) grid.innerHTML = '<div style="font-family:var(--font-mono);font-size:10px;color:var(--text-tertiary);padding:8px 0;">Visibility data unavailable.</div>';
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
//   RegionMapItem: { region_id, name, enabled, seeds_known, seeds_elevated, max_priority }
// GET /v1/passive/map/sites?region_id=X → SiteMapResponse { sites: SiteMapItem[] }
//   SiteMapItem: { coordinates:{lat,lon}, elevated, scan_priority, name, seed_key }
// GET /v1/passive/canonical-events → CanonicalEventsResponse { events: [...] }
let globeSitesLoaded = false;
async function refreshOpPicture() {
  try {
    const [regData, evData] = await Promise.all([
      API.get('/v1/passive/map/regions'),
      API.get('/v1/passive/canonical-events'),
    ]);
    const regions = regData.regions || [];
    const events  = evData.events   || [];

    // Fetch sites for each enabled region (cap at 3 regions)
    let allSites = [];
    for (const r of regions.filter(r => r.enabled).slice(0, 3)) {
      try {
        const sd = await API.get('/v1/passive/map/sites?region_id=' + encodeURIComponent(r.region_id));
        allSites = allSites.concat(sd.sites || []);
      } catch (_) {}
    }

    const activeRegions = regions.filter(r => r.enabled && (r.seeds_known > 0 || r.seeds_elevated > 0)).length
                        || regions.filter(r => r.enabled).length;
    const elevSites  = allSites.filter(s => s.elevated || s.scan_priority === 'high' || s.scan_priority === 'critical').length;
    const liveEvents = events.length;

    const opR   = document.getElementById('op-regions');
    const opS   = document.getElementById('op-sites');
    const opEv  = document.getElementById('op-events');
    const opMsg = document.getElementById('op-message');
    if (opR)  opR.textContent  = activeRegions;
    if (opS)  opS.textContent  = elevSites;
    if (opEv) opEv.textContent = liveEvents;
    if (opMsg) {
      if (!regions.length) {
        opMsg.innerHTML = 'No regions configured. Add regions to begin monitoring.';
      } else if (!liveEvents) {
        opMsg.innerHTML = regions.length + ' region' + (regions.length === 1 ? '' : 's') + ' monitored \u00b7 awaiting scan results.';
      } else {
        opMsg.innerHTML = '<strong>' + elevSites + ' site' + (elevSites === 1 ? '' : 's') + '</strong> elevated \u00b7 '
          + liveEvents + ' event' + (liveEvents === 1 ? '' : 's') + ' live across '
          + regions.length + ' region' + (regions.length === 1 ? '' : 's') + '.';
      }
    }

    // Add real sites to globe once
    if (!globeSitesLoaded && allSites.length > 0) {
      globeSitesLoaded = true;
      allSites.forEach(s => {
        if (s.coordinates) {
          const p     = s.scan_priority || 'low';
          const level = s.elevated ? (p === 'critical' ? 'critical' : 'elevated')
                      : p === 'high' ? 'elevated' : 'active';
          addSiteToGlobe({ lat: s.coordinates.lat, lng: s.coordinates.lon, level, label: s.name || s.seed_key || 'Site' });
        }
      });
    }
  } catch (_) { /* leave defaults */ }
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
    const items = data.attention_queue || [];
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
    list.innerHTML = items.slice(0, 5).map((item, idx) => {
      const pc  = pClass(idx);
      const km  = kindMap[item.kind] || { cls: 'evidence', label: String(item.kind || 'Item') };
      const num = String(idx + 1).padStart(2, '0');
      const lbl = item.primary_action_label || 'Open \u2192';
      return '<div class="attention-item">'
        + '<div class="priority-number ' + pc + '">' + num + '</div>'
        + '<div class="att-body">'
        + '<div class="att-head"><span class="att-kind ' + km.cls + '">' + km.label + '</span></div>'
        + '<div class="att-title">' + (item.title || 'Unnamed') + '</div>'
        + (item.reason ? '<div class="att-reason">' + item.reason + '</div>' : '')
        + '<button class="att-action">' + lbl + '</button>'
        + '</div></div>';
    }).join('');
  } catch (_) { /* leave */ }
}

// --- Recommended Actions ---
// maintenance.suggested_actions: [{ action_id, title, reason, method, path }]
async function refreshRecommendedActions() {
  try {
    const data    = await API.get('/v1/passive/command-center/summary');
    const list    = document.getElementById('recommended-actions-list');
    if (!list) return;
    const actions = (data.maintenance || {}).suggested_actions || [];
    if (!actions.length) {
      list.innerHTML = '<div style="font-family:var(--font-mono);font-size:10px;color:var(--text-tertiary);padding:6px 0;">No recommendations at this time.</div>';
      return;
    }
    const colors = ['var(--coral)', 'var(--amber)', 'var(--teal)', 'var(--lime)'];
    list.innerHTML = actions.slice(0, 4).map((a, i) => {
      const color = colors[i % colors.length];
      return '<div style="padding:11px 12px;background:var(--bg-elevated);border-left:2px solid ' + color + ';border-radius:0 2px 2px 0;">'
        + '<div style="font-size:12px;color:var(--text-primary);line-height:1.4;">' + (a.title || 'Action required') + '</div>'
        + (a.reason ? '<div style="font-size:11px;color:var(--text-secondary);margin-top:3px;line-height:1.4;">' + a.reason + '</div>' : '')
        + '<div style="font-family:var(--font-mono);font-size:9px;color:var(--text-tertiary);margin-top:6px;letter-spacing:0.08em;">'
        + (a.method || 'POST') + ' ' + (a.path || '') + '</div></div>';
    }).join('');
  } catch (_) { /* leave */ }
}

// --- Event Composition ---
// CanonicalPassiveEvent: { event_type, severity, priority }
async function refreshEventComposition() {
  try {
    const data   = await API.get('/v1/passive/canonical-events');
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

// --- Sys status ---
function refreshSysStatus() {
  const upd = document.getElementById('sys-updated');
  if (upd) upd.textContent = 'just now';
}

// --- Timeline labels ---
function updateTimelineLabels() {
  const now   = new Date();
  const start = new Date(now.getTime() - 36 * 3600 * 1000);
  const end   = new Date(now.getTime() + 36 * 3600 * 1000);
  const pad   = n => String(n).padStart(2, '0');
  const fmtTime = d => pad(d.getUTCHours()) + ':' + pad(d.getUTCMinutes()) + ' UTC';
  const months  = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'];
  const fmtDate = d => months[d.getUTCMonth()] + ' ' + d.getUTCDate() + ', ' + d.getUTCFullYear();
  const el = id => document.getElementById(id);
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
}

// Initial load + 30s polling
refreshAll();
setInterval(refreshAll, 30000);
</script>

</body>
</html>
"##;

const LANDING_PAGE_HTML: &str = r##"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>ShieldSky</title>
    <style>
      :root {
        color-scheme: dark;
        --bg: #08111d;
        --panel: rgba(14, 24, 39, 0.92);
        --panel-strong: rgba(20, 34, 56, 0.96);
        --line: rgba(151, 181, 214, 0.18);
        --text: #eff5ff;
        --muted: #a8bad2;
        --accent: #53d1c1;
        --accent-2: #f3c969;
      }
      * { box-sizing: border-box; }
      body {
        margin: 0;
        font-family: "Segoe UI", Arial, sans-serif;
        color: var(--text);
        background:
          radial-gradient(circle at top left, rgba(83, 209, 193, 0.14), transparent 24%),
          radial-gradient(circle at 85% 15%, rgba(243, 201, 105, 0.12), transparent 18%),
          linear-gradient(180deg, #08111d 0%, #0b1524 52%, #09111d 100%);
      }
      a { color: inherit; text-decoration: none; }
      .shell {
        min-height: 100vh;
        padding: 40px 24px 56px;
        max-width: 1180px;
        margin: 0 auto;
      }
      .nav {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 16px;
        margin-bottom: 36px;
      }
      .brand {
        display: flex;
        align-items: center;
        gap: 12px;
        font-weight: 700;
        letter-spacing: 0.04em;
      }
      .brand-mark {
        width: 14px;
        height: 14px;
        border-radius: 999px;
        background: linear-gradient(135deg, var(--accent), #7bd4ff);
        box-shadow: 0 0 20px rgba(83, 209, 193, 0.45);
      }
      .nav-links {
        display: flex;
        gap: 12px;
        flex-wrap: wrap;
      }
      .nav-links a,
      .cta,
      .cta-secondary {
        border: 1px solid var(--line);
        border-radius: 999px;
        padding: 10px 14px;
        font-size: 14px;
      }
      .cta {
        background: linear-gradient(180deg, rgba(83, 209, 193, 0.24), rgba(83, 209, 193, 0.08));
      }
      .cta-secondary {
        background: rgba(255,255,255,0.03);
      }
      .hero {
        display: grid;
        grid-template-columns: 1.2fr 0.8fr;
        gap: 20px;
        margin-bottom: 20px;
      }
      .panel {
        border: 1px solid var(--line);
        border-radius: 18px;
        background: var(--panel);
        padding: 24px;
        backdrop-filter: blur(10px);
      }
      h1 {
        margin: 0 0 14px;
        font-size: clamp(34px, 6vw, 64px);
        line-height: 0.98;
      }
      .eyebrow {
        color: var(--accent);
        font-size: 13px;
        letter-spacing: 0.14em;
        text-transform: uppercase;
        margin-bottom: 14px;
      }
      .lede {
        margin: 0;
        color: var(--muted);
        font-size: 17px;
        line-height: 1.65;
        max-width: 60ch;
      }
      .hero-actions {
        display: flex;
        gap: 12px;
        flex-wrap: wrap;
        margin-top: 22px;
      }
      .stat-grid,
      .feature-grid {
        display: grid;
        gap: 16px;
      }
      .stat-grid {
        grid-template-columns: repeat(3, 1fr);
        margin: 20px 0 28px;
      }
      .stat {
        border: 1px solid var(--line);
        border-radius: 14px;
        padding: 16px;
        background: rgba(255,255,255,0.025);
      }
      .stat strong {
        display: block;
        font-size: 26px;
        margin-bottom: 6px;
      }
      .stat span,
      .feature p,
      .note,
      .mini-list li {
        color: var(--muted);
      }
      .feature-grid {
        grid-template-columns: repeat(3, 1fr);
      }
      .feature {
        border: 1px solid var(--line);
        border-radius: 16px;
        padding: 18px;
        background: rgba(255,255,255,0.02);
      }
      .feature h3,
      .side-panel h2,
      .panel h2 {
        margin: 0 0 10px;
        font-size: 18px;
      }
      .side-panel {
        display: grid;
        gap: 14px;
      }
      .mini-list {
        margin: 0;
        padding-left: 18px;
        display: grid;
        gap: 8px;
      }
      code {
        font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
        color: #d9ebff;
      }
      .note {
        margin-top: 18px;
        font-size: 14px;
        line-height: 1.6;
      }
      @media (max-width: 900px) {
        .hero,
        .feature-grid,
        .stat-grid {
          grid-template-columns: 1fr;
        }
      }
    </style>
  </head>
  <body>
    <main class="shell">
      <div class="nav">
        <div class="brand"><span class="brand-mark"></span><span>ShieldSky</span></div>
        <div class="nav-links">
          <a href="/console">Operator Console</a>
          <a href="/v1/health">Health</a>
          <a href="/v1/version">Version</a>
        </div>
      </div>

      <section class="hero">
        <div class="panel">
          <div class="eyebrow">Sky Security System</div>
          <h1>Operational airspace intelligence for monitored sites.</h1>
          <p class="lede">ShieldSky combines passive sensing, orbital context, canonical event detection, and response workflows into one deployable Rust service built for monitoring, triage, and evidence-backed action.</p>
          <div class="hero-actions">
            <a class="cta" href="/console">Open Operator Console</a>
            <a class="cta-secondary" href="/v1/health">API Health</a>
          </div>
          <div class="stat-grid">
            <div class="stat"><strong>Passive</strong><span>Site and region surveillance from fused sources.</span></div>
            <div class="stat"><strong>Canonical</strong><span>Event normalization, semantic timelines, and remediation surfaces.</span></div>
            <div class="stat"><strong>Deployable</strong><span>Rust API running in Render with health endpoints and operator UI.</span></div>
          </div>
        </div>
        <div class="panel side-panel">
          <div>
            <h2>Primary surfaces</h2>
            <ul class="mini-list">
              <li><a href="/console"><code>/console</code></a> for the live operator interface</li>
              <li><a href="/v1/briefing/apod"><code>/v1/briefing/apod</code></a> for NASA APOD briefing</li>
              <li><a href="/v1/briefing/neows"><code>/v1/briefing/neows</code></a> for NEO risk briefing</li>
              <li><a href="/v1/passive/dashboard/summary"><code>/v1/passive/dashboard/summary</code></a> for passive operations summary</li>
            </ul>
          </div>
          <div>
            <h2>Deployment profile</h2>
            <ul class="mini-list">
              <li>Render-ready Docker deployment</li>
              <li><code>/health</code> and <code>/v1/health</code> health endpoints</li>
              <li>SQLite-backed operational storage</li>
            </ul>
          </div>
          <p class="note">This root page is intentionally concise. The operational UI lives in the console and the API remains directly accessible for integrations and dashboards.</p>
        </div>
      </section>

      <section class="feature-grid">
        <article class="feature">
          <h3>Passive Region Monitoring</h3>
          <p>Track monitored regions, mapped sites, risk history, maintenance state, and source-health evidence across the passive surveillance pipeline.</p>
        </article>
        <article class="feature">
          <h3>Event and Timeline Workflows</h3>
          <p>Canonical events, semantic timelines, and operational timelines surface what changed, why it matters, and which action path follows.</p>
        </article>
        <article class="feature">
          <h3>Briefings and Context</h3>
          <p>APOD and NeoWs briefings provide external context while the core ShieldSky console focuses on operational site protection and decision support.</p>
        </article>
      </section>
    </main>
  </body>
</html>
"##;
