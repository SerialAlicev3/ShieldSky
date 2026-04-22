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
  .focus-divider { border: none; border-top: 1px solid var(--line); margin: 10px 0; }
  .focus-actions { display: flex; gap: 8px; }

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
      <hr class="focus-divider">
      <div class="focus-actions">
        <button class="att-action" id="focus-primary-btn">Open &rarr;</button>
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
    kind:   (props.kind && props.kind.getValue()) || 'site',
    title:  (props.label && props.label.getValue()) || (ent.name) || 'Unknown',
    reason: (props.reason && props.reason.getValue()) || '',
    lat, lng,
  });
}, Cesium.ScreenSpaceEventType.LEFT_CLICK);

document.getElementById('focus-close-btn').addEventListener('click', () => {
  document.getElementById('focus-panel').classList.remove('visible');
  _focusCoords = { lat: null, lng: null };
});

document.getElementById('focus-primary-btn').addEventListener('click', () => {
  if (_focusCoords.lat != null) {
    focusOnGlobe(_focusCoords.lat, _focusCoords.lng, 180000);
  }
});

// --- Globe data cache + focus system ---
const GLOBE_DATA = { regions: [], events: [] };
let lastAttentionItems = [];
let _focusCoords = { lat: null, lng: null };
const PRESSURE_HISTORY = []; // circular buffer { t, score }

function focusOnGlobe(lat, lng, altM) {
  viewer.camera.flyTo({
    destination: Cesium.Cartesian3.fromDegrees(lng, lat, altM || 450000),
    orientation: { heading: 0, pitch: Cesium.Math.toRadians(-50), roll: 0 },
    duration: 1.5,
  });
}

function openFocusPanel({ kind, title, reason, lat, lng }) {
  const panel   = document.getElementById('focus-panel');
  const fKind   = document.getElementById('focus-kind');
  const fTitle  = document.getElementById('focus-title');
  const fReason = document.getElementById('focus-reason');
  const fCoords = document.getElementById('focus-coords');
  if (!panel) return;
  const kindLabel = { site: 'SITE', region: 'REGION', canonical_event: 'EVENT', neo: 'NEO', maintenance: 'MAINTENANCE' }[kind] || (kind || 'SITE').toUpperCase();
  const kindCls   = kind === 'region' ? 'region' : kind === 'canonical_event' ? 'event' : '';
  fKind.textContent   = kindLabel;
  fKind.className     = 'focus-kind-badge' + (kindCls ? ' ' + kindCls : '');
  fTitle.textContent  = title || '\u2014';
  fReason.textContent = reason || '';
  fCoords.textContent = lat != null ? lat.toFixed(4) + '\u00b0N \u00b7 ' + lng.toFixed(4) + '\u00b0' : '';
  _focusCoords = { lat, lng };
  panel.classList.add('visible');
  if (lat != null) focusOnGlobe(lat, lng);
}

// --- Console state (filter + time window) ---
const CONSOLE_STATE = { filter: 'global', window: '72h' };
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
        openFocusPanel({ kind: 'region', title: r.name || r.region_id, reason: (r.seeds_elevated || 0) + ' seeds elevated \u00b7 ' + (r.seeds_known || 0) + ' indexed', lat: lat, lng: lng });
        input.value = '';
        return;
      }
      const ev = GLOBE_DATA.events.find(function(e) { return ((e.site_name || '') + ' ' + (e.event_type || '') + ' ' + (e.summary || '')).toLowerCase().includes(q); });
      if (ev && ev.coordinates) {
        openFocusPanel({ kind: 'canonical_event', title: ev.site_name || ev.event_type || 'Event', reason: ev.summary || ev.operational_readout || '', lat: ev.coordinates.lat, lng: ev.coordinates.lon });
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
          + ' \u00b7 awaiting scan results.';
      } else {
        opMsg.innerHTML = '<strong>' + elevSites + ' site' + (elevSites === 1 ? '' : 's') + '</strong> elevated \u00b7 '
          + liveEvents + ' event' + (liveEvents === 1 ? '' : 's') + ' live across '
          + activeRegions + ' region' + (activeRegions === 1 ? '' : 's') + '.';
      }
    }

    // Store in global cache for focus lookups
    GLOBE_DATA.regions = enabledRegions;
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
          properties: { kind: 'region', label: r.name || r.region_id, region_id: r.region_id, reason: (r.seeds_elevated || 0) + ' seeds elevated · ' + (r.seeds_known || 0) + ' indexed' },
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
          properties: { kind: 'canonical_event', label: ev.site_name || ev.event_type || 'Event', reason: ev.summary || ev.operational_readout || '' },
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
      return '<div class="attention-item">'
        + '<div class="priority-number ' + pc + '">' + num + '</div>'
        + '<div class="att-body">'
        + '<div class="att-head"><span class="att-kind ' + km.cls + '">' + km.label + '</span></div>'
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
    // Sites have no separate cache — search events for a coordinate match
    const ev = GLOBE_DATA.events.find(e => (e.site_name || '').toLowerCase() === item.title.toLowerCase());
    if (ev && ev.coordinates) { lat = ev.coordinates.lat; lng = ev.coordinates.lon; }
  } else if (item.kind === 'canonical_event' || item.kind === 'neo') {
    const ev = GLOBE_DATA.events.find(e =>
      (e.summary || e.event_type || '').toLowerCase().includes(item.title.toLowerCase()) ||
      (e.site_name || '').toLowerCase().includes(item.title.toLowerCase()));
    if (ev && ev.coordinates) { lat = ev.coordinates.lat; lng = ev.coordinates.lon; }
    // No dangerous fallback — lat stays null, panel opens without fly-to
  }
  openFocusPanel({ kind: item.kind, title: item.title, reason: item.reason, lat, lng });
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
      list.innerHTML = '<div style="font-family:var(--font-mono);font-size:10px;color:var(--text-tertiary);padding:6px 0;line-height:1.6;">No operator actions required in the current window. Monitoring continues.</div>';
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
function refreshTimeline(events) {
  const strip = document.getElementById('timeline-events');
  if (!strip) return;
  if (!events || !events.length) { strip.innerHTML = ''; return; }
  const now   = Date.now();
  const start = now - 36 * 3600 * 1000;
  const total = 72 * 3600 * 1000;
  const colorMap = { critical: 'var(--coral)', elevated: 'var(--amber)', active: 'var(--teal)', resolved: 'var(--lime)' };
  strip.innerHTML = events
    .filter(function(ev) { return ev.last_observed_at_unix_seconds; })
    .map(function(ev) {
      const pct   = Math.max(2, Math.min(97, ((ev.last_observed_at_unix_seconds * 1000 - start) / total) * 100));
      const cls   = sevCls(ev.severity);
      const color = colorMap[cls] || 'var(--teal)';
      const label = (ev.site_name || ev.event_type || 'event').replace(/_/g, ' ').toUpperCase();
      const reason = (ev.summary || '').replace(/'/g, ' ').substring(0, 80);
      const lat   = ev.coordinates ? ev.coordinates.lat : null;
      const lng   = ev.coordinates ? ev.coordinates.lon : null;
      return '<div class="timeline-event" '
        + 'style="left:' + pct.toFixed(1) + '%;background:' + color + ';cursor:pointer;width:10px;height:10px;border:2px solid rgba(255,255,255,0.25);" '
        + 'title="' + label + '" '
        + 'onclick="openFocusPanel({kind:\'canonical_event\',title:\'' + label.replace(/'/g, '') + '\',reason:\'' + reason + '\',lat:' + lat + ',lng:' + lng + '})"></div>';
    }).join('');
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

const LANDING_PAGE_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>ShieldSky â€” Operational Intelligence</title>
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

/* â”€â”€â”€ NOISE TEXTURE OVERLAY â”€â”€â”€ */
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

/* â”€â”€â”€ SCANLINE â”€â”€â”€ */
body::after {
  content: '';
  position: fixed;
  inset: 0;
  background: repeating-linear-gradient(0deg, transparent, transparent 2px, rgba(0,0,0,0.03) 2px, rgba(0,0,0,0.03) 4px);
  pointer-events: none;
  z-index: 999;
}

/* â”€â”€â”€ GRID OVERLAY â”€â”€â”€ */
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

/* â”€â”€â”€ NAV â”€â”€â”€ */
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

/* â”€â”€â”€ HERO â”€â”€â”€ */
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

/* â”€â”€â”€ SECTIONS â”€â”€â”€ */
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

/* â”€â”€â”€ SURFACES SECTION â”€â”€â”€ */
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

/* â”€â”€â”€ THINKING SECTION â”€â”€â”€ */
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

/* â”€â”€â”€ WHY MATTERS â”€â”€â”€ */
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

/* â”€â”€â”€ OPERATORS SECTION â”€â”€â”€ */
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

/* â”€â”€â”€ CTA SECTION â”€â”€â”€ */
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

/* â”€â”€â”€ FOOTER â”€â”€â”€ */
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

/* â”€â”€â”€ SCROLL REVEAL â”€â”€â”€ */
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

/* â”€â”€â”€ RESPONSIVE â”€â”€â”€ */
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
    <svg width="28" height="28" viewBox="0 0 28 28" fill="none" xmlns="http://www.w3.org/2000/svg" style="flex-shrink:0"><defs><filter id="gl"><feGaussianBlur stdDeviation="1.5" result="blur"/><feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge></filter></defs><path d="M14 2L3 6.5V14c0 5.8 4.5 11.2 11 13 6.5-1.8 11-7.2 11-13V6.5L14 2z" fill="rgba(0,212,184,0.10)" stroke="#00d4b8" stroke-width="1.1" filter="url(#gl)"/><circle cx="14" cy="13" r="3.5" stroke="#00d4b8" stroke-width="0.7" opacity="0.55"/><circle cx="14" cy="13" r="6" stroke="#00d4b8" stroke-width="0.4" opacity="0.3"/><line x1="14" y1="8.5" x2="14" y2="17.5" stroke="#00d4b8" stroke-width="0.6" opacity="0.6"/><line x1="9.5" y1="13" x2="18.5" y2="13" stroke="#00d4b8" stroke-width="0.6" opacity="0.6"/><circle cx="14" cy="13" r="1.4" fill="#00d4b8"/><circle cx="14" cy="13" r="1.4" fill="#00d4b8" filter="url(#gl)" opacity="0.8"/></svg>
    <span class="nav-logo-text">ShieldSky</span>
  </a>
  <div class="nav-links">
    <a href="/health" class="nav-link">Health</a>
    <a href="/v1/health" class="nav-link">API</a>
    <a href="/console" class="nav-link nav-cta">Operator Console â†’</a>
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
      The sky<br>
      <em>doesn't</em><br>
      hide.
    </h1>
    <p class="hero-subline">
      ShieldSky transforms <strong>passive signals, orbital context, and environmental anomalies</strong> into prioritized intelligence, narrative, and decision surfaces for monitored regions and critical sites.
    </p>
    <div class="hero-actions">
      <a href="/console" class="btn-primary">
        <svg viewBox="0 0 16 16" fill="currentColor"><path d="M3 8h10M9 4l4 4-4 4"/></svg>
        Enter Command Center
      </a>
      <a href="/v1/health" class="btn-secondary">API Surface â†’</a>
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
      <span class="stat-value">âˆž</span>
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
        Not charts. Not dashboards. A living intelligence surface â€” attention queue, narrative context, source health, and evidence â€” assembled in real time.
      </p>
    </div>

    <div class="surfaces-grid reveal">
      <!-- Large card: Command Center -->
      <div class="surface-card large">
        <div class="surface-card-tag">Primary Surface</div>
        <div class="surface-card-title">Operator Command Center</div>
        <p class="surface-card-desc">The full intelligence surface. Attention queue, narrative briefings, site status, anomaly detection, and evidence replay â€” unified in a single operational view.</p>
        <div class="surface-mock">
          <div class="mock-bar">
            <span class="mock-dot red"></span>
            <span class="mock-dot yellow"></span>
            <span class="mock-dot green"></span>
            <span class="mock-title">SHIELDSKY / CONSOLE</span>
          </div>
          <div class="mock-row"><span class="mock-row-label">SYSTEM STATUS</span><span class="mock-row-val">â— OPERATIONAL</span></div>
          <div class="mock-row"><span class="mock-row-label">ACTIVE REGIONS</span><span class="mock-row-val">7</span></div>
          <div class="mock-row"><span class="mock-row-label">MONITORED SITES</span><span class="mock-row-val">34</span></div>
          <div class="mock-row"><span class="mock-row-label">ATTENTION QUEUE</span><span class="mock-row-val amber">3 ITEMS</span></div>
          <div class="mock-row"><span class="mock-row-label">LAST INGEST</span><span class="mock-row-val dim">00:00:04 AGO</span></div>
          <div class="mock-alert"><strong>PRIORITY:</strong> Orbital anomaly detected over Grid Sector 7-C. Canonical event triggered. Evidence capture active.</div>
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
            <div class="mock-queue-text"><strong>ANOMALY â€” SECTOR 7C</strong><br>Orbital pass intersect with passive signal cluster</div>
            <span class="mock-queue-time">NOW</span>
          </div>
          <div class="mock-queue-item">
            <span class="mock-queue-dot med"></span>
            <div class="mock-queue-text"><strong>SITE ALPHA-03</strong><br>Environmental threshold exceeded</div>
            <span class="mock-queue-time">04m</span>
          </div>
          <div class="mock-queue-item">
            <span class="mock-queue-dot low"></span>
            <div class="mock-queue-text"><strong>NEO RISK BRIEF</strong><br>Updated orbital context available</div>
            <span class="mock-queue-time">12m</span>
          </div>
        </div>
      </div>

      <!-- Source Health -->
      <div class="surface-card">
        <div class="surface-card-tag">System Awareness</div>
        <div class="surface-card-title">Source Health</div>
        <p class="surface-card-desc">Signal quality, ingest latency, and source reliability â€” always visible.</p>
        <div class="surface-mock" style="margin-top:20px">
          <div class="mock-bar">
            <span class="mock-dot red"></span><span class="mock-dot yellow"></span><span class="mock-dot green"></span>
            <span class="mock-title">SOURCE / HEALTH</span>
          </div>
          <div class="mock-row"><span class="mock-row-label">PASSIVE / RF</span><span class="mock-row-val">â— 98.2%</span></div>
          <div class="mock-row"><span class="mock-row-label">ORBITAL CTX</span><span class="mock-row-val">â— 100%</span></div>
          <div class="mock-row"><span class="mock-row-label">ENV SENSORS</span><span class="mock-row-val amber">â— 87.1%</span></div>
          <div class="mock-row"><span class="mock-row-label">EVENT BUS</span><span class="mock-row-val">â— NOMINAL</span></div>
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
        <p class="thinking-step-desc">Continuous passive ingestion of regional signals, orbital passes, environmental data, and site anomalies â€” without interruption.</p>
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
        <p class="thinking-step-desc">Every event arrives with narrative context, evidence references, and confidence levels â€” not just an alert.</p>
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
        <p class="why-item-text">Alerts create noise. Narratives create understanding. Every anomaly arrives with context, trajectory, and recommended priority â€” already written.</p>
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
        <h2 class="section-title" style="font-size: clamp(40px,5vw,72px)">For those<br>who cannot<br>miss.</h2>
        <p class="operators-statement" style="font-size:17px; margin-top:28px">
          ShieldSky is built for <strong>infrastructure security teams, site protection officers, and regional intelligence operators</strong> who need answers before questions form.
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
            <div class="metric-num">âˆž</div>
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
              <p class="cap-desc">Define regions, set thresholds, receive structured intelligence â€” not raw sensor dumps.</p>
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
              <p class="cap-desc">Ground signals enriched with orbital passes and environmental data â€” automatically fused.</p>
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
    <div class="cta-tag">â€” System Ready</div>
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
  <div class="footer-links">
    <a href="/console" class="footer-link">Console</a>
    <a href="/health" class="footer-link">Health</a>
    <a href="/v1/health" class="footer-link">API</a>
    <a href="/v1/briefing/neows" class="footer-link">NEO Brief</a>
  </div>
  <div class="footer-meta">Operational Intelligence Layer Â· Always On</div>
</footer>

<script>
/* â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
   GLOBE CANVAS
â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â• */
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

/* â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
   CTA CANVAS â€” PARTICLE FIELD
â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â• */
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

/* â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
   SCROLL REVEAL
â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â• */
const reveals = document.querySelectorAll('.reveal');
const observer = new IntersectionObserver((entries) => {
  entries.forEach(e => { if (e.isIntersecting) { e.target.classList.add('visible'); } });
}, { threshold: 0.1, rootMargin: '0px 0px -40px 0px' });
reveals.forEach(el => observer.observe(el));

/* â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
   NAV SCROLL
â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â• */
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
