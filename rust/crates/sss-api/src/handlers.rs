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

const OPERATOR_CONSOLE_HTML: &str = r##"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>SSS Operator Console</title>
    <link href="https://cesium.com/downloads/cesiumjs/releases/1.127/Build/Cesium/Widgets/widgets.css" rel="stylesheet">
    <style>
    :root {
      color-scheme: dark;
      --bg: #08111d;
      --panel: rgba(12, 23, 38, 0.92);
      --panel-strong: rgba(18, 34, 56, 0.96);
      --line: rgba(151, 181, 214, 0.18);
      --text: #ecf3ff;
      --muted: #a7bbd6;
      --accent: #53d1c1;
      --accent-2: #f3c969;
      --danger: #ff8e7c;
      --ok: #87e39b;
    }

    * { box-sizing: border-box; }
    body {
      margin: 0;
      font-family: "Segoe UI", Arial, sans-serif;
      background:
        radial-gradient(circle at top left, rgba(83, 209, 193, 0.12), transparent 28%),
        radial-gradient(circle at top right, rgba(243, 201, 105, 0.12), transparent 22%),
        linear-gradient(180deg, #08111d, #0c1626 42%, #0a1320 100%);
      color: var(--text);
    }
    a { color: inherit; }
    .shell {
      min-height: 100vh;
      padding: 24px;
      display: grid;
      gap: 18px;
      background: linear-gradient(180deg, rgba(6, 10, 18, 0.15), rgba(6, 10, 18, 0.68));
    }
    .hero {
      display: grid;
      gap: 8px;
      padding: 8px 0 2px;
    }
    .globe-stage {
      min-height: 60vh;
      position: relative;
      border: 1px solid var(--line);
      border-radius: 8px;
      overflow: hidden;
      background: rgba(6, 12, 20, 0.72);
    }
    #cesium-globe {
      width: 100%;
      height: 60vh;
      min-height: 480px;
    }
    .globe-overlay {
      position: absolute;
      left: 16px;
      bottom: 16px;
      z-index: 2;
      max-width: min(480px, calc(100% - 32px));
      padding: 12px;
      border: 1px solid var(--line);
      border-radius: 8px;
      background: rgba(10, 18, 31, 0.84);
      backdrop-filter: blur(10px);
    }
    .globe-overlay h3 {
      margin: 0 0 6px 0;
      font-size: 14px;
    }
    .globe-overlay p {
      margin: 0;
      color: var(--muted);
      font-size: 12px;
      line-height: 1.5;
    }
    .globe-legend {
      position: absolute;
      right: 16px;
      top: 16px;
      z-index: 2;
      max-width: min(340px, calc(100% - 32px));
      padding: 12px;
      border: 1px solid var(--line);
      border-radius: 8px;
      background: rgba(10, 18, 31, 0.84);
      backdrop-filter: blur(10px);
      display: grid;
      gap: 10px;
    }
    .globe-legend h3 {
      margin: 0;
      font-size: 14px;
    }
    .legend-grid {
      display: grid;
      gap: 8px;
    }
    .legend-row {
      display: flex;
      gap: 8px;
      align-items: center;
      flex-wrap: wrap;
      color: var(--muted);
      font-size: 12px;
    }
    .legend-dot {
      width: 10px;
      height: 10px;
      border-radius: 999px;
      display: inline-block;
      border: 1px solid rgba(255,255,255,0.25);
      flex: 0 0 auto;
    }
    .hero h1 {
      margin: 0;
      font-size: 32px;
      line-height: 1.05;
      font-weight: 700;
    }
    .hero p {
      margin: 0;
      color: var(--muted);
      max-width: 880px;
      line-height: 1.5;
    }
    .toolbar, .grid, .detail-grid {
      display: grid;
      gap: 16px;
    }
    .toolbar {
      grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
      align-items: end;
    }
    .toggle {
      display: flex;
      align-items: center;
      gap: 10px;
      min-height: 42px;
      padding: 10px 12px;
      border: 1px solid var(--line);
      border-radius: 8px;
      background: rgba(255,255,255,0.03);
    }
    .toggle input {
      width: 16px;
      height: 16px;
      margin: 0;
    }
    .grid {
      grid-template-columns: 1.1fr 1.1fr 0.8fr;
    }
    .detail-grid {
      grid-template-columns: 1fr 1fr;
    }
    .panel {
      border: 1px solid var(--line);
      border-radius: 8px;
      background: var(--panel);
      backdrop-filter: blur(10px);
      overflow: hidden;
      min-height: 140px;
    }
    .panel-head {
      padding: 14px 16px;
      border-bottom: 1px solid var(--line);
      display: flex;
      justify-content: space-between;
      align-items: center;
      gap: 12px;
      background: var(--panel-strong);
    }
    .panel-head h2, .panel-head h3 {
      margin: 0;
      font-size: 15px;
      font-weight: 600;
    }
    .panel-body { padding: 16px; }
    .metrics {
      display: grid;
      grid-template-columns: repeat(4, minmax(0, 1fr));
      gap: 12px;
    }
    .metric {
      border: 1px solid var(--line);
      border-radius: 8px;
      padding: 14px;
      background: rgba(255,255,255,0.02);
      min-height: 92px;
    }
    .metric-label {
      color: var(--muted);
      font-size: 12px;
      text-transform: uppercase;
    }
    .metric-value {
      margin-top: 10px;
      font-size: 28px;
      font-weight: 700;
    }
    .row {
      display: flex;
      gap: 10px;
      align-items: center;
      flex-wrap: wrap;
    }
    label {
      display: grid;
      gap: 6px;
      font-size: 12px;
      color: var(--muted);
    }
    input, select, button {
      border-radius: 8px;
      border: 1px solid var(--line);
      background: rgba(7, 16, 28, 0.94);
      color: var(--text);
      padding: 10px 12px;
      font: inherit;
      min-height: 40px;
    }
    button {
      cursor: pointer;
      background: linear-gradient(180deg, rgba(83, 209, 193, 0.22), rgba(83, 209, 193, 0.08));
    }
    button.secondary {
      background: rgba(255,255,255,0.03);
    }
    .status {
      font-size: 12px;
      color: var(--muted);
    }
    .list {
      display: grid;
      gap: 10px;
    }
    .card {
      border: 1px solid var(--line);
      border-radius: 8px;
      padding: 12px;
      background: rgba(255,255,255,0.02);
    }
    .neo-briefing-item {
      cursor: pointer;
      transition: border-color 120ms ease, background 120ms ease;
    }
    .neo-briefing-item:hover {
      border-color: rgba(83, 209, 193, 0.28);
      background: rgba(83, 209, 193, 0.06);
    }
    .neo-briefing-item-selected {
      border-color: rgba(83, 209, 193, 0.44);
      background: rgba(83, 209, 193, 0.08);
    }
    .card h4 {
      margin: 0 0 6px 0;
      font-size: 14px;
    }
    .meta {
      display: flex;
      gap: 8px;
      flex-wrap: wrap;
      color: var(--muted);
      font-size: 12px;
    }
    .pill {
      display: inline-flex;
      align-items: center;
      border: 1px solid var(--line);
      border-radius: 999px;
      padding: 4px 8px;
      font-size: 11px;
      color: var(--text);
      background: rgba(255,255,255,0.04);
    }
    .pill.ok { color: var(--ok); }
    .pill.warn { color: var(--accent-2); }
    .pill.danger { color: var(--danger); }
    .mono {
      font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
      font-size: 12px;
    }
    pre {
      margin: 0;
      white-space: pre-wrap;
      word-break: break-word;
      font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
      font-size: 12px;
      color: #d8e6fb;
    }
    .empty {
      color: var(--muted);
      font-size: 13px;
      padding: 4px 0;
    }
    @keyframes stl-flash {
      0%   { background: rgba(255,255,100,0.18); }
      100% { background: transparent; }
    }
    .stl-updated { animation: stl-flash 0.7s ease-out; }
    .stl-empty {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 6px;
      padding: 24px 0;
      color: var(--muted);
      font-size: 13px;
    }
    .stl-empty-icon { font-size: 22px; opacity: 0.5; }
    .stl-entry-headline { font-weight: 600; font-size: 13px; margin-bottom: 2px; }
    .stl-entry-trend { display: flex; gap: 10px; align-items: center; margin: 3px 0; font-size: 12px; }
    .stl-entry-when { color: var(--muted); font-size: 11px; margin-top: 3px; }
    .stl-action-escalate { background: #e53935; color: #fff; font-weight: 700; }
    .stl-action-monitor  { background: #f9a825; color: #1a1a1a; font-weight: 700; }
    .stl-action-ignore   { background: #555; color: #ccc; }
    .stl-action-respond  { background: #7b1fa2; color: #fff; font-weight: 700; }
    .stl-highlight { box-shadow: 0 0 0 2px currentColor, 0 0 10px 1px rgba(229,57,53,0.25); border-radius: 5px; }
    .stl-badge { display: inline-block; font-size: 10px; font-weight: 800; letter-spacing: 0.05em; padding: 1px 6px; border-radius: 3px; margin-left: 6px; vertical-align: middle; text-transform: uppercase; }
    .stl-badge-spike  { background: #e53935; color: #fff; }
    .stl-badge-new    { background: #1565c0; color: #fff; }
    .stl-quick-actions { display: flex; gap: 6px; margin-top: 6px; flex-wrap: wrap; }
    .stl-quick-actions button { font-size: 11px; padding: 2px 8px; border-radius: 4px; cursor: pointer; border: 1px solid var(--line,#555); background: rgba(255,255,255,0.06); color: inherit; }
    .stl-quick-actions button:hover { background: rgba(255,255,255,0.14); }
    @media (max-width: 1100px) {
      .grid, .detail-grid { grid-template-columns: 1fr; }
      .metrics { grid-template-columns: repeat(2, minmax(0, 1fr)); }
    }
    @media (max-width: 700px) {
      .shell { padding: 16px; }
      .metrics { grid-template-columns: 1fr; }
      .hero h1 { font-size: 26px; }
    }
  </style>
</head>
<body>
  <main class="shell">
    <section class="hero">
      <h1>SSS Operator Console</h1>
      <p>ShieldSky fuses orbital context, passive sensing, and response routing into one operator surface for site protection, airspace monitoring, and evidence-backed decisions.</p>
    </section>

    <section class="globe-stage">
      <div id="cesium-globe"></div>
      <div class="globe-overlay">
        <h3>ShieldSky Operational Map</h3>
        <p>Regions, observed sites, canonical events, future checkpoints, and orbital context are coordinated here as one live operational picture.</p>
      </div>
      <div class="globe-legend" id="passive-map-legend"></div>
    </section>

    <section class="panel">
      <div class="panel-head">
        <h2>Mission Snapshot</h2>
        <div class="status" id="refresh-status">Refreshing...</div>
      </div>
      <div class="panel-body">
        <div class="metrics">
          <div class="metric"><div class="metric-label">Source Freshness</div><div class="metric-value" id="metric-freshness">-</div></div>
          <div class="metric"><div class="metric-label">Future Events</div><div class="metric-value" id="metric-events">-</div></div>
          <div class="metric"><div class="metric-label">Deliveries</div><div class="metric-value" id="metric-deliveries">-</div></div>
          <div class="metric"><div class="metric-label">Replay Drift</div><div class="metric-value" id="metric-drift">-</div></div>
        </div>
      </div>
    </section>

    <section class="panel">
      <div class="panel-head">
        <h2>Operator Controls</h2>
        <div class="row">
          <button id="refresh-all">Refresh</button>
          <button id="dispatch-alert" class="secondary">Dispatch Alert</button>
        </div>
      </div>
      <div class="panel-body toolbar">
        <label>Object
          <input id="object-id" value="SAT-001" />
        </label>
        <label>Timeline Horizon (h)
          <input id="timeline-horizon" type="number" min="1" max="168" value="72" />
        </label>
        <label>Window (h)
          <input id="overview-window" type="number" min="1" max="168" value="24" />
        </label>
        <label>Ingest Source
          <input id="source-name" value="celestrak-active" />
        </label>
        <label>Event Type
          <select id="event-type-filter">
            <option value="">All</option>
            <option value="PredictedCloseApproach">PredictedCloseApproach</option>
            <option value="BehaviorShiftDetected">BehaviorShiftDetected</option>
            <option value="UnstableTrack">UnstableTrack</option>
            <option value="CoordinationPatternSuspected">CoordinationPatternSuspected</option>
          </select>
        </label>
        <label class="toggle">
          <input id="toggle-neo-layer" type="checkbox" />
          <span>Show NEO context</span>
        </label>
        <label class="toggle">
          <input id="toggle-close-approach-layer" type="checkbox" checked />
          <span>Show close approaches</span>
        </label>
      </div>
    </section>

      <section class="grid">
        <article class="panel">
          <div class="panel-head"><h3>APOD Briefing</h3></div>
          <div class="panel-body" id="apod-briefing"></div>
        </article>
        <article class="panel">
          <div class="panel-head"><h3>NEO Risk Briefing</h3></div>
          <div class="panel-body" id="neows-briefing"></div>
        </article>
        <article class="panel">
          <div class="panel-head"><h3>Future Event Timeline</h3></div>
          <div class="panel-body list" id="events-timeline"></div>
        </article>
        <article class="panel">
          <div class="panel-head"><h3>Event Queue</h3></div>
          <div class="panel-body list" id="event-queue"></div>
        </article>
        <article class="panel">
          <div class="panel-head"><h3>Passive Operations</h3></div>
          <div class="panel-body list" id="passive-dashboard"></div>
        </article>
        <article class="panel">
          <div class="panel-head">
            <h3>Operational Visibility</h3>
            <button class="secondary" id="refresh-operational-visibility">Refresh</button>
          </div>
          <div class="panel-body list" id="operational-visibility"></div>
        </article>
        <article class="panel">
          <div class="panel-head"><h3>Command Center</h3></div>
          <div class="panel-body list" id="passive-command-center"></div>
        </article>
        <article class="panel">
          <div class="panel-head">
            <h3>Canonical Events</h3>
            <button class="secondary" id="refresh-canonical-events">Refresh</button>
          </div>
          <div class="panel-body list" id="canonical-events"></div>
        </article>
        <article class="panel">
        <div class="panel-head"><h3>Deliveries</h3></div>
        <div class="panel-body list" id="deliveries"></div>
      </article>
      <article class="panel">
        <div class="panel-head"><h3>Ingest Status</h3></div>
        <div class="panel-body list" id="ingest-status"></div>
      </article>
    </section>

    <section class="detail-grid">
      <article class="panel">
        <div class="panel-head"><h3>Object Timeline</h3></div>
        <div class="panel-body list" id="object-timeline"></div>
      </article>
      <article class="panel">
        <div class="panel-head"><h3>Close Approaches</h3></div>
        <div class="panel-body list" id="close-approaches"></div>
      </article>
      <article class="panel">
        <div class="panel-head"><h3>Prediction Snapshots</h3></div>
        <div class="panel-body list" id="prediction-snapshots"></div>
      </article>
      <article class="panel">
        <div class="panel-head"><h3>Replay Diff</h3></div>
        <div class="panel-body" id="replay-diff"></div>
      </article>
      <article class="panel">
        <div class="panel-head">
          <h3>Semantic Timeline</h3>
          <div class="row">
            <span class="status" id="semantic-timeline-label"></span>
            <button class="secondary" id="refresh-semantic-timeline">Refresh</button>
          </div>
        </div>
        <div class="panel-body list" id="semantic-timeline"></div>
      </article>
      <article class="panel">
        <div class="panel-head"><h3>Passive Focus</h3></div>
        <div class="panel-body list" id="passive-focus"></div>
      </article>
    </section>
  </main>

  <script src="https://cesium.com/downloads/cesiumjs/releases/1.127/Build/Cesium/Cesium.js"></script>
  <script>
      const state = {
        lastBundleHash: null,
        lastManifestHash: null,
        viewer: null,
        globeReady: false,
        globeEntities: [],
        passiveGlobeEntities: [],
        passiveFocusOverlayEntities: [],
        neoEntityIndex: {},
        passiveEntityIndex: {},
        passiveSiteEntityIndex: {},
        passiveSiteEventIndex: {},
        neoBriefing: null,
        closeApproaches: null,
        selectedNeoReferenceId: null,
        passiveSemanticFilter: "all",
        passivePressureFilter: "all",
        passiveAttentionKindFilter: "all",
        passiveAttentionPriorityFilter: "all",
        passiveAttentionMapMode: false,
        selectedPassiveOperationalTimeline: null,
        selectedPassiveRegionId: null,
        selectedPassiveSiteId: null,
        selectedPassiveCanonicalEventId: null,
        passiveSelectedAttention: null,
        passiveMapSummary: null,
        passiveDashboardSummary: null,
        passiveMaintenanceSummary: null,
        passiveCommandCenterSummary: null,
        passiveOperationalVisibility: null,
        semanticTimeline: null,
      };

    const $ = (id) => document.getElementById(id);

    function formatSeconds(seconds) {
      if (seconds == null || Number.isNaN(seconds)) return "-";
      if (seconds < 60) return `${seconds}s`;
      if (seconds < 3600) return `${Math.round(seconds / 60)}m`;
      return `${(seconds / 3600).toFixed(1)}h`;
    }

    function formatEpoch(epoch) {
      if (!epoch) return "n/a";
      return new Date(epoch * 1000).toLocaleString();
    }

    function pillClass(value) {
      if (value === "delivered" || value === "ok") return "pill ok";
      if (value === "Warning" || value === "Watch") return "pill warn";
      if (value === "Critical" || value === "failed") return "pill danger";
      return "pill";
    }

    function riskDeltaClass(classification) {
      if (!classification) return "";
      const c = String(classification).toLowerCase();
      if (c === "spike") return "danger";
      if (c === "increase") return "warn";
      if (c === "decrease") return "ok";
      return "";
    }

    function renderRiskDeltaPill(classification) {
      if (!classification) return "";
      const cls = riskDeltaClass(classification);
      return `<span class="pill${cls ? " " + cls : ""}">${escapeHtml(String(classification))}</span>`;
    }

    async function api(path, options) {
      const response = await fetch(path, options);
      const body = await response.json();
      if (!response.ok) {
        throw new Error(body.error?.message || `Request failed: ${response.status}`);
      }
      return body.data;
    }

    function renderEmpty(target, text) {
      target.innerHTML = `<div class="empty">${text}</div>`;
    }

    function renderCards(target, items, render) {
      if (!items.length) return renderEmpty(target, "No data in this window.");
      target.innerHTML = items.map(render).join("");
    }

    function escapeHtml(value) {
      return String(value ?? "")
        .replaceAll("&", "&amp;")
        .replaceAll("<", "&lt;")
        .replaceAll(">", "&gt;")
        .replaceAll('"', "&quot;")
        .replaceAll("'", "&#39;");
    }

    function cesiumPropertyValue(property) {
      if (!property) return null;
      if (typeof property.getValue === "function") {
        return property.getValue(Cesium.JulianDate.now());
      }
      return property;
    }

    function passiveActionLinks(bundleHashes, manifestHashes) {
      const evidenceLinks = (bundleHashes || []).slice(0, 2).map((bundleHash) =>
        `<a class="secondary" href="/v1/evidence/${encodeURIComponent(bundleHash)}" target="_blank" rel="noreferrer">Evidence ${escapeHtml(bundleHash.slice(0, 12))}</a>`
      ).join("");
      const replayButtons = (manifestHashes || []).slice(0, 2).map((manifestHash) =>
        `<button class="secondary passive-replay" data-manifest-hash="${escapeHtml(manifestHash)}">Replay ${escapeHtml(manifestHash.slice(0, 12))}</button>`
      ).join("");
      return `${evidenceLinks}${replayButtons}`;
    }

    function renderFocusedChainActions(payload) {
      const primaryBundleHash = payload?.bundle_hashes?.[0];
      const primaryManifestHash = payload?.manifest_hashes?.[0];
      return `
        <div class="card">
          <h4>Focused Chain Actions</h4>
          <div class="meta">
            <span>${escapeHtml(payload?.site_name ?? payload?.name ?? "unknown site")}</span>
            <span>${escapeHtml(payload?.event_type ?? payload?.top_canonical_status ?? "site focus")}</span>
            <span>${payload?.support_count != null ? `${payload.support_count} signals` : "site scope"}</span>
          </div>
          <div class="row" style="margin-top:12px">
            ${payload?.canonical_event_id ? `<button class="secondary passive-go-map" data-canonical-id="${escapeHtml(payload.canonical_event_id)}" data-site-id="${escapeHtml(payload.site_id)}">Go to Map</button>` : ""}
            ${payload?.site_id ? `<button class="secondary passive-go-site" data-site-id="${escapeHtml(payload.site_id)}">Focus Site</button>` : ""}
            ${primaryBundleHash ? `<button class="secondary passive-open-evidence" data-bundle-hash="${escapeHtml(primaryBundleHash)}">Open Evidence</button>` : ""}
            ${primaryManifestHash ? `<button class="secondary passive-run-replay" data-manifest-hash="${escapeHtml(primaryManifestHash)}">Run Replay</button>` : ""}
          </div>
        </div>`;
    }

    function renderNarrativeProvenance(provenance) {
      const drivers = provenance?.top_drivers || [];
      const paths = [
        provenance?.site_overview_path,
        provenance?.site_narrative_path,
        ...(provenance?.canonical_event_paths || []).slice(0, 2),
        ...(provenance?.evidence_paths || []).slice(0, 2),
        ...(provenance?.replay_paths || []).slice(0, 2),
      ].filter(Boolean);
      return `
        <div class="card">
          <h4>Narrative Provenance</h4>
          <div class="meta">
            <span>${provenance?.canonical_event_ids?.length ?? 0} canonical events</span>
            <span>${provenance?.bundle_hashes?.length ?? 0} evidence bundles</span>
            <span>${provenance?.manifest_hashes?.length ?? 0} replays</span>
          </div>
          <div class="list" style="margin-top:10px">
            ${drivers.slice(0, 3).map((driver) => `
              <div class="card">
                <h4>${escapeHtml(driver.event_type)} Â· ${escapeHtml(driver.status)}</h4>
                <div class="meta">
                  <span>risk ${Math.round((driver.risk_score ?? 0) * 100)}%</span>
                  <span>confidence ${Math.round((driver.confidence ?? 0) * 100)}%</span>
                  <span>${driver.support_count ?? 0} signals</span>
                </div>
                <div class="meta" style="margin-top:8px">${escapeHtml(driver.summary ?? "no summary")}</div>
                <div class="row" style="margin-top:8px">
                  ${driver.bundle_hashes?.[0] ? `<button class="secondary passive-open-evidence" data-bundle-hash="${escapeHtml(driver.bundle_hashes[0])}">Open Evidence</button>` : ""}
                  ${driver.manifest_hashes?.[0] ? `<button class="secondary passive-run-replay" data-manifest-hash="${escapeHtml(driver.manifest_hashes[0])}">Run Replay</button>` : ""}
                </div>
              </div>`).join("") || `<div class="empty">No provenance drivers yet.</div>`}
          </div>
          <div class="row" style="margin-top:10px">
            ${readPathButtons(paths, null)}
          </div>
          <div class="meta" style="margin-top:10px">
            ${paths.slice(0, 6).map((path) => `<span>${escapeHtml(path)}</span>`).join("") || `<span>no paths</span>`}
          </div>
        </div>`;
    }

    function renderRegionProvenance(provenance) {
      const paths = [
        provenance?.region_overview_path,
        provenance?.semantic_timeline_path,
        provenance?.operational_timeline_path,
        provenance?.remediation_path,
        provenance?.runs_path,
        provenance?.source_health_path,
        ...(provenance?.top_site_overview_paths || []).slice(0, 2),
        ...(provenance?.top_site_narrative_paths || []).slice(0, 2),
      ].filter(Boolean);
      return `
        <div class="card">
          <h4>Region Provenance</h4>
          <div class="meta">
            <span>${provenance?.top_site_overview_paths?.length ?? 0} site overviews</span>
            <span>${provenance?.top_site_narrative_paths?.length ?? 0} site narratives</span>
          </div>
          <div class="row" style="margin-top:10px">
            ${readPathButtons(paths, null)}
          </div>
          <div class="meta" style="margin-top:10px">
            ${paths.map((path) => `<span>${escapeHtml(path)}</span>`).join("") || `<span>no paths</span>`}
          </div>
        </div>`;
    }

    async function openPassiveLease(regionId, message) {
      const leases = await api("/v1/passive/regions/leases?limit=100");
      const lease = (leases || []).find((item) => item.region_id === regionId);
      if (!lease) {
        renderEmpty($("passive-focus"), `No active lease found for ${regionId}.`);
        return;
      }
      $("passive-focus").innerHTML = `
        <div class="card">
          <h4>Region Lease</h4>
          <div class="meta">
            <span>${escapeHtml(lease.region_id)}</span>
            <span>${escapeHtml(lease.worker_id)}</span>
            <span>run ${escapeHtml(lease.run_id)}</span>
          </div>
          <div class="meta" style="margin-top:8px">
            <span>acquired ${formatEpoch(lease.acquired_at_unix_seconds)}</span>
            <span>heartbeat ${formatEpoch(lease.heartbeat_at_unix_seconds)}</span>
            <span>expires ${formatEpoch(lease.expires_at_unix_seconds)}</span>
          </div>
        </div>`;
      $("refresh-status").textContent = message;
    }

    async function openPassiveHeartbeat(workerId, message) {
      const heartbeats = await api("/v1/passive/worker/heartbeats?limit=100");
      const heartbeat = (heartbeats || []).find((item) => item.worker_id === workerId);
      if (!heartbeat) {
        renderEmpty($("passive-focus"), `No heartbeat found for ${workerId}.`);
        return;
      }
      $("passive-focus").innerHTML = `
        <div class="card">
          <h4>Worker Heartbeat</h4>
          <div class="meta">
            <span>${escapeHtml(heartbeat.worker_id)}</span>
            <span>${escapeHtml(heartbeat.status)}</span>
            <span>${escapeHtml(heartbeat.current_phase)}</span>
          </div>
          <div class="meta" style="margin-top:8px">
            <span>started ${formatEpoch(heartbeat.started_at_unix_seconds)}</span>
            <span>last ${formatEpoch(heartbeat.last_heartbeat_unix_seconds)}</span>
            <span>${escapeHtml(heartbeat.current_region_id ?? "no region")}</span>
          </div>
          <div class="meta" style="margin-top:8px">
            <span>${escapeHtml(heartbeat.last_error ?? "no error")}</span>
            <span>${escapeHtml(heartbeat.version)}</span>
          </div>
        </div>`;
      $("refresh-status").textContent = message;
    }

    async function openPassiveHeartbeatList(staleOnly, message) {
      const suffix = staleOnly ? "&stale_only=true" : "";
      const heartbeats = await api(`/v1/passive/worker/heartbeats?limit=12${suffix}`);
      $("passive-focus").innerHTML = heartbeats.length ? heartbeats.map((heartbeat) => `
        <div class="card">
          <h4>${escapeHtml(heartbeat.worker_id)}</h4>
          <div class="meta">
            <span>${escapeHtml(heartbeat.status)}</span>
            <span>${escapeHtml(heartbeat.current_phase)}</span>
            <span>${escapeHtml(heartbeat.current_region_id ?? "no region")}</span>
          </div>
          <div class="meta" style="margin-top:8px">
            <span>started ${formatEpoch(heartbeat.started_at_unix_seconds)}</span>
            <span>last ${formatEpoch(heartbeat.last_heartbeat_unix_seconds)}</span>
            <span>${escapeHtml(heartbeat.version)}</span>
          </div>
          <div class="meta" style="margin-top:8px">${escapeHtml(heartbeat.last_error ?? "no error")}</div>
        </div>`).join("") : `<div class="empty">${staleOnly ? "No stale worker heartbeats." : "No worker heartbeats."}</div>`;
      $("refresh-status").textContent = message;
    }

    async function openPassiveWorkerDiagnostics(message, regionId = null) {
      const regionQuery = regionId ? `&region_id=${encodeURIComponent(regionId)}` : "";
      const diagnostics = await api(`/v1/passive/worker/diagnostics?limit=12${regionQuery}`);
      $("passive-focus").innerHTML = `
        <div class="card">
          <h4>Worker Diagnostics</h4>
          <div class="meta">
            <span>${diagnostics.total_worker_heartbeat_count} total</span>
            <span>${diagnostics.active_worker_heartbeat_count} active</span>
            <span>${diagnostics.stale_worker_heartbeat_count} stale</span>
          </div>
          <div class="meta" style="margin-top:8px">
            <span>${diagnostics.active_region_lease_count} active leases</span>
            <span>${diagnostics.stale_region_lease_count} stale leases</span>
            <span>cutoff ${formatEpoch(diagnostics.stale_cutoff_unix_seconds)}</span>
          </div>
          ${diagnostics.region_id ? `<div class="meta" style="margin-top:8px"><span>scope ${escapeHtml(diagnostics.region_id)}</span></div>` : ""}
          <div class="meta" style="margin-top:8px">${escapeHtml(diagnostics.recommendation)}</div>
          <div class="row" style="margin-top:12px">
            ${diagnostics.stale_worker_heartbeat_count > 0 ? `<button class="secondary passive-prune-stale-heartbeats">Prune Stale Heartbeats</button>` : ""}
          </div>
        </div>
        ${diagnostics.active_region_leases.length ? `
          <div class="card">
            <h4>Active Leases</h4>
            <div class="list" style="margin-top:10px">
              ${diagnostics.active_region_leases.map((lease) => `
                <div class="card">
                  <h4>${escapeHtml(lease.region_id)}</h4>
                  <div class="meta">
                    <span>${escapeHtml(lease.worker_id)}</span>
                    <span>run ${escapeHtml(lease.run_id)}</span>
                    <span>${lease.expires_in_seconds}s to expiry</span>
                  </div>
                  <div class="meta" style="margin-top:8px">
                    <span>heartbeat lag ${lease.heartbeat_lag_seconds}s</span>
                    <span>${formatEpoch(lease.heartbeat_at_unix_seconds)}</span>
                    <span>${formatEpoch(lease.expires_at_unix_seconds)}</span>
                  </div>
                </div>`).join("")}
            </div>
          </div>` : ""}
        ${diagnostics.stale_region_leases.length ? `
          <div class="card">
            <h4>Stale Leases</h4>
            <div class="list" style="margin-top:10px">
              ${diagnostics.stale_region_leases.map((lease) => `
                <div class="card">
                  <h4>${escapeHtml(lease.region_id)}</h4>
                  <div class="meta">
                    <span>${escapeHtml(lease.worker_id)}</span>
                    <span>run ${escapeHtml(lease.run_id)}</span>
                    <span class="${pillClass("failed")}">stale</span>
                  </div>
                  <div class="meta" style="margin-top:8px">
                    <span>heartbeat lag ${lease.heartbeat_lag_seconds}s</span>
                    <span>${formatEpoch(lease.heartbeat_at_unix_seconds)}</span>
                    <span>${formatEpoch(lease.expires_at_unix_seconds)}</span>
                  </div>
                </div>`).join("")}
            </div>
          </div>` : ""}
        ${diagnostics.region_metrics.length ? `
          <div class="card">
            <h4>Region Metrics</h4>
            <div class="list" style="margin-top:10px">
              ${diagnostics.region_metrics.map((metric) => `
                <div class="card">
                  <h4>${escapeHtml(metric.region_id)}</h4>
                  <div class="meta">
                    <span>${metric.run_count} runs</span>
                    <span>${metric.event_count} events</span>
                    <span>${metric.source_error_count} source errors</span>
                  </div>
                  <div class="meta" style="margin-top:8px">
                    <span>${metric.active_lease_count} active leases</span>
                    <span>${metric.stale_lease_count} stale leases</span>
                    <span>${metric.active_worker_count} active workers</span>
                    <span>${metric.stale_worker_count} stale workers</span>
                  </div>
                  <div class="meta" style="margin-top:8px">
                    <span>${metric.completed_run_count} completed</span>
                    <span>${metric.partial_run_count} partial</span>
                    <span>${metric.failed_run_count} failed</span>
                    <span>${escapeHtml(metric.latest_run_status ?? "no recent run")}</span>
                  </div>
                </div>`).join("")}
            </div>
          </div>` : ""}
        ${diagnostics.source_metrics.length ? `
          <div class="card">
            <h4>Source Metrics</h4>
            <div class="list" style="margin-top:10px">
              ${diagnostics.source_metrics.map((metric) => `
                <div class="card">
                  <h4>${escapeHtml(metric.source)}</h4>
                  <div class="meta">
                    <span>${escapeHtml(metric.region_id ?? "global")}</span>
                    <span class="${pillClass(metric.health_status === "healthy" ? "ok" : metric.health_status === "watch" ? "Warning" : "failed")}">${escapeHtml(metric.health_status ?? "unknown")}</span>
                    <span>${metric.sample_count} samples</span>
                    <span>${Math.round((metric.success_rate ?? 0) * 100)}% success</span>
                    <span>${Math.round((metric.reliability_score ?? 0) * 100)}% reliable</span>
                  </div>
                  <div class="meta" style="margin-top:8px">
                    <span>${metric.success_count} ok</span>
                    <span>${metric.failure_count} fail</span>
                    <span>${metric.consecutive_failure_count ?? 0} consecutive fail</span>
                    <span>${metric.staleness_seconds != null ? `${formatSeconds(metric.staleness_seconds)} old` : "no staleness"}</span>
                    <span>${metric.latest_generated_at_unix_seconds ? formatEpoch(metric.latest_generated_at_unix_seconds) : "no samples"}</span>
                  </div>
                  <div class="row" style="margin-top:8px">
                    <button class="secondary passive-open-source-samples" data-source-kind="${escapeHtml(metric.source)}" data-region-id="${escapeHtml(metric.region_id ?? "")}">View Samples</button>
                    <button class="secondary passive-preview-source-prune" data-source-kind="${escapeHtml(metric.source)}" data-region-id="${escapeHtml(metric.region_id ?? "")}">Preview Prune</button>
                    <button class="secondary passive-prune-source-health" data-source-kind="${escapeHtml(metric.source)}" data-region-id="${escapeHtml(metric.region_id ?? "")}">Prune Old Samples</button>
                  </div>
                  <div class="empty">${escapeHtml(metric.recovery_hint ?? "No recovery hint.")}</div>
                  ${metric.last_error ? `<div class="empty">${escapeHtml(metric.last_error)}</div>` : ""}
                </div>`).join("")}
            </div>
          </div>` : ""}
        ${diagnostics.stale_worker_heartbeats.length ? diagnostics.stale_worker_heartbeats.map((heartbeat) => `
          <div class="card">
            <h4>${escapeHtml(heartbeat.worker_id)}</h4>
            <div class="meta">
              <span>${escapeHtml(heartbeat.status)}</span>
              <span>${escapeHtml(heartbeat.current_phase)}</span>
              <span>${escapeHtml(heartbeat.current_region_id ?? "no region")}</span>
            </div>
            <div class="meta" style="margin-top:8px">
              <span>started ${formatEpoch(heartbeat.started_at_unix_seconds)}</span>
              <span>last ${formatEpoch(heartbeat.last_heartbeat_unix_seconds)}</span>
              <span>${escapeHtml(heartbeat.version)}</span>
            </div>
            <div class="meta" style="margin-top:8px">${escapeHtml(heartbeat.last_error ?? "no error")}</div>
          </div>`).join("") : `<div class="empty">No stale worker heartbeats.</div>`}`;
      $("refresh-status").textContent = message;
    }

    async function pruneStaleHeartbeats(message) {
      const result = await api("/v1/passive/worker/heartbeats/prune", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ older_than_seconds: 86400 }),
      });
      await refreshPassiveDashboardSummary();
      await openPassiveWorkerDiagnostics(`${message}: pruned ${result.pruned_count}`);
    }

    async function pruneSourceHealthSamples(sourceKind = null, regionId = null, message = "Pruned source health samples", dryRun = false) {
      const body = { older_than_seconds: 604800, dry_run: dryRun };
      if (sourceKind) body.source_kind = sourceKind;
      if (regionId) body.region_id = regionId;
      const result = await api("/v1/passive/source-health/samples/prune", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(body),
      });
      const sourceLabel = sourceKind || "all source";
      if (dryRun) {
        $("refresh-status").textContent = `${message}: ${result.pruned_count} ${sourceLabel} samples would be pruned`;
        return result;
      }
      await refreshPassiveDashboardSummary();
      await openPassiveWorkerDiagnostics(
        `${message}: pruned ${result.pruned_count} ${sourceLabel} samples`,
        regionId,
      );
    }

    async function openPassiveSourceSamples(sourceKind, message, regionId = null) {
      const regionQuery = regionId ? `&region_id=${encodeURIComponent(regionId)}` : "";
      const data = await api(`/v1/passive/source-health/samples?limit=8&source_kind=${encodeURIComponent(sourceKind)}${regionQuery}`);
      $("passive-focus").innerHTML = data.length ? data.map((sample) => `
        <div class="card">
          <h4>${escapeHtml(sample.source_kind)} sample</h4>
          <div class="meta">
            <span class="${pillClass(sample.fetched ? "ok" : "failed")}">${sample.fetched ? "fetched" : "failed"}</span>
            <span>${sample.observations_collected} observations</span>
            <span>${escapeHtml(sample.region_id ?? "global")}</span>
          </div>
          <div class="meta" style="margin-top:8px">
            <span>${formatEpoch(sample.generated_at_unix_seconds)}</span>
            <span>${formatEpoch(sample.window_start_unix_seconds)} -> ${formatEpoch(sample.window_end_unix_seconds)}</span>
          </div>
          <div class="row" style="margin-top:8px">
            <button class="secondary passive-preview-source-prune" data-source-kind="${escapeHtml(sample.source_kind)}" data-region-id="${escapeHtml(sample.region_id ?? "")}">Preview Prune</button>
            <button class="secondary passive-prune-source-health" data-source-kind="${escapeHtml(sample.source_kind)}" data-region-id="${escapeHtml(sample.region_id ?? "")}">Prune Old ${escapeHtml(sample.source_kind)} Samples</button>
          </div>
          <div class="meta" style="margin-top:8px">${escapeHtml(sample.detail)}</div>
        </div>`).join("") : `<div class="empty">No source samples available for ${escapeHtml(sourceKind)}.</div>`;
      $("refresh-status").textContent = message;
    }

    async function openPassiveRegionRuns(regionId, message) {
      const runs = await api(`/v1/passive/regions/runs?limit=12&region_id=${encodeURIComponent(regionId)}`);
      $("passive-focus").innerHTML = runs.length ? runs.map((run) => `
        <div class="card">
          <h4>${escapeHtml(run.run_id)}</h4>
          <div class="meta">
            <span>${escapeHtml(run.status)}</span>
            <span>${escapeHtml(run.origin)}</span>
            <span>${run.event_count} events</span>
          </div>
          <div class="meta" style="margin-top:8px">
            <span>${run.region_ids?.join(", ") || "no region"}</span>
            <span>${formatEpoch(run.started_at_unix_seconds)}</span>
            <span>${run.finished_at_unix_seconds ? formatEpoch(run.finished_at_unix_seconds) : "running"}</span>
          </div>
          <div class="row" style="margin-top:8px">
            <button class="secondary passive-open-region-run" data-run-id="${escapeHtml(run.run_id)}">Open Run</button>
          </div>
        </div>`).join("") : `<div class="empty">No region runs available for ${escapeHtml(regionId)}.</div>`;
      $("refresh-status").textContent = message;
    }

    async function openPassiveRegionRun(runId, message) {
      const run = await api(`/v1/passive/regions/runs/${encodeURIComponent(runId)}`);
      $("passive-focus").innerHTML = `
        <div class="card">
          <h4>Region Run</h4>
          <div class="meta">
            <span>${escapeHtml(run.run_id)}</span>
            <span>${escapeHtml(run.status)}</span>
            <span>${escapeHtml(run.origin)}</span>
          </div>
          <div class="meta" style="margin-top:8px">
            <span>${run.region_ids?.join(", ") || "no region"}</span>
            <span>${formatEpoch(run.started_at_unix_seconds)}</span>
            <span>${run.finished_at_unix_seconds ? formatEpoch(run.finished_at_unix_seconds) : "running"}</span>
          </div>
          <div class="meta" style="margin-top:8px">
            <span>${run.discovered_seed_count} discovered</span>
            <span>${run.selected_seed_count} selected</span>
            <span>${run.event_count} events</span>
            <span>${run.source_errors?.length || 0} source errors</span>
          </div>
          ${run.source_errors?.length ? `<div class="meta" style="margin-top:8px">${escapeHtml(run.source_errors.join(" | "))}</div>` : ""}
        </div>`;
      $("refresh-status").textContent = message;
    }

    function remediationReadButtons(action, regionId) {
      const paths = action?.suggested_read_paths || [];
      return paths.map((path) => {
        if (path.includes("/passive/worker/diagnostics")) {
          return `<button class="secondary passive-open-worker-diagnostics" data-region-id="${escapeHtml(regionId)}">Diagnostics</button>`;
        }
        if (path.includes("/passive/regions/runs")) {
          return `<button class="secondary passive-open-region-runs" data-region-id="${escapeHtml(regionId)}">Region Runs</button>`;
        }
        if (path.includes("/passive/regions/") && path.includes("/overview")) {
          return `<button class="secondary passive-open-region-overview" data-region-id="${escapeHtml(regionId)}">Region Overview</button>`;
        }
        return "";
      }).join("");
    }

    function readPathLabel(path) {
      if (!path) return "Open Path";
      if (path.includes("/v1/evidence/")) return "Evidence";
      if (path.includes("/v1/replay/")) return "Replay";
      if (path.includes("/passive/regions/") && path.includes("/overview")) return "Region Overview";
      if (path.includes("/passive/worker/diagnostics")) return "Diagnostics";
      if (path.includes("/passive/worker/heartbeats?stale_only=true")) return "Stale Heartbeats";
      if (path.includes("/passive/worker/heartbeats")) return "Worker Heartbeats";
      if (path.includes("/passive/map/sites")) return "Map Sites";
      if (path.includes("/passive/map/canonical-events")) return "Map Events";
      if (path.includes("/passive/sites/") && path.includes("/overview")) return "Site Overview";
      if (path.includes("/passive/sites/") && path.includes("/narrative")) return "Narrative";
      if (path.includes("/semantic-timeline")) return "Timeline";
      if (path.includes("/passive/operational-visibility")) return "Operational Visibility";
      return "Open Path";
    }

    function readPathButtons(paths, regionId, attentionItem) {
      const attentionAttrs = attentionItem ? attentionItemDataAttrs(attentionItem) : "";
      return (paths || []).slice(0, 4).map((path) => {
        if (path.includes("/v1/evidence/")) {
          const bundleHash = path.split("/v1/evidence/")[1]?.split("?")[0];
          return bundleHash
            ? `<button class="secondary passive-open-evidence" data-bundle-hash="${escapeHtml(bundleHash)}" ${attentionAttrs}>${escapeHtml(readPathLabel(path))}</button>`
            : "";
        }
        if (path.includes("/v1/replay/")) {
          const manifestHash = path.split("/v1/replay/")[1]?.split("/")[0]?.split("?")[0];
          return manifestHash
            ? `<button class="secondary passive-run-replay" data-manifest-hash="${escapeHtml(manifestHash)}" ${attentionAttrs}>${escapeHtml(readPathLabel(path))}</button>`
            : "";
        }
        if (path.includes("/passive/regions/") && path.includes("/overview")) {
          return `<button class="secondary passive-open-region-overview" data-region-id="${escapeHtml(regionId || "")}" ${attentionAttrs}>${escapeHtml(readPathLabel(path))}</button>`;
        }
        if (path.includes("/passive/worker/diagnostics")) {
          return `<button class="secondary passive-open-worker-diagnostics" data-region-id="${escapeHtml(regionId || "")}" ${attentionAttrs}>${escapeHtml(readPathLabel(path))}</button>`;
        }
        if (path.includes("/passive/worker/heartbeats?stale_only=true")) {
          return `<button class="secondary passive-open-stale-heartbeats" ${attentionAttrs}>${escapeHtml(readPathLabel(path))}</button>`;
        }
        if (path.includes("/passive/worker/heartbeats")) {
          return `<button class="secondary passive-open-heartbeat-list" ${attentionAttrs}>${escapeHtml(readPathLabel(path))}</button>`;
        }
        return `<button class="secondary passive-open-path" data-path="${escapeHtml(path)}" ${attentionAttrs}>${escapeHtml(readPathLabel(path))}</button>`;
      }).join("");
    }

    function renderReplayExecution(replay, manifestHash) {
      state.lastManifestHash = manifestHash;
      $("metric-drift").textContent = replay.drift_detected ? "Yes" : "No";
      $("replay-diff").innerHTML = `
        <div class="list">
          <div class="card">
            <h4>Replay ${escapeHtml(manifestHash.slice(0, 12))}</h4>
            <div class="meta">
              <span class="${pillClass(replay.drift_detected ? "failed" : "ok")}">${replay.drift_detected ? "drift detected" : "stable"}</span>
            </div>
          </div>
          <div class="card">
            <h4>Assessment</h4>
            <pre>${escapeHtml(JSON.stringify(replay.diff.assessment, null, 2))}</pre>
          </div>
          <div class="card">
            <h4>Decision</h4>
            <pre>${escapeHtml(JSON.stringify(replay.diff.decision, null, 2))}</pre>
          </div>
          </div>`; 
    }

    function focusPassiveSite(siteId, statusLabel) {
      if (!siteId || !state.viewer) return false;
      const entity = state.passiveSiteEntityIndex[siteId];
      if (!entity) {
        $("refresh-status").textContent = `${statusLabel || "Site"} ${siteId.slice(0, 12)} is outside the current map region`;
        return false;
      }
      applyPassiveSiteFocus(siteId, null);
      state.viewer.selectedEntity = entity;
      state.viewer.flyTo(entity, { duration: 1.2 });
      $("refresh-status").textContent = `${statusLabel || "Focused site"} ${siteId.slice(0, 12)}`;
      return true;
    }

    async function focusPassiveSiteFromAttention(siteId, regionId, statusLabel) {
      if (focusPassiveSite(siteId, statusLabel)) return true;
      if (!regionId) return false;
      state.selectedPassiveRegionId = regionId;
      state.passivePressureFilter = "all";
      await refreshPassiveMapLayers();
      return focusPassiveSite(siteId, statusLabel);
    }

    function openPassivePath(path, statusLabel) {
      if (!path) return false;
      window.open(path, "_blank", "noopener,noreferrer");
      $("refresh-status").textContent = statusLabel || `Opened ${path}`;
      return true;
    }

    function formatNeoDistance(distanceKm) {
      if (distanceKm == null || Number.isNaN(distanceKm)) return "distance n/a";
      return `${Math.round(distanceKm).toLocaleString()} km`;
    }

    function formatNeoVelocity(velocityKmS) {
      if (velocityKmS == null || Number.isNaN(velocityKmS)) return "velocity n/a";
      return `${velocityKmS.toFixed(1)} km/s`;
    }

    function formatNeoDiameter(minMeters, maxMeters) {
      const min = Number.isFinite(minMeters) ? Math.round(minMeters).toLocaleString() : null;
      const max = Number.isFinite(maxMeters) ? Math.round(maxMeters).toLocaleString() : null;
      if (min && max) return `${min}-${max} m`;
      if (max) return `up to ${max} m`;
      if (min) return `${min} m+`;
      return "diameter n/a";
    }

    function neoBriefingFeedPath(briefing) {
      const params = new URLSearchParams();
      if (briefing?.start_date) params.set("start_date", briefing.start_date);
      if (briefing?.end_date) params.set("end_date", briefing.end_date);
      const query = params.toString();
      return query ? `/v1/briefing/neows?${query}` : "/v1/briefing/neows";
    }

    function neoBriefingItemDataAttrs(item) {
      return `data-neo-reference-id="${escapeHtml(item.neo_reference_id ?? "")}"
        data-neo-name="${escapeHtml(item.name ?? "")}"
        data-neo-jpl-url="${escapeHtml(item.nasa_jpl_url ?? "")}"`;
    }

    function selectedNeoBriefingItem() {
      const items = state.neoBriefing?.highest_priority || [];
      if (!items.length) return null;
      const selected = items.find((item) => item.neo_reference_id === state.selectedNeoReferenceId) || items[0];
      state.selectedNeoReferenceId = selected.neo_reference_id;
      return selected;
    }

    function renderNeoBriefingActionButtons(item, focusLabel) {
      const feedPath = neoBriefingFeedPath(state.neoBriefing);
      return `
        <button class="secondary neo-briefing-focus" ${neoBriefingItemDataAttrs(item)}>${escapeHtml(focusLabel || "Focus on Globe")}</button>
        ${item.nasa_jpl_url ? `<button class="secondary neo-open-path" data-path="${escapeHtml(item.nasa_jpl_url)}" ${neoBriefingItemDataAttrs(item)}>JPL Profile</button>` : ""}
        <button class="secondary neo-open-path" data-path="${escapeHtml(feedPath)}" ${neoBriefingItemDataAttrs(item)}>Feed Window</button>`;
    }

    function renderNeoWsBriefing(data) {
      if (!data) {
        state.selectedNeoReferenceId = null;
        renderEmpty($("neows-briefing"), "No NEO briefing available.");
        return;
      }

      const items = data.highest_priority || [];
      const selected = selectedNeoBriefingItem();
      const feedPath = neoBriefingFeedPath(data);
      $("neows-briefing").innerHTML = `
        <div class="list">
          <div class="card">
            <h4>${data.total_objects} NEOs in window</h4>
            <div class="meta">
              <span class="pill">${data.hazardous_objects} hazardous</span>
              <span>${data.start_date} -> ${data.end_date}</span>
              <span>generated ${formatEpoch(data.generated_at_unix_seconds)}</span>
            </div>
            <div class="row" style="margin-top:10px">
              <button class="secondary neo-open-path" data-path="${escapeHtml(feedPath)}">Open Feed Window</button>
              ${selected ? `<button class="secondary neo-briefing-focus" ${neoBriefingItemDataAttrs(selected)}>Focus Selected</button>` : ""}
            </div>
          </div>
          ${selected ? `
            <div class="card neo-briefing-item neo-briefing-item-selected" ${neoBriefingItemDataAttrs(selected)}>
              <h4>Attention Focus: ${escapeHtml(selected.name)}</h4>
              <div class="meta">
                <span class="${pillClass(selected.hazardous ? "Critical" : "Watch")}">${selected.hazardous ? "hazardous" : "watch"}</span>
                <span>priority ${(selected.priority_score * 100).toFixed(0)}%</span>
                <span>${escapeHtml(selected.close_approach_date ?? "date n/a")}</span>
                <span>${formatNeoDiameter(selected.estimated_diameter_min_m, selected.estimated_diameter_max_m)}</span>
              </div>
              <div class="meta" style="margin-top:8px">
                <span>${formatNeoDistance(selected.miss_distance_km)}</span>
                <span>${formatNeoVelocity(selected.relative_velocity_km_s)}</span>
                <span class="mono">${escapeHtml(selected.neo_reference_id)}</span>
              </div>
              <div class="meta" style="margin-top:8px">${escapeHtml(selected.briefing_summary)}</div>
              <div class="row" style="margin-top:10px">${renderNeoBriefingActionButtons(selected, "Focus on Globe")}</div>
            </div>` : `<div class="empty">No priority NEO objects in this window.</div>`}
          ${items.slice(0, 3).map((item, index) => `
            <div class="card neo-briefing-item ${selected?.neo_reference_id === item.neo_reference_id ? "neo-briefing-item-selected" : ""}" ${neoBriefingItemDataAttrs(item)}>
              <div class="row" style="justify-content:space-between;align-items:flex-start">
                <h4>${escapeHtml(item.name)}</h4>
                <span class="pill ${item.hazardous ? "danger" : "warn"}">${index === 0 ? "top priority" : "read target"}</span>
              </div>
              <div class="meta">
                <span>${item.hazardous ? "hazardous" : "watch"}</span>
                <span>priority ${(item.priority_score * 100).toFixed(0)}%</span>
                <span>${escapeHtml(item.close_approach_date ?? "date n/a")}</span>
              </div>
              <div class="meta" style="margin-top:8px">
                <span>${formatNeoDistance(item.miss_distance_km)}</span>
                <span>${formatNeoVelocity(item.relative_velocity_km_s)}</span>
                <span>${formatNeoDiameter(item.estimated_diameter_min_m, item.estimated_diameter_max_m)}</span>
              </div>
              <div class="meta" style="margin-top:8px">${escapeHtml(item.briefing_summary)}</div>
              <div class="row" style="margin-top:8px">${renderNeoBriefingActionButtons(item, selected?.neo_reference_id === item.neo_reference_id ? "Refocus Globe" : "Focus on Globe")}</div>
            </div>
          `).join("")}
        </div>`;
    }

    function selectNeoBriefingItem(neoReferenceId, statusLabel) {
      if (!neoReferenceId) return null;
      const item = (state.neoBriefing?.highest_priority || []).find((candidate) => candidate.neo_reference_id === neoReferenceId);
      if (!item) return null;
      state.selectedNeoReferenceId = item.neo_reference_id;
      renderNeoWsBriefing(state.neoBriefing);
      if (statusLabel) {
        $("refresh-status").textContent = `${statusLabel} ${item.name}`;
      }
      return item;
    }

    async function focusNeoBriefingItem(neoReferenceId, statusLabel) {
      const item = selectNeoBriefingItem(neoReferenceId, null);
      if (!item) return false;

      if (!$("toggle-neo-layer")?.checked) {
        $("toggle-neo-layer").checked = true;
        await refreshAll();
      }

      const entity = state.neoEntityIndex[item.neo_reference_id];
      if (!state.viewer || !entity) {
        $("refresh-status").textContent = `${statusLabel || "Selected NEO"} ${item.name}`;
        return true;
      }

      if (state.viewer.selectedEntity !== entity) {
        state.viewer.selectedEntity = entity;
      }
      state.viewer.flyTo(entity, { duration: 1.2 });
      $("refresh-status").textContent = `${statusLabel || "Focused NEO"} ${item.name}`;
      return true;
    }

    function openPassiveEvidence(bundleHash, statusLabel) {
      if (!bundleHash) return false;
      window.open(`/v1/evidence/${encodeURIComponent(bundleHash)}`, "_blank", "noopener,noreferrer");
      $("refresh-status").textContent = `${statusLabel || "Opened evidence"} ${bundleHash.slice(0, 12)}`;
      return true;
    }

    async function runPassiveReplay(manifestHash, statusLabel) {
      if (!manifestHash) return false;
      $("refresh-status").textContent = "Running passive replay...";
      const replay = await api(`/v1/replay/${encodeURIComponent(manifestHash)}/execute`);
      renderReplayExecution(replay, manifestHash);
      $("refresh-status").textContent = `${statusLabel || "Passive replay"} ${manifestHash.slice(0, 12)} executed`;
      return true;
    }

    function focusPassiveCanonicalEvent(canonicalId, siteId, statusLabel) {
      if (!canonicalId || !state.viewer) return false;
      const entity = state.passiveEntityIndex[canonicalId];
      if (!entity) {
        $("refresh-status").textContent = `${statusLabel || "Canonical event"} ${canonicalId.slice(0, 12)} is outside the current map filter`;
        return false;
      }
      applyPassiveSiteFocus(siteId || cesiumPropertyValue(entity?.properties?.payload)?.site_id, canonicalId);
      state.viewer.selectedEntity = entity;
      state.viewer.flyTo(entity, { duration: 1.2 });
      $("refresh-status").textContent = `${statusLabel || "Focused canonical event"} ${canonicalId.slice(0, 12)}`;
      return true;
    }

    async function focusPassiveCanonicalFromAttention(canonicalId, siteId, regionId, statusLabel) {
      if (focusPassiveCanonicalEvent(canonicalId, siteId, statusLabel)) return true;
      if (!regionId && state.passiveSemanticFilter === "all") return false;
      if (regionId) state.selectedPassiveRegionId = regionId;
      state.passivePressureFilter = "all";
      state.passiveSemanticFilter = "all";
      await refreshPassiveMapLayers();
      return focusPassiveCanonicalEvent(canonicalId, siteId, statusLabel);
    }

    async function focusPassiveAttentionItem(kind, regionId, siteId, canonicalId) {
      if (kind === "canonical_event") {
        return focusPassiveCanonicalFromAttention(
          canonicalId,
          siteId,
          regionId,
          "Focused canonical event",
        );
      }
      if (kind === "site") {
        return focusPassiveSiteFromAttention(siteId, regionId, "Focused site");
      }
      if (kind === "region") {
        return openPassiveRegionOverview(regionId, "Opened region");
      }
      return false;
    }

    async function openPassiveRegionOverview(regionId, statusLabel) {
      if (!regionId) return false;
      state.selectedPassiveRegionId = regionId;
      let regionEntity = state.viewer?.entities?.getById?.(`passive-region-${regionId}`);
      if (!regionEntity) {
        state.passivePressureFilter = "all";
        await refreshPassiveMapLayers();
        regionEntity = state.viewer?.entities?.getById?.(`passive-region-${regionId}`);
      }
      if (!regionEntity) {
        $("refresh-status").textContent = `${statusLabel || "Region"} ${regionId} is outside the current map data`;
        return false;
      }
      await handlePassiveSelection(regionEntity);
      $("refresh-status").textContent = `${statusLabel || "Opened region"} ${regionId}`;
      return true;
    }

      function formatRelative(epochSeconds) {
        if (!epochSeconds) return "n/a";
        const diffS = Math.round(Date.now() / 1000 - epochSeconds);
        if (diffS < 60) return `${diffS}s ago`;
        if (diffS < 3600) return `${Math.round(diffS / 60)}m ago`;
        if (diffS < 86400) return `${Math.round(diffS / 3600)}h ago`;
        return `${Math.round(diffS / 86400)}d ago`;
      }

      function trendArrow(classification) {
        if (!classification) return "";
        const c = String(classification).toLowerCase();
        if (c === "spike" || c === "critical") return "⬆ ";
        if (c === "increase" || c === "high") return "↗ ";
        if (c === "decrease") return "↘ ";
        if (c === "stable") return "→ ";
        return "";
      }

      function suggestAction(entry) {
        const delta = (entry.risk_delta?.classification || "").toLowerCase();
        const risk = entry.risk_score || 0;
        if (risk > 0.85 || delta === "spike" || delta === "critical") return "ESCALATE";
        if (delta === "increase" || delta === "high") return "MONITOR";
        if (delta === "stable" || delta === "decrease") return "IGNORE";
        return "RESPOND";
      }

      function actionPillClass(action) {
        if (action === "ESCALATE") return "stl-action-escalate";
        if (action === "MONITOR")  return "stl-action-monitor";
        if (action === "IGNORE")   return "stl-action-ignore";
        return "stl-action-respond";
      }

      function actionLabel(action) {
        if (action === "ESCALATE") return "🔴 ESCALATE";
        if (action === "MONITOR")  return "🟡 MONITOR";
        if (action === "IGNORE")   return "⚪ IGNORE";
        return "🟣 RESPOND";
      }

      function renderSemanticTimeline(timeline) {
        const entries = timeline?.entries || [];
        if (!entries.length) {
          const ctx = timeline?.scope_id ? `for ${escapeHtml(timeline.scope_id)}` : "";
          return `<div class="card"><h4>Semantic Timeline</h4><div class="stl-empty"><span class="stl-empty-icon">○</span><span>No events recorded ${ctx}</span><span>Events will appear when passive scans detect activity in this region.</span></div></div>`;
        }
        const dominantCls = riskDeltaClass(timeline.dominant_status);
        const nowS = Math.round(Date.now() / 1000);
        return `
          <div class="card">
            <h4>Semantic Timeline</h4>
            <div class="meta">
              <span>${escapeHtml(timeline.scope_id)}</span>
              <span class="pill${dominantCls ? " " + dominantCls : ""}">${escapeHtml(timeline.dominant_status ?? "no state")}</span>
              <span>${entries.length} event${entries.length === 1 ? "" : "s"}</span>
            </div>
            <div class="list" style="margin-top:10px">
              ${entries.slice(0, 10).map((entry) => {
                const delta = (entry.risk_delta?.classification || "").toLowerCase();
                const cls = riskDeltaClass(entry.risk_delta?.classification);
                const riskPct = entry.risk_score != null ? `${(entry.risk_score * 100).toFixed(0)}%` : "n/a";
                const arrow = trendArrow(entry.risk_delta?.classification);
                const ago = formatRelative(entry.last_observed_at_unix_seconds);
                const firstAgo = formatRelative(entry.first_observed_at_unix_seconds);
                const action = suggestAction(entry);
                const isHighlight = action === "ESCALATE" || delta === "spike";
                const isNew = entry.last_observed_at_unix_seconds && (nowS - entry.last_observed_at_unix_seconds) < 7200;
                const badges = [
                  isHighlight ? `<span class="stl-badge stl-badge-spike">SPIKE</span>` : "",
                  (!isHighlight && isNew) ? `<span class="stl-badge stl-badge-new">NEW</span>` : "",
                ].join("");
                const hasSite = !!entry.site_id;
                const hasCanonical = !!entry.canonical_event_id;
                const hasReplay = !!(entry.manifest_hash);
                const regionId = escapeHtml(entry.region_id || timeline.scope_id || "");
                return `
                  <div class="passive-timeline-entry${isHighlight ? " stl-highlight" : ""}"
                       data-canonical-id="${escapeHtml(entry.canonical_event_id)}"
                       data-site-id="${escapeHtml(entry.site_id)}"
                       data-region-id="${regionId}"
                       data-manifest-hash="${escapeHtml(entry.manifest_hash || "")}"
                       style="margin-bottom:8px; padding:8px; border-radius:5px; background:rgba(255,255,255,0.03); border:1px solid var(--line,#444);">
                    <div class="stl-entry-headline">${escapeHtml(entry.event_type)} — ${escapeHtml(entry.site_name)}${badges}</div>
                    <div class="stl-entry-trend">
                      <span class="pill${cls ? " " + cls : ""}"><span aria-hidden="true">${arrow}</span>${escapeHtml(entry.risk_delta?.classification ?? "stable")}</span>
                      <span>risk ${riskPct}</span>
                      <span>${escapeHtml(entry.status)}</span>
                      <span>${entry.support_count} signal${entry.support_count === 1 ? "" : "s"}</span>
                    </div>
                    <div class="stl-entry-when" title="first seen ${firstAgo}">last ${ago} · first ${firstAgo}</div>
                    <div class="stl-quick-actions">
                      <span class="pill ${actionPillClass(action)}" style="font-size:11px;">${actionLabel(action)}</span>
                      ${hasCanonical ? `<button class="stl-qa-view" data-canonical-id="${escapeHtml(entry.canonical_event_id)}" data-site-id="${escapeHtml(entry.site_id)}" data-region-id="${regionId}">View Event</button>` : ""}
                      ${hasSite ? `<button class="stl-qa-site" data-site-id="${escapeHtml(entry.site_id)}" data-region-id="${regionId}">Open Site</button>` : ""}
                      ${hasReplay ? `<button class="stl-qa-replay" data-manifest-hash="${escapeHtml(entry.manifest_hash)}">Replay</button>` : ""}
                    </div>
                  </div>
                `;
              }).join("")}
            </div>
          </div>`;
      }

    function operationalPriorityClass(priority) {
      if (priority === "critical" || priority === "high") return "failed";
      if (priority === "medium") return "Warning";
      return "ok";
    }

    function visibilityStateClass(stateValue) {
      if (stateValue === "failing") return "danger";
      if (stateValue === "degraded" || stateValue === "pressured") return "warn";
      return "ok";
    }

    function regionOperationalState(regionId) {
      return (state.passiveOperationalVisibility?.regions || [])
        .find((r) => r.region_id === regionId)?.state ?? null;
    }

    function renderRegionVisibilityCard(regionId, withRunsButton) {
      const vis = (state.passiveOperationalVisibility?.regions || [])
        .find((r) => r.region_id === regionId);
      if (!vis) return "";
      return `<div class="card">
        <h4>Operational State</h4>
        <div class="meta">
          <span class="pill ${visibilityStateClass(vis.state)}">${escapeHtml(vis.state)}</span>
          <span>${vis.drivers.active_worker_count} active workers</span>
          <span>${vis.drivers.stale_worker_count} stale workers</span>
          <span>${vis.drivers.active_lease_count} active leases</span>
          <span>${vis.drivers.stale_lease_count} stale leases</span>
        </div>
        <div class="meta" style="margin-top:8px">
          <span>${vis.drivers.run_count} runs</span>
          <span>${vis.drivers.recent_failed_run_count} failed</span>
          <span>${vis.drivers.recent_partial_run_count} partial</span>
          ${vis.drivers.cadence_gap_seconds != null ? `<span>gap ${formatSeconds(vis.drivers.cadence_gap_seconds)}</span>` : ""}
        </div>
        ${vis.narrative ? `<div class="meta" style="margin-top:8px">${escapeHtml(vis.narrative)}</div>` : ""}
        ${withRunsButton ? `<div class="row" style="margin-top:8px">
          <button class="secondary passive-open-region-runs" data-region-id="${escapeHtml(regionId)}">View Region Runs</button>
        </div>` : ""}
      </div>`;
    }

    function renderPassiveCommandCenterPanel(commandCenter) {
      const target = $("passive-command-center");
      if (!target) return;
      if (!commandCenter) { renderEmpty(target, "No command center data."); return; }
      const highlights = commandCenter.highlights || {};
      const topRegion = highlights.top_region;
      const topSite = highlights.top_site;
      const topEvent = highlights.top_event;
      const overallState = state.passiveOperationalVisibility?.overall_state ?? null;
      const vis = state.passiveOperationalVisibility;
      target.innerHTML = `
        ${overallState ? `<div class="card">
          <h4>Fleet State</h4>
          <div class="meta">
            <span class="pill ${visibilityStateClass(overallState)}">${escapeHtml(overallState)}</span>
            <span>${vis.total_regions ?? 0} regions</span>
            <span>${vis.active_workers ?? 0} active workers</span>
            <span>${vis.active_leases ?? 0} active leases</span>
            ${(vis.stale_leases ?? 0) > 0 ? `<span class="pill danger">${vis.stale_leases} stale leases</span>` : ""}
            ${(vis.stale_workers ?? 0) > 0 ? `<span class="pill warn">${vis.stale_workers} stale workers</span>` : ""}
          </div>
          ${vis.narrative ? `<div class="meta" style="margin-top:8px">${escapeHtml(vis.narrative)}</div>` : ""}
        </div>` : ""}
        ${topRegion ? `<div class="card">
          <h4>Top Region: ${escapeHtml(topRegion.name)}</h4>
          <div class="meta">
            <span class="${pillClass(operationalPriorityClass(topRegion.operational_pressure_priority))}">${escapeHtml(topRegion.operational_pressure_priority)}</span>
            <span>${escapeHtml(topRegion.dominant_status ?? "no_canonical_state")}</span>
          </div>
          <div class="meta" style="margin-top:8px">${escapeHtml(topRegion.narrative_summary)}</div>
          ${renderRegionVisibilityCard(topRegion.region_id, false)}
          <div class="row" style="margin-top:8px">
            <button class="secondary passive-open-region-overview" data-region-id="${escapeHtml(topRegion.region_id)}">Open Region</button>
            <button class="secondary passive-open-region-runs" data-region-id="${escapeHtml(topRegion.region_id)}">View Runs</button>
            <button class="secondary passive-open-worker-diagnostics" data-region-id="${escapeHtml(topRegion.region_id)}">Diagnostics</button>
          </div>
        </div>` : `<div class="empty">No top region data.</div>`}
        ${topSite ? `<div class="card">
          <h4>Top Site: ${escapeHtml(topSite.name)}</h4>
          <div class="meta">
            <span>${escapeHtml(topSite.site_type)}</span>
            <span>risk ${(topSite.risk_score * 100).toFixed(0)}%</span>
            <span>${escapeHtml(topSite.risk_trend)}</span>
            <span>${escapeHtml(topSite.top_canonical_status ?? "no_canonical_state")}</span>
          </div>
          <div class="row" style="margin-top:8px">
            <button class="secondary passive-refocus-site" data-site-id="${escapeHtml(topSite.site_id)}">Focus Site</button>
          </div>
        </div>` : ""}
        ${topEvent ? `<div class="card">
          <h4>Top Event: ${escapeHtml(topEvent.event_type)}</h4>
          <div class="meta">
            <span>${escapeHtml(topEvent.site_name)}</span>
            <span>${escapeHtml(topEvent.status)}</span>
            <span>${escapeHtml(topEvent.temporal_phase)}</span>
          </div>
          <div class="meta" style="margin-top:8px">${escapeHtml(topEvent.operational_readout)}</div>
          <div class="row" style="margin-top:8px">
            <button class="secondary passive-refocus-canonical" data-canonical-id="${escapeHtml(topEvent.canonical_event_id)}" data-site-id="${escapeHtml(topEvent.site_id)}">Focus Event</button>
          </div>
        </div>` : ""}
        <div class="card">
          <h4>Summary</h4>
          <div class="meta">${escapeHtml(commandCenter.summary)}</div>
        </div>`;
    }

    function operationalTrend(timeline) {
      const buckets = timeline?.buckets || [];
      const activeBuckets = buckets.filter((bucket) => (bucket.run_count || bucket.source_sample_count || bucket.source_failure_count));
      if (activeBuckets.length < 2) return "stable";
      const recent = activeBuckets.slice(-2).reduce((sum, bucket) => sum + (bucket.pressure_score || 0), 0) / Math.min(2, activeBuckets.length);
      const previousSlice = activeBuckets.slice(0, Math.max(1, activeBuckets.length - 2));
      const previous = previousSlice.reduce((sum, bucket) => sum + (bucket.pressure_score || 0), 0) / previousSlice.length;
      if (recent - previous >= 0.18) return "rising";
      if (previous - recent >= 0.18) return "cooling";
      return "stable";
    }

    function renderOperationalTimeline(timeline) {
      const buckets = timeline?.buckets || [];
      const snapshot = timeline?.current_snapshot;
      const trend = operationalTrend(timeline);
      const visibleBuckets = buckets.slice(-8);
      return `
        <div class="card">
          <h4>Operational Timeline</h4>
          <div class="meta">
            <span class="${pillClass(operationalPriorityClass(snapshot?.pressure_priority ?? "low"))}">now ${escapeHtml(snapshot?.pressure_priority ?? "low")}</span>
            <span>trend ${escapeHtml(trend)}</span>
            <span>${timeline?.window_hours ?? 0}h window</span>
            <span>${timeline?.bucket_hours ?? 0}h buckets</span>
          </div>
          <div class="timeline-strip" style="margin-top:10px">
            ${visibleBuckets.map((bucket) => `
              <span class="${pillClass(operationalPriorityClass(bucket.pressure_priority))}" title="${escapeHtml(bucket.summary)}">
                ${escapeHtml(bucket.pressure_priority)} ${Math.round((bucket.pressure_score || 0) * 100)}%
              </span>`).join("") || `<span class="pill">no buckets</span>`}
          </div>
          <div class="meta" style="margin-top:8px">
            ${(visibleBuckets.find((bucket) => (bucket.suggested_read_paths || []).length)?.suggested_read_paths || []).slice(0, 4).map((path) => `<span>${escapeHtml(path)}</span>`).join("") || `<span>no bucket read paths</span>`}
          </div>
          <div class="meta" style="margin-top:10px">
            <span>${snapshot?.active_lease_count ?? 0} active leases</span>
            <span>${snapshot?.stale_lease_count ?? 0} stale leases</span>
            <span>${snapshot?.active_worker_count ?? 0} active workers</span>
            <span>${snapshot?.stale_worker_count ?? 0} stale workers</span>
          </div>
          <div class="meta" style="margin-top:8px">
            <span>last run ${escapeHtml(snapshot?.latest_run_status ?? "none")}</span>
            <span>${formatEpoch(snapshot?.latest_run_finished_at_unix_seconds)}</span>
          </div>
          <div class="meta" style="margin-top:8px">${escapeHtml(snapshot?.summary ?? "No operational snapshot yet.")}</div>
        </div>`;
    }

    function renderOperationalVisibility(data) {
      if (!data) return `<div class="empty">No operational visibility data.</div>`;
      const regions = data.regions || [];
      const workers = data.worker_heartbeats || [];
      const activeLeases = data.active_region_leases || [];
      const staleLeases = data.stale_region_leases || [];
      return `
        <div class="card">
          <h4>Overall State</h4>
          <div class="meta">
            <span class="pill ${visibilityStateClass(data.overall_state)}">${escapeHtml(data.overall_state ?? "healthy")}</span>
            <span>${data.total_regions ?? regions.length} regions</span>
            <span>${data.active_workers ?? workers.length} active workers</span>
            <span>${data.active_leases ?? activeLeases.length} active leases</span>
            ${(data.stale_leases ?? staleLeases.length) > 0 ? `<span class="pill danger">${data.stale_leases ?? staleLeases.length} stale leases</span>` : ""}
            ${(data.stale_workers ?? 0) > 0 ? `<span class="pill warn">${data.stale_workers} stale workers</span>` : ""}
          </div>
          ${data.narrative ? `<div class="meta" style="margin-top:8px">${escapeHtml(data.narrative)}</div>` : ""}
          <div class="row" style="margin-top:10px">
            ${(data.panel_paths || []).slice(0, 2).map((path) => `<button class="secondary passive-open-path" data-path="${escapeHtml(path)}">Open Panel</button>`).join("")}
          </div>
        </div>
        ${regions.slice(0, 5).map((region) => `
          <div class="card">
            <h4>${escapeHtml(region.name ?? region.region_id)}</h4>
            <div class="meta">
              <span class="pill ${visibilityStateClass(region.state)}">${escapeHtml(region.state ?? "healthy")}</span>
              <span class="${pillClass(operationalPriorityClass(region.pressure_priority))}">${escapeHtml(region.pressure_priority ?? "low")}</span>
              ${region.latest_run_status ? `<span>${escapeHtml(region.latest_run_status)}</span>` : ""}
            </div>
            <div class="meta" style="margin-top:8px">
              <span>${region.drivers?.active_worker_count ?? 0} active workers</span>
              <span>${region.drivers?.stale_worker_count ?? 0} stale workers</span>
              <span>${region.drivers?.active_lease_count ?? 0} active leases</span>
              <span>${region.drivers?.stale_lease_count ?? 0} stale leases</span>
            </div>
            <div class="meta" style="margin-top:8px">
              <span>${region.drivers?.run_count ?? 0} runs</span>
              <span>${region.drivers?.recent_failed_run_count ?? 0} failed</span>
              <span>${region.drivers?.recent_partial_run_count ?? 0} partial</span>
              ${region.drivers?.cadence_gap_seconds != null ? `<span>gap ${escapeHtml(formatSeconds(region.drivers.cadence_gap_seconds))}</span>` : ""}
            </div>
            ${region.narrative ? `<div class="meta" style="margin-top:8px">${escapeHtml(region.narrative)}</div>` : ""}
            <div class="row" style="margin-top:8px">
              <button class="secondary passive-open-worker-diagnostics" data-region-id="${escapeHtml(region.region_id)}">Diagnostics</button>
              <button class="secondary passive-open-region-runs" data-region-id="${escapeHtml(region.region_id)}">Runs</button>
            </div>
          </div>`).join("") || `<div class="empty">No regions in this scope.</div>`}
        ${workers.slice(0, 3).map((hb) => `
          <button class="secondary passive-open-heartbeat" data-worker-id="${escapeHtml(hb.worker_id)}" style="text-align:left">
            <span class="meta">
              <span>${escapeHtml(hb.worker_id.slice(0, 20))}</span>
              <span>${escapeHtml(hb.status)}</span>
              <span>${escapeHtml(hb.current_phase)}</span>
              <span>${formatEpoch(hb.last_heartbeat_unix_seconds)}</span>
            </span>
          </button>`).join("")}
        ${staleLeases.slice(0, 3).map((lease) => `
          <div class="card">
            <h4>${escapeHtml(lease.region_id)}</h4>
            <div class="meta">
              <span class="pill danger">stale lease</span>
              <span>${escapeHtml(lease.worker_id.slice(0, 20))}</span>
              <span>${lease.heartbeat_lag_seconds}s lag</span>
            </div>
          </div>`).join("")}`;
    }

    function dashboardSemanticFilterButtons(activeFilter) {
      const filters = [
        { value: "all", label: "All" },
        { value: "new", label: "New" },
        { value: "recurring", label: "Recurring" },
        { value: "escalating", label: "Escalating" },
        { value: "cooling", label: "Cooling" },
      ];
      return filters.map((filter) => `
        <button class="${activeFilter === filter.value ? "" : "secondary "}passive-semantic-filter" data-filter="${filter.value}">
          ${filter.label}
          </button>`).join("");
      }

    function dashboardPressureFilterButtons(activeFilter) {
      const filters = [
        { value: "all", label: "All" },
        { value: "low", label: "Low" },
        { value: "medium", label: "Medium" },
        { value: "high", label: "High" },
        { value: "critical", label: "Critical" },
      ];
        return filters.map((filter) => `
        <button class="${activeFilter === filter.value ? "" : "secondary "}passive-pressure-filter" data-filter="${filter.value}">
          ${filter.label}
          </button>`).join("");
    }

    function dashboardAttentionKindButtons(activeFilter) {
      const filters = [
        { value: "all", label: "All" },
        { value: "maintenance", label: "Maintenance" },
        { value: "canonical_event", label: "Events" },
        { value: "site", label: "Sites" },
        { value: "region", label: "Regions" },
      ];
      return filters.map((filter) => `
        <button class="${activeFilter === filter.value ? "" : "secondary "}passive-attention-kind-filter" data-filter="${filter.value}">
          ${filter.label}
        </button>`).join("");
    }

    function dashboardAttentionPriorityButtons(activeFilter) {
      const filters = [
        { value: "all", label: "All" },
        { value: "medium", label: "Medium+" },
        { value: "high", label: "High+" },
        { value: "critical", label: "Critical" },
      ];
      return filters.map((filter) => `
        <button class="${activeFilter === filter.value ? "" : "secondary "}passive-attention-priority-filter" data-filter="${filter.value}">
          ${filter.label}
        </button>`).join("");
    }

    function updatePassiveMapSummary(region, sites, canonicalEvents, visibleRegionCount) {
      const focusedEvent = state.selectedPassiveCanonicalEventId
        ? canonicalEvents.find((event) => event.canonical_event_id === state.selectedPassiveCanonicalEventId)
        : null;
      const focusedSite = state.selectedPassiveSiteId
        ? sites.find((site) => site.site_id === state.selectedPassiveSiteId)
        : null;
      const highlightedSiteIds = new Set(
        canonicalEvents
          .map((event) => event.site_id)
          .filter(Boolean)
      );
      const operationalTimeline = state.selectedPassiveOperationalTimeline;
      const operationalSnapshot = operationalTimeline?.current_snapshot;
      state.passiveMapSummary = {
        visibleRegionCount: visibleRegionCount ?? 0,
        visibleSiteCount: sites.length,
        visibleCanonicalEventCount: canonicalEvents.length,
        highlightedSiteCount: highlightedSiteIds.size,
        selectedRegionId: state.selectedPassiveRegionId,
        selectedRegionName: region?.name ?? null,
        selectedSiteId: state.selectedPassiveSiteId,
        selectedSiteName: focusedSite?.name ?? null,
        selectedEventId: state.selectedPassiveCanonicalEventId,
        selectedEventType: focusedEvent?.event_type ?? null,
        selectedEventStatus: focusedEvent?.status ?? null,
        operationalTrend: operationalTrend(operationalTimeline),
        operationalPriority: operationalSnapshot?.pressure_priority ?? null,
      };
      return state.passiveMapSummary;
    }

    function renderPassiveCommandContext(maintenance, commandCenter) {
      const summary = state.passiveMapSummary || {
        visibleRegionCount: 0,
        visibleSiteCount: 0,
        visibleCanonicalEventCount: 0,
        highlightedSiteCount: 0,
        selectedRegionId: state.selectedPassiveRegionId,
        selectedRegionName: null,
        selectedSiteId: state.selectedPassiveSiteId,
        selectedSiteName: null,
        selectedEventId: state.selectedPassiveCanonicalEventId,
        selectedEventType: null,
        selectedEventStatus: null,
        operationalTrend: state.selectedPassiveOperationalTimeline ? operationalTrend(state.selectedPassiveOperationalTimeline) : null,
        operationalPriority: state.selectedPassiveOperationalTimeline?.current_snapshot?.pressure_priority ?? null,
      };
      const semanticFilter = state.passiveSemanticFilter || "all";
      const pressureFilter = state.passivePressureFilter || "all";
      const highlights = commandCenter?.highlights || {};
      const topRegion = highlights.top_region || null;
      const topSite = highlights.top_site || null;
      const topEvent = highlights.top_event || null;
      const operatorPaths = commandCenter?.operator_paths || [];
      const focusPaths = commandCenter?.focus_paths || [];
      return `
        <div class="card">
          <h4>Command Context</h4>
          <div class="meta">
            <span class="pill">${escapeHtml(semanticFilter === "all" ? "semantics all" : `semantics ${semanticFilter}`)}</span>
            <span class="pill">${escapeHtml(pressureFilter === "all" ? "pressure all" : `pressure ${pressureFilter}`)}</span>
            <span>${escapeHtml(summary.selectedRegionName ?? summary.selectedRegionId ?? "no selected region")}</span>
          </div>
          <div class="meta" style="margin-top:8px">
            <span>${summary.visibleRegionCount ?? 0} visible regions</span>
            <span>${summary.visibleSiteCount ?? 0} visible sites</span>
            <span>${summary.visibleCanonicalEventCount ?? 0} visible canonical events</span>
            <span>${summary.highlightedSiteCount ?? 0} highlighted sites</span>
          </div>
          <div class="meta" style="margin-top:8px">
            <span>site ${escapeHtml(summary.selectedSiteName ?? summary.selectedSiteId ?? "none")}</span>
            <span>${summary.selectedEventType ? `${escapeHtml(summary.selectedEventType)} ${escapeHtml(summary.selectedEventStatus ?? "")}` : "no focused phenomenon"}</span>
          </div>
          <div class="meta" style="margin-top:8px">
            <span>ops ${escapeHtml(summary.operationalTrend ?? "n/a")}</span>
            <span>now ${escapeHtml(summary.operationalPriority ?? "n/a")}</span>
            <span>${maintenance?.stale_heartbeat_count ?? 0} stale heartbeat candidates</span>
            <span>${maintenance?.source_health_prune_candidate_count ?? 0} source prune candidates</span>
          </div>
          <div class="meta" style="margin-top:8px">
            <span>top_region ${escapeHtml(topRegion?.name ?? "none")}</span>
            <span>top_site ${escapeHtml(topSite?.name ?? "none")}</span>
            <span>top_event ${escapeHtml(topEvent?.event_type ?? "none")}</span>
          </div>
          ${commandCenter?.summary ? `<div class="meta" style="margin-top:8px"><span>${escapeHtml(commandCenter.summary)}</span></div>` : ""}
          ${topEvent?.operational_readout ? `<div class="meta" style="margin-top:8px"><span>${escapeHtml(topEvent.operational_readout)}</span></div>` : ""}
          <div class="meta" style="margin-top:8px">
            <span>Operator Paths</span>
            ${operatorPaths.slice(0, 3).map((path) => `<span>${escapeHtml(path)}</span>`).join("") || `<span>none</span>`}
          </div>
          <div class="meta" style="margin-top:8px">
            <span>Focus Paths</span>
            ${focusPaths.slice(0, 3).map((path) => `<span>${escapeHtml(path)}</span>`).join("") || `<span>none</span>`}
          </div>
          <div class="row" style="margin-top:10px">
            ${semanticFilter !== "all" ? `<button class="secondary passive-reset-semantic-filter">Show All Semantics</button>` : ""}
            ${pressureFilter !== "all" ? `<button class="secondary passive-reset-pressure-filter">Show All Pressure</button>` : ""}
            ${summary.selectedEventId ? `<button class="secondary passive-refocus-canonical" data-canonical-id="${escapeHtml(summary.selectedEventId)}" data-site-id="${escapeHtml(summary.selectedSiteId ?? "")}">Focus Selected Event</button>` : ""}
            ${summary.selectedSiteId ? `<button class="secondary passive-refocus-site" data-site-id="${escapeHtml(summary.selectedSiteId)}">Focus Selected Site</button>` : ""}
            ${summary.selectedRegionId ? `<button class="secondary passive-open-region-overview" data-region-id="${escapeHtml(summary.selectedRegionId)}">Open Selected Region</button>` : ""}
            ${topEvent?.canonical_event_id ? `<button class="secondary passive-refocus-canonical" data-canonical-id="${escapeHtml(topEvent.canonical_event_id)}" data-site-id="${escapeHtml(topEvent.site_id ?? "")}">Focus Top Event</button>` : ""}
            ${topSite?.site_id ? `<button class="secondary passive-refocus-site" data-site-id="${escapeHtml(topSite.site_id)}">Focus Top Site</button>` : ""}
            ${topRegion?.region_id ? `<button class="secondary passive-open-region-overview" data-region-id="${escapeHtml(topRegion.region_id)}">Open Top Region</button>` : ""}
          </div>
        </div>`;
    }

    function refreshPassiveDashboardSurface() {
      if (state.passiveDashboardSummary && state.passiveMaintenanceSummary) {
        renderPassiveDashboard(
          state.passiveDashboardSummary,
          state.passiveMaintenanceSummary,
          state.passiveCommandCenterSummary,
        );
      }
    }

    function attentionItemDataAttrs(item) {
      return `data-attention-item-id="${escapeHtml(item.item_id ?? "")}"
        data-attention-kind="${escapeHtml(item.kind ?? "")}"
        data-attention-priority="${escapeHtml(item.priority ?? "")}"
        data-attention-title="${escapeHtml(item.title ?? "")}"
        data-attention-reason="${escapeHtml(item.reason ?? "")}"
        data-attention-action-label="${escapeHtml(item.primary_action_label ?? "")}"
        data-region-id="${escapeHtml(item.region_id ?? "")}"
        data-site-id="${escapeHtml(item.site_id ?? "")}"
        data-canonical-id="${escapeHtml(item.canonical_event_id ?? "")}"`;
    }

    function setPassiveSelectedAttentionFromDataset(dataset) {
      if (!dataset?.attentionItemId) return false;
      state.passiveSelectedAttention = {
        item_id: dataset.attentionItemId,
        kind: dataset.attentionKind || null,
        priority: dataset.attentionPriority || null,
        title: dataset.attentionTitle || null,
        reason: dataset.attentionReason || null,
        primary_action_label: dataset.attentionActionLabel || null,
        region_id: dataset.regionId || null,
        site_id: dataset.siteId || null,
        canonical_event_id: dataset.canonicalId || null,
      };
      refreshPassiveDashboardSurface();
      return true;
    }

    function selectedPassiveAttentionItem() {
      const attention = state.passiveSelectedAttention;
      if (!attention?.item_id) return null;
      const queue = state.passiveCommandCenterSummary?.attention_queue || [];
      return queue.find((item) => item.item_id === attention.item_id) || attention;
    }

    function renderPassiveAttentionPrimaryAction(item) {
      if (!item) return "";
      if (item.kind === "maintenance" && item.item_id?.includes("prune-stale-heartbeats")) {
        return `<button class="secondary passive-prune-stale-heartbeats" ${attentionItemDataAttrs(item)}>${escapeHtml(item.primary_action_label ?? "Prune Heartbeats")}</button>`;
      }
      if (item.kind === "maintenance" && item.item_id?.includes("prune-source-health-samples")) {
        return `<button class="secondary passive-preview-source-prune" data-source-kind="" ${attentionItemDataAttrs(item)}>${escapeHtml(item.primary_action_label ?? "Preview Prune")}</button>`;
      }
      if (item.kind === "canonical_event" && item.canonical_event_id) {
        return `<button class="secondary passive-refocus-canonical" ${attentionItemDataAttrs(item)}>${escapeHtml(item.primary_action_label ?? "Focus Event")}</button>`;
      }
      if (item.kind === "site" && item.site_id) {
        return `<button class="secondary passive-refocus-site" ${attentionItemDataAttrs(item)}>${escapeHtml(item.primary_action_label ?? "Focus Site")}</button>`;
      }
      if (item.kind === "region" && item.region_id) {
        return `<button class="secondary passive-open-region-overview" ${attentionItemDataAttrs(item)}>${escapeHtml(item.primary_action_label ?? "Open Region")}</button>`;
      }
      return "";
    }

    function renderPassiveSelectedAttentionContext() {
      const attention = selectedPassiveAttentionItem();
      if (!attention?.item_id) return "";
      const targetSummary = [
        attention.region_id ? `region ${attention.region_id}` : "",
        attention.site_id ? `site ${attention.site_id}` : "",
        attention.canonical_event_id ? `event ${attention.canonical_event_id}` : "",
      ].filter(Boolean).join(" • ");
      const primaryAction = renderPassiveAttentionPrimaryAction(attention);
      const quickActions = [
        attention.canonical_event_id ? `<button class="secondary passive-refocus-canonical" ${attentionItemDataAttrs(attention)}>Focus Event</button>` : "",
        attention.site_id ? `<button class="secondary passive-refocus-site" ${attentionItemDataAttrs(attention)}>Focus Site</button>` : "",
        attention.region_id ? `<button class="secondary passive-open-region-overview" ${attentionItemDataAttrs(attention)}>Open Region</button>` : "",
      ].filter(Boolean).join("");
      const readActions = (attention.confirmation_read_paths || []).length
        ? readPathButtons(attention.confirmation_read_paths, attention.region_id, attention)
        : "";
      return `
        <div class="card passive-selected-attention-context">
          <h4>Selected Attention</h4>
          <div class="meta">
            <span class="${pillClass(attention.priority === "critical" || attention.priority === "high" ? "failed" : attention.priority === "medium" ? "Warning" : "ok")}">${escapeHtml(attention.priority ?? "low")}</span>
            <span>${escapeHtml(attention.kind ?? "unknown")}</span>
            <span>${escapeHtml(attention.title ?? "Untitled")}</span>
          </div>
          <div class="meta" style="margin-top:8px">
            <span>${escapeHtml(attention.reason ?? "No operational reason provided.")}</span>
          </div>
          ${targetSummary ? `<div class="meta" style="margin-top:8px"><span>${escapeHtml(targetSummary)}</span></div>` : ""}
          ${attention.primary_action_label ? `<div class="meta" style="margin-top:8px"><span>action ${escapeHtml(attention.primary_action_label)}</span></div>` : ""}
          ${primaryAction ? `<div class="row passive-selected-attention-actions" style="margin-top:10px">${primaryAction}</div>` : ""}
          ${quickActions ? `<div class="row passive-selected-attention-actions" style="margin-top:8px">${quickActions}</div>` : ""}
          ${readActions ? `<div class="row passive-selected-attention-read-actions" style="margin-top:8px">${readActions}</div>` : ""}
          <div class="row" style="margin-top:8px">
            <button class="secondary passive-clear-selected-attention">Clear Selection</button>
          </div>
        </div>`;
    }
    function renderPassiveAttentionQueue(commandCenter) {
      const queue = commandCenter?.attention_queue || [];
      const kindFilter = state.passiveAttentionKindFilter || "all";
      const priorityFilter = state.passiveAttentionPriorityFilter || "all";
      const attentionMapActive = state.passiveAttentionMapMode ? "on" : "off";
      return `
        <div class="card">
          <h4>Attention Queue</h4>
          <div class="meta">
            <span class="pill">${escapeHtml(kindFilter === "all" ? "attention all" : `attention ${kindFilter}`)}</span>
            <span class="pill">${escapeHtml(priorityFilter === "all" ? "priority all" : `priority ${priorityFilter}+`)}</span>
          </div>
          <div class="row" style="margin-top:10px">${dashboardAttentionKindButtons(kindFilter)}</div>
          <div class="row" style="margin-top:10px">${dashboardAttentionPriorityButtons(priorityFilter)}</div>
          <div class="row" style="margin-top:10px">
            <button class="${state.passiveAttentionMapMode ? "" : "secondary "}passive-toggle-attention-map">Attention Map ${attentionMapActive}</button>
            ${kindFilter !== "all" ? `<button class="secondary passive-reset-attention-kind-filter">Show All Attention</button>` : ""}
            ${priorityFilter !== "all" ? `<button class="secondary passive-reset-attention-priority-filter">Show All Priorities</button>` : ""}
          </div>
          <div class="list" style="margin-top:10px">
            ${queue.length ? queue.slice(0, 6).map((item) => `
              <div class="card ${item.kind !== "maintenance" ? "passive-attention-item-card" : ""} ${state.passiveSelectedAttention?.item_id === item.item_id ? "passive-attention-item-selected" : ""}" ${attentionItemDataAttrs(item)}>
                <div class="meta">
                  <span class="${pillClass(item.priority === "critical" || item.priority === "high" ? "failed" : item.priority === "medium" ? "Warning" : "ok")}">${escapeHtml(item.priority ?? "low")}</span>
                  <span>${escapeHtml(item.kind ?? "unknown")}</span>
                  <span>${escapeHtml(item.title ?? "Untitled")}</span>
                </div>
                <div class="meta" style="margin-top:8px">
                  <span>${escapeHtml(item.reason ?? "No reason provided.")}</span>
                </div>
                ${(item.region_id || item.site_id || item.canonical_event_id) ? `<div class="meta" style="margin-top:8px">
                  ${item.region_id ? `<span>region ${escapeHtml(item.region_id)}</span>` : ""}
                  ${item.site_id ? `<span>site ${escapeHtml(item.site_id)}</span>` : ""}
                  ${item.canonical_event_id ? `<span>event ${escapeHtml(item.canonical_event_id)}</span>` : ""}
                </div>` : ""}
              <div class="row" style="margin-top:8px">
                ${renderPassiveAttentionPrimaryAction(item)}
              </div>
              ${(item.confirmation_read_paths || []).length ? `<div class="row" style="margin-top:8px">${readPathButtons(item.confirmation_read_paths, item.region_id, item)}</div>` : ""}
              ${(item.confirmation_read_paths || []).length ? `<div class="meta" style="margin-top:8px">${(item.confirmation_read_paths || []).slice(0, 3).map((path) => `<span>${escapeHtml(path)}</span>`).join("")}</div>` : ""}
            </div>
          `).join("") : `<div class="empty">No operational attention items.</div>`}
          </div>
        </div>`;
    }

    function clearPassiveSelectedAttention(statusLabel) {
      state.passiveSelectedAttention = null;
      refreshPassiveDashboardSurface();
      const selectedContext = $("passive-focus")?.querySelector(".passive-selected-attention-context");
      if (selectedContext) {
        selectedContext.remove();
      }
      if (statusLabel) {
        $("refresh-status").textContent = statusLabel;
      }
    }

    async function handlePassiveRefocusCanonicalButton(button, statusLabel) {
      setPassiveSelectedAttentionFromDataset(button.dataset);
      await focusPassiveCanonicalFromAttention(
        button.dataset.canonicalId,
        button.dataset.siteId,
        button.dataset.regionId,
        statusLabel || "Focused canonical event",
      );
    }

    async function handlePassiveRefocusSiteButton(button, statusLabel) {
      setPassiveSelectedAttentionFromDataset(button.dataset);
      await focusPassiveSiteFromAttention(
        button.dataset.siteId,
        button.dataset.regionId,
        statusLabel || "Focused site",
      );
    }

    function handlePassiveOpenPathButton(button, statusLabel, syncAttention) {
      if (syncAttention !== false) {
        setPassiveSelectedAttentionFromDataset(button.dataset);
      }
      openPassivePath(button.dataset.path, statusLabel || "Opened read path");
    }

    function handlePassiveOpenEvidenceButton(button, statusLabel, syncAttention) {
      if (syncAttention !== false) {
        setPassiveSelectedAttentionFromDataset(button.dataset);
      }
      openPassiveEvidence(button.dataset.bundleHash, statusLabel || "Opened evidence");
    }

    async function handlePassiveRunReplayButton(button, statusLabel, syncAttention) {
      if (syncAttention !== false) {
        setPassiveSelectedAttentionFromDataset(button.dataset);
      }
      button.disabled = true;
      try {
        await runPassiveReplay(button.dataset.manifestHash, statusLabel || "Passive replay");
      } catch (error) {
        $("refresh-status").textContent = error.message;
      } finally {
        button.disabled = false;
      }
    }

    async function handlePassiveOpenRegionOverviewButton(button, statusLabel, syncAttention) {
      if (syncAttention !== false) {
        setPassiveSelectedAttentionFromDataset(button.dataset);
      }
      await openPassiveRegionOverview(button.dataset.regionId, statusLabel || "Opened region");
    }

    function passiveAttentionSets() {
      const queue = state.passiveCommandCenterSummary?.attention_queue || [];
      const regionIds = new Set();
      const siteIds = new Set();
      const canonicalIds = new Set();
      queue.forEach((item) => {
        if (item.region_id) regionIds.add(item.region_id);
        if (item.site_id) siteIds.add(item.site_id);
        if (item.canonical_event_id) canonicalIds.add(item.canonical_event_id);
      });
      return {
        regionIds,
        siteIds,
        canonicalIds,
        hasMapTargets: regionIds.size > 0 || siteIds.size > 0 || canonicalIds.size > 0,
      };
    }

    function attentionModeAppliesToSite(site, attentionSets) {
      if (!state.passiveAttentionMapMode || !attentionSets.hasMapTargets) return true;
      if (!attentionSets.siteIds.size && !attentionSets.canonicalIds.size) return true;
      return Boolean(site.site_id && attentionSets.siteIds.has(site.site_id));
    }

    function attentionModeAppliesToEvent(event, attentionSets) {
      if (!state.passiveAttentionMapMode || !attentionSets.hasMapTargets) return true;
      if (!attentionSets.canonicalIds.size && !attentionSets.siteIds.size) return true;
      return attentionSets.canonicalIds.has(event.canonical_event_id)
        || Boolean(event.site_id && attentionSets.siteIds.has(event.site_id));
    }

    function renderPassiveDashboard(data, maintenance, commandCenter) {
      const health = data.worker_health || {};
      const sourceHealth = data.source_health || [];
      const ops = data.operations || {};
      const maintenanceActions = maintenance.suggested_actions || [];
      const topRegions = data.top_regions || [];
      const filteredTopRegions = topRegions.filter(matchesPassivePressureFilter);
      const topEvents = data.top_canonical_events || [];
      const topSites = data.top_sites || [];
      const semanticTimeline = data.semantic_timeline || [];
      $("passive-dashboard").innerHTML = `
            ${renderPassiveCommandContext(maintenance, commandCenter)}
            ${renderPassiveAttentionQueue(commandCenter)}
            <div class="card">
              <h4>Autonomous Observation</h4>
            <div class="meta">
              <span class="${pillClass(health.status === "healthy" ? "ok" : "Warning")}">${health.status ?? "unknown"}</span>
              <span>${ops.enabled_region_count ?? 0} active regions</span>
              <span>${ops.seed_count ?? 0} seeds</span>
              <span>${ops.elevated_seed_count ?? 0} elevated</span>
            </div>
            <div class="meta" style="margin-top:8px">
              <span>Active Leases: ${health.active_region_lease_count ?? ops.active_region_lease_count ?? 0}</span>
              <span>Worker Heartbeats: ${health.worker_heartbeat_count ?? ops.worker_heartbeat_count ?? 0}</span>
              <span>Active Workers: ${health.active_worker_count ?? 0}</span>
              <span>Stale Workers: ${health.stale_worker_count ?? 0}</span>
            </div>
            <div class="meta" style="margin-top:8px">
              <span>Stale Leases: ${ops.stale_region_lease_count ?? 0}</span>
              <span>Stale Heartbeats: ${ops.stale_worker_heartbeat_count ?? 0}</span>
            </div>
            <div class="row" style="margin-top:10px">
              <button class="secondary passive-open-worker-diagnostics">Worker Diagnostics</button>
              <button class="secondary passive-open-heartbeat-list">View Heartbeats</button>
              ${(ops.stale_worker_heartbeat_count ?? 0) > 0 ? `<button class="secondary passive-open-stale-heartbeats">View Stale Heartbeats</button>` : ""}
            </div>
            <div class="row" style="margin-top:10px">${dashboardPressureFilterButtons(state.passivePressureFilter || "all")}</div>
            <div class="meta" style="margin-top:8px">
              <span>Running Regions:</span>
              <span>${(health.running_region_ids || []).length ? (health.running_region_ids || []).slice(0, 5).map((regionId) => escapeHtml(regionId)).join(", ") : "none"}</span>
            </div>
            <div class="list" style="margin-top:10px">
              ${((ops.active_region_leases || []).slice(0, 3).map((lease) => `
                <button class="secondary passive-open-lease" data-region-id="${escapeHtml(lease.region_id)}" style="text-align:left">
                  <span class="meta">
                    <span>${escapeHtml(lease.region_id)}</span>
                    <span>${escapeHtml(lease.worker_id)}</span>
                    <span>${formatEpoch(lease.expires_at_unix_seconds)}</span>
                  </span>
                </button>`)).join("") || `<div class="empty">No active leases.</div>`}
            </div>
            <div class="list" style="margin-top:10px">
              ${((ops.worker_heartbeats || []).slice(0, 3).map((heartbeat) => `
                <button class="secondary passive-open-heartbeat" data-worker-id="${escapeHtml(heartbeat.worker_id)}" style="text-align:left">
                  <span class="meta">
                    <span>${escapeHtml(heartbeat.worker_id)}</span>
                    <span>${escapeHtml(heartbeat.status)}</span>
                    <span>${escapeHtml(heartbeat.current_phase)}</span>
                    <span>${formatEpoch(heartbeat.last_heartbeat_unix_seconds)}</span>
                  </span>
                </button>`)).join("") || `<div class="empty">No worker heartbeats.</div>`}
            </div>
              <div class="meta" style="margin-top:8px">${data.narrative}</div>
            </div>
            <div class="card">
              <h4>Maintenance</h4>
              <div class="meta">
                <span>${maintenance.stale_heartbeat_count ?? 0} stale heartbeat candidates</span>
                <span>${maintenance.source_health_prune_candidate_count ?? 0} source prune candidates</span>
              </div>
              <div class="meta" style="margin-top:8px">
                <span>heartbeat retention ${formatSeconds(maintenance.heartbeat_retention_seconds ?? 0)}</span>
                <span>source retention ${formatSeconds(maintenance.source_retention_seconds ?? 0)}</span>
              </div>
              <div class="row" style="margin-top:10px">
                ${(maintenance.stale_heartbeat_count ?? 0) > 0 ? `<button class="secondary passive-prune-stale-heartbeats">Prune Stale Heartbeats</button>` : `<button class="secondary passive-open-worker-diagnostics">Diagnostics</button>`}
                ${(maintenance.source_health_prune_candidate_count ?? 0) > 0 ? `<button class="secondary passive-preview-source-prune" data-source-kind="" data-region-id="">Preview Global Source Prune</button>` : `<button class="secondary passive-open-worker-diagnostics">No Source Prune Needed</button>`}
              </div>
              ${maintenanceActions.length ? `<div class="meta" style="margin-top:8px">${maintenanceActions.slice(0, 3).map((action) => `<span>${escapeHtml(action.title)}</span>`).join("")}</div>` : ""}
              ${maintenanceActions.length ? `<div class="meta" style="margin-top:8px">${maintenanceActions.flatMap((action) => action.confirmation_read_paths || []).slice(0, 4).map((path) => `<span>${escapeHtml(path)}</span>`).join("")}</div>` : ""}
            </div>
            <div class="card">
              <h4>Source Health</h4>
              <div class="list" style="margin-top:10px">
                ${sourceHealth.length ? sourceHealth.slice(0, 5).map((source) => `
                  <div class="meta">
                    <span>${escapeHtml(source.source)}</span>
                    <span class="${pillClass(source.health_status === "healthy" ? "ok" : source.health_status === "watch" ? "Warning" : "failed")}">${escapeHtml(source.health_status ?? "unknown")}</span>
                    <span>${Math.round((source.reliability_score ?? 0) * 100)}% reliable</span>
                    <span>${source.success_count ?? 0} ok</span>
                    <span>${source.failure_count ?? 0} fail</span>
                    <span>${source.consecutive_failure_count ?? 0} consecutive fail</span>
                  </div>
                  <div class="meta" style="margin-top:8px">
                    <span>${source.staleness_seconds != null ? `${formatSeconds(source.staleness_seconds)} old` : "no staleness"}</span>
                    <span>${source.latest_generated_at_unix_seconds ? formatEpoch(source.latest_generated_at_unix_seconds) : "no samples"}</span>
                  </div>
                  <div class="row" style="margin-top:8px">
                    <button class="secondary passive-open-source-samples" data-source-kind="${escapeHtml(source.source)}" data-region-id="${escapeHtml(source.region_id ?? "")}">View Samples</button>
                  </div>
                  <div class="empty">${escapeHtml(source.recovery_hint ?? "No recovery hint.")}</div>
                  ${source.last_error ? `<div class="empty">${escapeHtml(source.last_error)}</div>` : ""}`).join("") : `<div class="empty">No source health yet.</div>`}
              </div>
            </div>
            <div class="card">
              <h4>Top Regions</h4>
              <div class="list" style="margin-top:10px">
                ${filteredTopRegions.length ? filteredTopRegions.slice(0, 3).map((region) => `
                  <button class="secondary passive-open-worker-diagnostics" data-region-id="${escapeHtml(region.region_id)}" style="text-align:left">
                    <span class="meta">
                      <span>${escapeHtml(region.name)}</span>
                      <span>${escapeHtml(region.operational_pressure_priority ?? "low")}</span>
                      <span>${region.recent_failed_run_count ?? 0} failed</span>
                      <span>${region.recent_lease_loss_count ?? 0} lease loss</span>
                    </span>
                  </button>`).join("") : `<div class="empty">No regions in this pressure filter.</div>`}
              </div>
            </div>
            <div class="card">
              <h4>Semantic Strip</h4>
              <div class="row" style="margin-top:10px">${dashboardSemanticFilterButtons(data.semantic_filter ?? "all")}</div>
              <div class="list" style="margin-top:10px">
                ${semanticTimeline.length ? semanticTimeline.slice(0, 5).map((entry) => `
                  <button class="secondary passive-strip-entry" data-canonical-id="${escapeHtml(entry.canonical_event_id)}" data-site-id="${escapeHtml(entry.site_id)}" style="text-align:left">
                    <span class="meta">
                      <span>${entry.status}</span>
                      <span>${entry.event_type}</span>
                      <span>${entry.site_name}</span>
                      <span>${entry.support_count} signals</span>
                      ${renderRiskDeltaPill(entry.risk_delta.classification)}
                      <span>${formatEpoch(entry.last_observed_at_unix_seconds)}</span>
                    </span>
                  </button>`).join("") : `<div class="empty">No semantic entries in this filter.</div>`}
              </div>
            </div>
            <div class="card">
              <h4>Top Passive Events</h4>
              <div class="list" style="margin-top:10px">
              ${topEvents.length ? topEvents.slice(0, 3).map((event) => `
                <div class="card">
                  <div class="meta">
                    <span class="${pillClass(event.severity === "critical" || event.severity === "high" ? "failed" : "ok")}">${escapeHtml(event.severity)}</span>
                    ${renderRiskDeltaPill(event.risk_delta?.classification)}
                    <span>${escapeHtml(event.event_type)}</span>
                    <span>${escapeHtml(event.site_name)}</span>
                  </div>
                  <div class="meta" style="margin-top:6px">
                    <span>${escapeHtml(event.status)}</span>
                    <span>${event.support_count} signals</span>
                    <span>risk ${(event.risk_score * 100).toFixed(0)}%</span>
                  </div>
                  <div class="row" style="margin-top:8px">
                    <button class="secondary passive-refocus-canonical" data-canonical-id="${escapeHtml(event.canonical_event_id)}" data-site-id="${escapeHtml(event.site_id)}">Focus Event</button>
                  </div>
                </div>`).join("") : `<div class="empty">No passive events yet.</div>`}
            </div>
          </div>
          <div class="card">
            <h4>Top Sites</h4>
            <div class="list" style="margin-top:10px">
                ${topSites.length ? topSites.slice(0, 3).map((site) => `
                  <div class="meta">
                    <span>${site.name}</span>
                    <span>${site.site_type}</span>
                    <span>risk ${(site.risk_score * 100).toFixed(0)}%</span>
                    <span>${site.risk_trend}</span>
                    <span>${site.top_canonical_status ?? "no_canonical_state"}</span>
                  </div>`).join("") : `<div class="empty">No observed sites yet.</div>`}
            </div>
          </div>`;
    }

      function passiveStatusColor(status) {
        if (status === "escalating") return Cesium.Color.RED;
        if (status === "new") return Cesium.Color.ORANGE;
        if (status === "recurring") return Cesium.Color.YELLOW;
        if (status === "cooling") return Cesium.Color.CYAN;
        return Cesium.Color.LIME;
      }

    function initGlobe() {
      if (state.viewer || typeof Cesium === "undefined") return;
      state.viewer = new Cesium.Viewer("cesium-globe", {
        animation: false,
        timeline: false,
        baseLayerPicker: false,
        geocoder: false,
        sceneModePicker: false,
        homeButton: false,
        navigationHelpButton: false,
        fullscreenButton: false,
        infoBox: false,
        selectionIndicator: false,
        shouldAnimate: true,
        imageryProvider: new Cesium.OpenStreetMapImageryProvider({
          url: "https://tile.openstreetmap.org/"
        }),
      });
      state.viewer.scene.globe.enableLighting = true;
      state.viewer.scene.skyAtmosphere.show = true;
      state.viewer.clock.multiplier = 20;
      state.viewer.camera.flyHome(0);
      state.viewer.selectedEntityChanged.addEventListener((entity) => {
        void handlePassiveSelection(entity);
      });
      state.globeReady = true;
    }

    function clearGlobeEntities() {
      if (!state.viewer) return;
      state.globeEntities.forEach((entity) => state.viewer.entities.remove(entity));
      state.globeEntities = [];
      state.neoEntityIndex = {};
    }

    function clearPassiveMapEntities() {
      if (!state.viewer) return;
      state.passiveGlobeEntities.forEach((entity) => state.viewer.entities.remove(entity));
      state.passiveGlobeEntities = [];
      state.passiveFocusOverlayEntities.forEach((entity) => state.viewer.entities.remove(entity));
      state.passiveFocusOverlayEntities = [];
      state.passiveEntityIndex = {};
      state.passiveSiteEntityIndex = {};
      state.passiveSiteEventIndex = {};
    }

    function cartesianFromState(orbitalState) {
      if (!orbitalState?.position_km) return null;
      return new Cesium.Cartesian3(
        orbitalState.position_km.x * 1000,
        orbitalState.position_km.y * 1000,
        orbitalState.position_km.z * 1000
      );
    }

    function updateGlobe(objectTimeline, eventsTimeline, neoBriefing, closeApproaches) {
      if (!state.globeReady) return;
      clearGlobeEntities();
      const checkpoints = objectTimeline?.checkpoints ?? [];
      const positions = checkpoints
        .map((checkpoint) => cartesianFromState(checkpoint.predicted_state))
        .filter(Boolean);

      if (positions.length) {
        const track = state.viewer.entities.add({
          id: "sss-track",
          polyline: {
            positions,
            width: 3,
            material: Cesium.Color.AQUA,
          },
        });
        state.globeEntities.push(track);

        checkpoints.forEach((checkpoint, index) => {
          const position = cartesianFromState(checkpoint.predicted_state);
          if (!position) return;
          const point = state.viewer.entities.add({
            id: `sss-checkpoint-${index}`,
            position,
            point: {
              pixelSize: index === checkpoints.length - 1 ? 12 : 8,
              color: index === checkpoints.length - 1 ? Cesium.Color.GOLD : Cesium.Color.LIME,
              outlineColor: Cesium.Color.BLACK,
              outlineWidth: 1,
            },
            label: {
              text: `t+${checkpoint.offset_hours}h`,
              font: "12px sans-serif",
              pixelOffset: new Cesium.Cartesian2(0, -18),
              fillColor: Cesium.Color.WHITE,
              showBackground: true,
              backgroundColor: Cesium.Color.fromAlpha(Cesium.Color.BLACK, 0.55),
            },
          });
          state.globeEntities.push(point);
        });
      }

      const futureEvents = (eventsTimeline?.buckets ?? []).flatMap((bucket) => bucket.events ?? []);
      futureEvents.forEach((event, index) => {
        const predictedState = event.prediction?.predicted_state;
        const position = cartesianFromState(predictedState);
        if (!position) return;
        const marker = state.viewer.entities.add({
          id: `sss-event-${index}`,
          position,
          billboard: {
            image: "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='24' height='24'%3E%3Ccircle cx='12' cy='12' r='10' fill='%23ff8e7c' /%3E%3C/svg%3E",
            width: 12,
            height: 12,
          },
          label: {
            text: event.event_type,
            font: "11px sans-serif",
            pixelOffset: new Cesium.Cartesian2(0, 18),
            fillColor: Cesium.Color.fromCssColorString("#ffcfbf"),
            showBackground: true,
            backgroundColor: Cesium.Color.fromAlpha(Cesium.Color.BLACK, 0.55),
          },
        });
        state.globeEntities.push(marker);
      });

      if ($("toggle-neo-layer")?.checked) {
        (neoBriefing?.highest_priority ?? []).forEach((neo, index) => {
          const selectedNeo = state.selectedNeoReferenceId === neo.neo_reference_id;
          const position = Cesium.Cartesian3.fromRadians(
            (index + 1) * 0.85,
            0.32 - (index * 0.11),
            9_000_000 + (index * 700_000),
          );
          const neoEntity = state.viewer.entities.add({
            id: `sss-neo-${index}`,
            name: neo.name,
            position,
            point: {
              pixelSize: selectedNeo ? (neo.hazardous ? 18 : 14) : (neo.hazardous ? 14 : 10),
              color: neo.hazardous ? Cesium.Color.ORANGERED : Cesium.Color.CORNFLOWERBLUE,
              outlineColor: selectedNeo ? Cesium.Color.WHITE : Cesium.Color.BLACK,
              outlineWidth: selectedNeo ? 3 : 1,
            },
            label: {
              text: `${neo.name} ${(neo.priority_score * 100).toFixed(0)}%`,
              font: "11px sans-serif",
              pixelOffset: new Cesium.Cartesian2(0, 18),
              fillColor: selectedNeo ? Cesium.Color.fromCssColorString("#ffd580") : Cesium.Color.WHITE,
              showBackground: true,
              backgroundColor: Cesium.Color.fromAlpha(Cesium.Color.BLACK, 0.55),
            },
            properties: { layer: "neo-briefing", payload: neo },
            description: `${selectedNeo ? "focused | " : ""}${neo.briefing_summary}`,
          });
          state.neoEntityIndex[neo.neo_reference_id] = neoEntity;
          state.globeEntities.push(neoEntity);
        });
      }

      if ($("toggle-close-approach-layer")?.checked) {
        (closeApproaches?.close_approaches ?? []).slice(0, 3).forEach((approach, index) => {
          const primaryPosition = cartesianFromState(approach.primary_state);
          const counterpartPosition = cartesianFromState(approach.counterpart_state);
          if (!primaryPosition || !counterpartPosition) return;
          const line = state.viewer.entities.add({
            id: `sss-close-approach-line-${index}`,
            polyline: {
              positions: [primaryPosition, counterpartPosition],
              width: 2,
              material: Cesium.Color.fromCssColorString("#ffb347"),
            },
          });
          const marker = state.viewer.entities.add({
            id: `sss-close-approach-marker-${index}`,
            position: counterpartPosition,
            point: {
              pixelSize: 10,
              color: Cesium.Color.fromCssColorString("#ffb347"),
              outlineColor: Cesium.Color.BLACK,
              outlineWidth: 1,
            },
            label: {
              text: `${approach.counterpart_object_name} ${approach.miss_distance_km.toFixed(1)} km`,
              font: "11px sans-serif",
              pixelOffset: new Cesium.Cartesian2(0, 18),
              fillColor: Cesium.Color.WHITE,
              showBackground: true,
              backgroundColor: Cesium.Color.fromAlpha(Cesium.Color.BLACK, 0.55),
            },
            description: approach.summary,
          });
          state.globeEntities.push(line, marker);
        });
      }

      if (positions.length) {
        state.viewer.zoomTo(state.globeEntities);
      }
    }

    function passiveRegionColor(status, pressure = "low") {
      if (pressure === "critical" || pressure === "high") return Cesium.Color.RED.withAlpha(0.22);
      if (pressure === "medium") return Cesium.Color.ORANGE.withAlpha(0.20);
      if (status === "due") return Cesium.Color.ORANGE.withAlpha(0.22);
      if (status === "degraded") return Cesium.Color.RED.withAlpha(0.20);
      return Cesium.Color.LIME.withAlpha(0.14);
    }

    function passiveRegionPressure(region) {
      if (!region) return "low";
      if ((region.stale_lease_count ?? 0) > 0 || (region.recent_lease_loss_count ?? 0) > 0 || (region.recent_failed_run_count ?? 0) > 0) {
        return "critical";
      }
      if ((region.stale_worker_count ?? 0) > 0 || (region.recent_partial_run_count ?? 0) > 0 || region.status === "degraded" || (region.critical_events ?? 0) > 0 || (region.escalating_event_count ?? 0) > 0) {
        return "high";
      }
      if (region.status === "due" || (region.new_event_count ?? 0) > 0) {
        return "medium";
      }
      return "low";
    }

    function matchesPassivePressureFilter(region) {
      const filter = state.passivePressureFilter || "all";
      if (filter === "all") return true;
      return passiveRegionPressure(region) === filter;
    }

    function passiveSiteColor(siteType) {
      if (siteType === "solar_plant") return Cesium.Color.GOLD;
      if (siteType === "substation") return Cesium.Color.ORANGE;
      return Cesium.Color.CYAN;
    }

    function passiveEventColor(severity) {
      if (severity === "critical") return Cesium.Color.RED;
      if (severity === "high") return Cesium.Color.ORANGE;
      if (severity === "medium") return Cesium.Color.YELLOW;
      return Cesium.Color.LIME;
    }

    function passiveCanonicalPixelSize(event) {
      if (event.severity === "critical") return 18;
      if (event.severity === "high") return 15;
      return 11;
    }

    function applyPassiveSiteFocus(siteId, canonicalEventId) {
      state.selectedPassiveSiteId = siteId || null;
      state.selectedPassiveCanonicalEventId = canonicalEventId || null;
      const focusedSiteId = state.selectedPassiveSiteId;
      const focusedCanonicalEventId = state.selectedPassiveCanonicalEventId;

      state.passiveFocusOverlayEntities.forEach((entity) => state.viewer?.entities.remove(entity));
      state.passiveFocusOverlayEntities = [];

      Object.entries(state.passiveSiteEntityIndex).forEach(([candidateSiteId, entity]) => {
        const payload = cesiumPropertyValue(entity?.properties?.payload);
        if (!payload || !entity.point || !entity.label) return;
        const siteColor = passiveSiteColor(payload.site_type);
        const statusColor = payload.top_canonical_status ? passiveStatusColor(payload.top_canonical_status) : Cesium.Color.WHITE;
        const matches = focusedSiteId && candidateSiteId === focusedSiteId;
        const highlighted = !focusedSiteId
          ? payload.elevated || payload.risk_score >= 0.6
          : matches;
        entity.point.pixelSize = (payload.risk_score >= 0.75 ? 16 : payload.risk_score >= 0.45 ? 13 : 10) + (highlighted ? 3 : 0);
        entity.point.color = highlighted ? siteColor : Cesium.Color.fromAlpha(siteColor, 0.18);
        entity.point.outlineColor = highlighted ? statusColor : Cesium.Color.fromAlpha(Cesium.Color.WHITE, 0.25);
        entity.point.outlineWidth = highlighted ? 4 : 1;
        entity.label.fillColor = highlighted ? Cesium.Color.WHITE : Cesium.Color.fromAlpha(Cesium.Color.WHITE, 0.40);
        entity.label.showBackground = highlighted;
      });

      Object.entries(state.passiveEntityIndex).forEach(([_, entity]) => {
        const payload = cesiumPropertyValue(entity?.properties?.payload);
        if (!payload || !entity.point || !entity.label) return;
        const baseColor = passiveStatusColor(payload.status);
        const matchesSite = focusedSiteId && payload.site_id === focusedSiteId;
        const matchesCanonical = focusedCanonicalEventId && payload.canonical_event_id === focusedCanonicalEventId;
        const matches = matchesSite || matchesCanonical;
        const highlighted = !focusedSiteId || matches;
        entity.point.pixelSize = passiveCanonicalPixelSize(payload) + (matchesCanonical ? 6 : matches ? 4 : 0);
        entity.point.color = highlighted ? baseColor : Cesium.Color.fromAlpha(baseColor, 0.18);
        entity.point.outlineColor = matchesCanonical ? Cesium.Color.CYAN : matches ? Cesium.Color.WHITE : Cesium.Color.BLACK;
        entity.point.outlineWidth = matchesCanonical ? 4 : matches ? 3 : 2;
        entity.label.fillColor = highlighted ? Cesium.Color.WHITE : Cesium.Color.fromAlpha(Cesium.Color.WHITE, 0.35);
        entity.label.showBackground = highlighted || matchesCanonical;
      });

      if (!state.viewer || !focusedSiteId) {
        return;
      }

      const focusedSiteEntity = state.passiveSiteEntityIndex[focusedSiteId];
      const focusedSitePosition = focusedSiteEntity?.position?.getValue?.(Cesium.JulianDate.now());
      const siteEventEntities = state.passiveSiteEventIndex[focusedSiteId] || [];
      if (!focusedSitePosition || !siteEventEntities.length) {
        return;
      }

      siteEventEntities.forEach((eventEntity) => {
        const payload = cesiumPropertyValue(eventEntity?.properties?.payload);
        const eventPosition = eventEntity?.position?.getValue?.(Cesium.JulianDate.now());
        if (!payload || !eventPosition) return;
        const isFocusedCanonical = focusedCanonicalEventId && payload.canonical_event_id === focusedCanonicalEventId;
        const connector = state.viewer.entities.add({
          id: `passive-focus-link-${payload.canonical_event_id}`,
          polyline: {
            positions: [focusedSitePosition, eventPosition],
            width: isFocusedCanonical ? 3 : 2,
            material: isFocusedCanonical
              ? Cesium.Color.CYAN.withAlpha(0.95)
              : passiveStatusColor(payload.status).withAlpha(0.55),
            clampToGround: false,
          },
        });
        state.passiveFocusOverlayEntities.push(connector);
      });
    }

    function renderPassiveMapLegend(region, sites, canonicalEvents, visibleRegionCount) {
      const target = $("passive-map-legend");
      if (!target) return;
      const filter = state.passiveSemanticFilter || "all";
      const pressureFilter = state.passivePressureFilter || "all";
      const attentionSets = passiveAttentionSets();
      const attentionMode = state.passiveAttentionMapMode && attentionSets.hasMapTargets;
      const summary = updatePassiveMapSummary(region, sites, canonicalEvents, visibleRegionCount);
      const focusedEvent = summary.selectedEventId
        ? canonicalEvents.find((event) => event.canonical_event_id === summary.selectedEventId)
        : null;
      target.innerHTML = `
        <h3>Passive Map Focus</h3>
        <div class="legend-grid">
          <div class="legend-row">
            <span class="pill">${escapeHtml(filter === "all" ? "all statuses" : filter)}</span>
            <span>${escapeHtml(region?.name ?? "No region selected")}</span>
          </div>
          <div class="legend-row">
            <span>pressure filter ${escapeHtml(pressureFilter)}</span>
            <span>${summary.visibleRegionCount ?? 0} visible regions</span>
          </div>
          <div class="legend-row">
            <span class="pill">${attentionMode ? "attention map on" : "attention map off"}</span>
            <span>${attentionSets.regionIds.size} attention regions</span>
            <span>${attentionSets.siteIds.size} attention sites</span>
            <span>${attentionSets.canonicalIds.size} attention events</span>
          </div>
          <div class="legend-row">
            <span>${summary.highlightedSiteCount ?? 0} highlighted sites</span>
            <span>${summary.visibleCanonicalEventCount ?? 0} canonical events</span>
            <span>${summary.visibleSiteCount ?? 0} sites in region</span>
          </div>
          <div class="legend-row">
            <span class="legend-dot" style="background:#ff4d4f"></span><span>pressure high</span>
            <span class="legend-dot" style="background:#ff9f43"></span><span>pressure medium</span>
            <span class="legend-dot" style="background:#8bd450"></span><span>pressure low</span>
          </div>
          <div class="legend-row">
            <span class="legend-dot" style="background:#ff4d4f"></span><span>escalating</span>
            <span class="legend-dot" style="background:#ff9f43"></span><span>new</span>
            <span class="legend-dot" style="background:#f4d35e"></span><span>recurring</span>
            <span class="legend-dot" style="background:#3dd6d0"></span><span>cooling</span>
          </div>
          <div class="legend-row">
            <span>${escapeHtml(region?.dominant_status ?? "no dominant status")}</span>
            <span>${region?.critical_events ?? 0} critical</span>
            <span>${region?.escalating_event_count ?? 0} escalating</span>
            <span>${region?.recent_failed_run_count ?? 0} failed runs</span>
          </div>
          <div class="legend-row">
            <span>ops trend ${escapeHtml(summary.operationalTrend ?? "n/a")}</span>
            <span>now ${escapeHtml(summary.operationalPriority ?? "n/a")}</span>
            <span>${state.selectedPassiveOperationalTimeline?.current_snapshot?.stale_lease_count ?? 0} stale leases</span>
            <span>${state.selectedPassiveOperationalTimeline?.current_snapshot?.stale_worker_count ?? 0} stale workers</span>
          </div>
          <div class="legend-row">
            <span>${escapeHtml(summary.selectedSiteName ?? summary.selectedSiteId ?? "no focused site")}</span>
            <span>${summary.selectedEventType ? `focused event ${escapeHtml(summary.selectedEventType)}` : "no focused phenomenon"}</span>
          </div>
          ${focusedEvent ? `<div class="legend-row"><span>chain ${escapeHtml(summary.selectedSiteName ?? focusedEvent.site_name ?? "site")} -> ${escapeHtml(focusedEvent.event_type)} -> ${focusedEvent.bundle_hashes?.length ?? 0} evidence / ${focusedEvent.manifest_hashes?.length ?? 0} replay</span></div>` : ""}
          ${state.passiveOperationalVisibility ? `<div class="legend-row">
            <span class="pill ${visibilityStateClass(state.passiveOperationalVisibility.overall_state)}">${escapeHtml(state.passiveOperationalVisibility.overall_state)}</span>
            <span>${state.passiveOperationalVisibility.total_regions} regions</span>
            <span>${state.passiveOperationalVisibility.active_workers} active workers</span>
            ${(state.passiveOperationalVisibility.stale_leases ?? 0) > 0 ? `<span class="pill danger">${state.passiveOperationalVisibility.stale_leases} stale leases</span>` : ""}
          </div>` : ""}
        </div>`;
    }

    async function handlePassiveSelection(entity) {
      const layer = cesiumPropertyValue(entity?.properties?.layer);
      const payload = cesiumPropertyValue(entity?.properties?.payload);
      if (layer === "neo-briefing" && payload?.neo_reference_id) {
        selectNeoBriefingItem(payload.neo_reference_id, "Selected NEO");
        return;
      }
      if (!layer || !payload) {
        renderEmpty($("passive-focus"), "Select a passive region, site, or canonical event on the map.");
        return;
      }

        try {
          if (layer === "passive-region") {
            state.selectedPassiveRegionId = payload.region_id;
            applyPassiveSiteFocus(null, null);
            const [overview, timeline, diagnostics, remediation, operationalTimeline] = await Promise.all([
              api(`/v1/passive/regions/${encodeURIComponent(payload.region_id)}/overview`),
              api(`/v1/passive/regions/${encodeURIComponent(payload.region_id)}/semantic-timeline?limit=6&window_hours=720`),
              api(`/v1/passive/worker/diagnostics?limit=12&region_id=${encodeURIComponent(payload.region_id)}`),
              api(`/v1/passive/regions/${encodeURIComponent(payload.region_id)}/remediation`),
              api(`/v1/passive/regions/${encodeURIComponent(payload.region_id)}/operational-timeline?window_hours=72&bucket_hours=6&include_empty_buckets=false`),
            ]);
            state.selectedPassiveOperationalTimeline = operationalTimeline;
            syncSemanticTimelinePanel(timeline, payload.region_id);
            const regionMetric = (diagnostics.region_metrics || []).find((metric) => metric.region_id === payload.region_id) || diagnostics.region_metrics?.[0] || null;
            const regionPressure = passiveRegionPressure(payload);
            $("passive-focus").innerHTML = `
              ${renderPassiveSelectedAttentionContext()}
              <div class="card">
                <h4>${escapeHtml(overview.region.name)}</h4>
              <div class="meta">
                <span class="${pillClass(overview.discovery_due ? "Warning" : "ok")}">${overview.discovery_due ? "due" : "fresh"}</span>
                <span>${overview.seed_count} seeds</span>
                <span>${overview.recent_event_count} recent events</span>
                <span>${overview.critical_event_count} critical</span>
              </div>
              <div class="meta" style="margin-top:8px">
                <span>${escapeHtml(payload.dominant_status ?? "no_canonical_state")}</span>
                <span>${payload.escalating_event_count ?? 0} escalating</span>
                <span>${payload.new_event_count ?? 0} new</span>
                <span>${payload.cooling_event_count ?? 0} cooling</span>
                </div>
                <div class="meta" style="margin-top:8px">${escapeHtml(overview.narrative)}</div>
                <div class="meta" style="margin-top:8px">
                  <span class="${pillClass(regionPressure === "critical" || regionPressure === "high" ? "failed" : regionPressure === "medium" ? "Warning" : "ok")}">pressure ${escapeHtml(regionPressure)}</span>
                  <span>${diagnostics.stale_region_lease_count ?? 0} stale leases</span>
                  <span>${diagnostics.stale_worker_heartbeat_count ?? 0} stale workers</span>
                  <span>${regionMetric?.recent_failed_run_count ?? 0} failed runs</span>
                  <span>${regionMetric?.source_error_count ?? 0} source errors</span>
                </div>
                <div class="meta" style="margin-top:8px">${escapeHtml(payload.operational_summary ?? "No operational summary yet.")}</div>
                <div class="row" style="margin-top:12px">
                  <button class="secondary passive-open-worker-diagnostics" data-region-id="${escapeHtml(payload.region_id)}">Region Diagnostics</button>
                  ${(diagnostics.active_region_leases || []).length ? `<button class="secondary passive-open-lease" data-region-id="${escapeHtml(payload.region_id)}">View Active Lease</button>` : ""}
                  ${(diagnostics.stale_worker_heartbeat_count ?? 0) > 0 ? `<button class="secondary passive-open-stale-heartbeats">View Stale Heartbeats</button>` : ""}
                  ${(diagnostics.stale_worker_heartbeat_count ?? 0) > 0 ? `<button class="secondary passive-prune-stale-heartbeats">Prune Stale Heartbeats</button>` : ""}
                </div>
              </div>
              ${renderRegionProvenance(overview.provenance)}
              <div class="card">
                <h4>Remediation</h4>
                <div class="meta">
                  <span>${escapeHtml(remediation.posture ?? regionPressure)}</span>
                  <span>${(remediation.actions || []).length} actions</span>
                  <span>${escapeHtml(payload.operational_pressure_priority ?? regionPressure)}</span>
                </div>
                <div class="meta" style="margin-top:8px">${escapeHtml(remediation.summary)}</div>
                <div class="list" style="margin-top:10px">
                  ${(remediation.actions || []).slice(0, 4).map((action) => `
                    <div class="card">
                      <h4>${escapeHtml(action.title)}</h4>
                      <div class="meta">
                        <span>${escapeHtml(action.priority)}</span>
                        <span>${escapeHtml(action.source)}</span>
                      </div>
                      <div class="meta" style="margin-top:8px">${escapeHtml(action.reason)}</div>
                      ${action.related_sources?.length ? `<div class="row" style="margin-top:8px">
                        ${action.related_sources.map((source) => `<button class="secondary passive-open-source-samples" data-source-kind="${escapeHtml(source)}" data-region-id="${escapeHtml(payload.region_id)}">Samples ${escapeHtml(source)}</button>`).join("")}
                      </div>` : ""}
                      ${action.suggested_read_paths?.length ? `<div class="row" style="margin-top:8px">${remediationReadButtons(action, payload.region_id)}</div>` : ""}
                    </div>`).join("") || `<div class="empty">No remediation guidance available.</div>`}
                </div>
              </div>
              <div class="card">
                <h4>Top Sites</h4>
              <div class="list" style="margin-top:10px">
                ${(overview.top_sites || []).slice(0, 3).map((site) => `
                  <div class="meta">
                    <span>${escapeHtml(site.site_name)}</span>
                    <span>risk ${site.latest_peak_risk != null ? `${(site.latest_peak_risk * 100).toFixed(0)}%` : "n/a"}</span>
                    <span>${escapeHtml(site.risk_direction)}</span>
                    </div>`).join("") || `<div class="empty">No regional sites summarized yet.</div>`}
                </div>
              </div>
              ${renderOperationalTimeline(operationalTimeline)}
              ${renderRegionVisibilityCard(payload.region_id, true)}
              ${renderSemanticTimeline(timeline)}`;
            return;
          }

          if (layer === "passive-site") {
            if (!payload.site_id) {
              renderEmpty($("passive-focus"), "This seed has not been linked to a passive site profile yet.");
              return;
            }
            applyPassiveSiteFocus(payload.site_id, null);
            const [overview, narrative, timeline] = await Promise.all([
              api(`/v1/passive/sites/${encodeURIComponent(payload.site_id)}/overview?limit=10`),
              api(`/v1/passive/sites/${encodeURIComponent(payload.site_id)}/narrative?days=30`),
              api(`/v1/passive/sites/${encodeURIComponent(payload.site_id)}/semantic-timeline?limit=6&window_hours=720`),
            ]);
            $("passive-focus").innerHTML = `
              ${renderPassiveSelectedAttentionContext()}
              <div class="card">
                <h4>${escapeHtml(overview.site.site.name)}</h4>
              <div class="meta">
                <span>${escapeHtml(payload.site_type)}</span>
                <span>risk ${(payload.risk_score * 100).toFixed(0)}%</span>
                <span>confidence ${(payload.confidence * 100).toFixed(0)}%</span>
              </div>
              <div class="meta" style="margin-top:8px">
                <span>${escapeHtml(payload.risk_trend ?? "flat")}</span>
                <span>${escapeHtml(payload.top_canonical_status ?? "no_canonical_state")}</span>
              </div>
              <div class="meta" style="margin-top:8px">${escapeHtml(narrative.narrative)}</div>
              ${payload.risk_delta_explanation ? `<div class="meta" style="margin-top:8px">${escapeHtml(payload.risk_delta_explanation)}</div>` : ""}
            </div>
            ${renderNarrativeProvenance(narrative.provenance)}
            <div class="card">
              <h4>Recent Site Pressure</h4>
                <div class="meta">
                  <span>${overview.recent_events.length} recent events</span>
                  <span>${overview.recurring_patterns.length} recurring patterns</span>
                  <span>peak ${overview.latest_risk ? `${(overview.latest_risk.peak_risk * 100).toFixed(0)}%` : "n/a"}</span>
                </div>
              </div>
              ${renderFocusedChainActions(payload)}
              <div class="card">
                <h4>Region Runs</h4>
                <div class="row" style="margin-top:8px">
                  <button class="secondary passive-open-region-runs" data-region-id="${escapeHtml(payload.region_id)}">View Region Runs</button>
                </div>
              </div>
              ${renderSemanticTimeline(timeline)}`;
            syncSemanticTimelinePanel(timeline, payload.region_id);
            return;
          }

          if (layer === "passive-canonical-event") {
            applyPassiveSiteFocus(payload.site_id, payload.canonical_event_id);
            const [overview, narrative, timeline] = await Promise.all([
              api(`/v1/passive/sites/${encodeURIComponent(payload.site_id)}/overview?limit=10`),
              api(`/v1/passive/sites/${encodeURIComponent(payload.site_id)}/narrative?days=30`),
              api(`/v1/passive/sites/${encodeURIComponent(payload.site_id)}/semantic-timeline?limit=6&window_hours=720`),
            ]);
            $("passive-focus").innerHTML = `
              ${renderPassiveSelectedAttentionContext()}
              <div class="card">
                <h4>${escapeHtml(payload.event_type)} @ ${escapeHtml(payload.site_name)}</h4>
              <div class="meta">
                <span class="${pillClass(payload.severity === "critical" || payload.severity === "high" ? "failed" : "ok")}">${escapeHtml(payload.severity)}</span>
                  <span>${payload.support_count} signals</span>
                  <span>${escapeHtml(payload.status)}</span>
                  <span>risk ${(payload.risk_score * 100).toFixed(0)}%</span>
                  <span>confidence ${(payload.confidence * 100).toFixed(0)}%</span>
                </div>
                <div class="meta" style="margin-top:8px">${escapeHtml(payload.summary)}</div>
                <div class="meta" style="margin-top:8px">${escapeHtml(payload.status_summary)}</div>
                <div class="meta" style="margin-top:8px">${escapeHtml(payload.risk_delta.explanation)}</div>
                <div class="meta" style="margin-top:8px">first ${formatEpoch(payload.first_observed_at_unix_seconds)} | last ${formatEpoch(payload.last_observed_at_unix_seconds)}</div>
                <div class="row" style="margin-top:12px">${passiveActionLinks(payload.bundle_hashes, payload.manifest_hashes)}</div>
              </div>
            ${renderFocusedChainActions(payload)}
            <div class="card">
              <h4>Region Runs</h4>
              <div class="row" style="margin-top:8px">
                <button class="secondary passive-open-region-runs" data-region-id="${escapeHtml(payload.region_id)}">View Region Runs</button>
              </div>
            </div>
            <div class="card">
              <h4>Site Narrative</h4>
              <div class="meta">${escapeHtml(narrative.narrative)}</div>
                <div class="meta" style="margin-top:10px">
                  <span>${overview.recent_events.length} recent events</span>
                  <span>${overview.recurring_patterns.length} patterns</span>
                  <span>${overview.risk_history_narrative.risk_direction}</span>
                </div>
              </div>
              ${renderNarrativeProvenance(narrative.provenance)}
              ${renderSemanticTimeline(timeline)}`;
            syncSemanticTimelinePanel(timeline, payload.region_id);
          }
      } catch (error) {
        renderEmpty($("passive-focus"), error.message);
      }
    }

    async function refreshPassiveMapLayers() {
      if (!state.globeReady) return;
      clearPassiveMapEntities();
      const regionData = await api("/v1/passive/map/regions");
      const regions = regionData.regions || [];
      const attentionSets = passiveAttentionSets();
      const attentionMapActive = state.passiveAttentionMapMode && attentionSets.hasMapTargets;
      const visibleRegions = regions
        .filter(matchesPassivePressureFilter)
        .filter((region) => {
          if (!attentionMapActive) return true;
          if (attentionSets.siteIds.size || attentionSets.canonicalIds.size || !attentionSets.regionIds.size) return true;
          return attentionSets.regionIds.has(region.region_id);
        });

      if (!visibleRegions.length) {
        state.selectedPassiveRegionId = null;
        state.selectedPassiveSiteId = null;
        state.selectedPassiveCanonicalEventId = null;
        state.passiveMapSummary = {
          visibleRegionCount: 0,
          visibleSiteCount: 0,
          visibleCanonicalEventCount: 0,
          highlightedSiteCount: 0,
          selectedRegionId: null,
          selectedRegionName: null,
          selectedSiteId: null,
          selectedSiteName: null,
          selectedEventId: null,
          selectedEventType: null,
          selectedEventStatus: null,
          operationalTrend: null,
          operationalPriority: null,
        };
        renderPassiveMapLegend(null, [], [], 0);
        renderEmpty($("passive-focus"), "No passive regions match the selected pressure filter.");
        state.selectedPassiveOperationalTimeline = null;
        if (state.passiveDashboardSummary && state.passiveMaintenanceSummary) {
          renderPassiveDashboard(
            state.passiveDashboardSummary,
            state.passiveMaintenanceSummary,
            state.passiveCommandCenterSummary,
          );
        }
        return;
      }

      visibleRegions.forEach((region) => {
        const pressure = passiveRegionPressure(region);
        const outlineColor = pressure === "critical"
          ? Cesium.Color.RED
          : pressure === "high"
            ? Cesium.Color.ORANGE
            : pressure === "medium"
              ? Cesium.Color.YELLOW
              : Cesium.Color.LIME;
        const outlineWidth = pressure === "critical" ? 4 : pressure === "high" ? 3 : pressure === "medium" ? 2 : 1;
        const regionEntity = state.viewer.entities.add({
          id: `passive-region-${region.region_id}`,
          name: region.name,
          position: Cesium.Cartesian3.fromDegrees(
            (region.bbox.west + region.bbox.east) / 2,
            (region.bbox.south + region.bbox.north) / 2,
          ),
          rectangle: {
            coordinates: Cesium.Rectangle.fromDegrees(
              region.bbox.west,
              region.bbox.south,
              region.bbox.east,
              region.bbox.north,
            ),
            material: passiveRegionColor(region.status, pressure),
            outline: true,
            outlineColor,
            outlineWidth,
          },
          label: {
            text: `${region.name} â€¢ ${regionOperationalState(region.region_id) ?? pressure} â€¢ ${region.status}`,
            font: "12px sans-serif",
            fillColor: Cesium.Color.WHITE,
            showBackground: true,
            backgroundColor: Cesium.Color.fromAlpha(Cesium.Color.BLACK, 0.55),
          },
          properties: { layer: "passive-region", payload: region },
          description: `${region.narrative_summary} | ${region.operational_summary ?? "no operational summary"} | dominant ${region.dominant_status ?? "none"} | critical ${region.critical_events ?? 0} | escalating ${region.escalating_event_count ?? 0}`,
        });
        state.passiveGlobeEntities.push(regionEntity);
      });

        const currentRegionStillVisible = visibleRegions.find((region) => region.region_id === state.selectedPassiveRegionId)?.region_id;
        const preferredAttentionRegion = attentionMapActive
          ? visibleRegions.find((region) => attentionSets.regionIds.has(region.region_id))?.region_id
          : null;
        const selectedRegion = preferredAttentionRegion || currentRegionStillVisible || visibleRegions[0]?.region_id;
        if (!selectedRegion) return;
        state.selectedPassiveRegionId = selectedRegion;
        const selectedRegionSummary = visibleRegions.find((region) => region.region_id === selectedRegion) || null;

      const [siteData, eventData] = await Promise.all([
        api(`/v1/passive/map/sites?region_id=${encodeURIComponent(selectedRegion)}`),
        api(`/v1/passive/map/canonical-events?region_id=${encodeURIComponent(selectedRegion)}&limit=50`),
      ]);

      const filteredCanonicalEvents = (eventData.events || [])
        .filter((event) => state.passiveSemanticFilter === "all" || event.status === state.passiveSemanticFilter)
        .filter((event) => attentionModeAppliesToEvent(event, attentionSets));
      const visibleSites = (siteData.sites || []).filter((site) =>
        attentionModeAppliesToSite(site, attentionSets)
      );
      const highlightedSiteIds = new Set(
        filteredCanonicalEvents
          .map((event) => event.site_id)
          .filter(Boolean)
      );

      visibleSites.forEach((site) => {
        const siteColor = passiveSiteColor(site.site_type);
        const siteStatusColor = site.top_canonical_status ? passiveStatusColor(site.top_canonical_status) : Cesium.Color.WHITE;
        const attentionTarget = attentionMapActive && site.site_id && attentionSets.siteIds.has(site.site_id);
        const highlighted = attentionMapActive
          ? attentionTarget || (site.site_id && highlightedSiteIds.has(site.site_id))
          : state.passiveSemanticFilter === "all"
            ? site.elevated || site.risk_score >= 0.6
            : (site.site_id && highlightedSiteIds.has(site.site_id));
        const siteEntity = state.viewer.entities.add({
          id: `passive-site-${site.seed_key}`,
          name: site.name,
          position: Cesium.Cartesian3.fromDegrees(site.coordinates.lon, site.coordinates.lat),
          point: {
            pixelSize: (site.risk_score >= 0.75 ? 16 : site.risk_score >= 0.45 ? 13 : 10) + (highlighted ? 2 : 0),
            color: highlighted ? siteColor : Cesium.Color.fromAlpha(siteColor, 0.22),
            outlineColor: attentionTarget
              ? Cesium.Color.RED
              : highlighted
                ? siteStatusColor
                : Cesium.Color.fromAlpha(Cesium.Color.WHITE, 0.3),
            outlineWidth: attentionTarget ? 4 : highlighted ? 3 : 1,
          },
          label: {
            text: site.name,
            font: "11px sans-serif",
            pixelOffset: new Cesium.Cartesian2(0, -18),
            fillColor: highlighted ? Cesium.Color.WHITE : Cesium.Color.fromAlpha(Cesium.Color.WHITE, 0.45),
            showBackground: highlighted,
            backgroundColor: Cesium.Color.fromAlpha(Cesium.Color.BLACK, 0.55),
          },
          properties: { layer: "passive-site", payload: site },
          description: `${site.site_type} | risk ${site.risk_score.toFixed(2)} | ${site.seed_status} | ${site.top_canonical_status ?? "no_canonical_state"} | ${site.risk_trend} | ${attentionTarget ? "attention_target" : highlighted ? "highlighted" : "background"}`,
        });
        state.passiveGlobeEntities.push(siteEntity);
        if (site.site_id) {
          state.passiveSiteEntityIndex[site.site_id] = siteEntity;
        }
      });

        filteredCanonicalEvents.forEach((event) => {
          const attentionTarget = attentionMapActive && attentionSets.canonicalIds.has(event.canonical_event_id);
          const eventEntity = state.viewer.entities.add({
            id: `passive-canonical-event-${event.canonical_event_id}`,
            name: `${event.event_type} @ ${event.site_name}`,
            position: Cesium.Cartesian3.fromDegrees(event.coordinates.lon, event.coordinates.lat),
            point: {
              pixelSize: passiveCanonicalPixelSize(event),
              color: passiveStatusColor(event.status),
              outlineColor: attentionTarget ? Cesium.Color.WHITE : Cesium.Color.BLACK,
              outlineWidth: attentionTarget ? 4 : 2,
            },
            label: {
              text: `${event.event_type} ${event.status} (${event.support_count})`,
              font: "10px sans-serif",
              pixelOffset: new Cesium.Cartesian2(0, 18),
              fillColor: Cesium.Color.WHITE,
              showBackground: true,
              backgroundColor: Cesium.Color.fromAlpha(Cesium.Color.BLACK, 0.55),
          },
          properties: { layer: "passive-canonical-event", payload: event },
          description: `${attentionTarget ? "attention_target | " : ""}${event.summary}`,
          });
          state.passiveGlobeEntities.push(eventEntity);
          state.passiveEntityIndex[event.canonical_event_id] = eventEntity;
          if (!state.passiveSiteEventIndex[event.site_id]) {
            state.passiveSiteEventIndex[event.site_id] = [];
          }
          state.passiveSiteEventIndex[event.site_id].push(eventEntity);
        });

      renderPassiveMapLegend(selectedRegionSummary, visibleSites, filteredCanonicalEvents, visibleRegions.length);
      applyPassiveSiteFocus(state.selectedPassiveSiteId, state.selectedPassiveCanonicalEventId);
      if (state.passiveDashboardSummary && state.passiveMaintenanceSummary) {
        renderPassiveDashboard(
          state.passiveDashboardSummary,
          state.passiveMaintenanceSummary,
          state.passiveCommandCenterSummary,
        );
      }

      if (state.passiveGlobeEntities.length) {
        state.viewer.zoomTo(state.passiveGlobeEntities);
      }

      renderEmpty($("passive-focus"), "Select a passive region, site, or canonical event on the map.");
    }

    async function refreshPassiveDashboardSummary() {
      try {
        const semanticFilter = state.passiveSemanticFilter && state.passiveSemanticFilter !== "all"
          ? `&semantic_status=${encodeURIComponent(state.passiveSemanticFilter)}`
          : "";
        const pressureFilter = state.passivePressureFilter && state.passivePressureFilter !== "all"
          ? `&min_pressure_priority=${encodeURIComponent(state.passivePressureFilter)}`
          : "";
        const attentionKindFilter = state.passiveAttentionKindFilter && state.passiveAttentionKindFilter !== "all"
          ? `&attention_kind=${encodeURIComponent(state.passiveAttentionKindFilter)}`
          : "";
        const attentionPriorityFilter = state.passiveAttentionPriorityFilter && state.passiveAttentionPriorityFilter !== "all"
          ? `&min_attention_priority=${encodeURIComponent(state.passiveAttentionPriorityFilter)}`
          : "";
        const commandCenter = await api(`/v1/passive/command-center/summary?limit=5&semantic_window_hours=336${semanticFilter}${pressureFilter}${attentionKindFilter}${attentionPriorityFilter}`);
        const data = commandCenter.dashboard || {};
        const maintenance = commandCenter.maintenance || {};
        state.passiveDashboardSummary = data;
        state.passiveMaintenanceSummary = maintenance;
        state.passiveCommandCenterSummary = commandCenter;
        renderPassiveDashboard(data, maintenance, commandCenter);
        renderPassiveCommandCenterPanel(commandCenter);
        return data;
      } catch (error) {
        renderEmpty($("passive-dashboard"), error.message);
        return null;
      }
    }

    function renderPassiveCanonicalEvents(events) {
      const target = $("canonical-events");
      if (!target) return;
      if (!events || !events.length) {
        renderEmpty(target, "No canonical events.");
        return;
      }
      target.innerHTML = events.slice(0, 10).map((event) => `
        <div class="card">
          <div class="meta">
            ${renderRiskDeltaPill(event.risk_delta?.classification)}
            <span class="${pillClass(event.severity === "critical" || event.severity === "high" ? "failed" : "ok")}">${escapeHtml(event.severity)}</span>
            <span>${escapeHtml(event.event_type)}</span>
            <span>${escapeHtml(event.site_name)}</span>
          </div>
          <div class="meta" style="margin-top:6px">
            <span>${escapeHtml(event.status)}</span>
            <span>${escapeHtml(event.temporal_phase)}</span>
            <span>risk ${(event.risk_score * 100).toFixed(0)}%</span>
            <span>${event.support_count} signals</span>
          </div>
          <div class="meta" style="margin-top:6px">${escapeHtml(event.status_summary)}</div>
          ${event.risk_delta?.explanation ? `<div class="meta" style="margin-top:6px">${escapeHtml(event.risk_delta.explanation)}</div>` : ""}
          <div class="row" style="margin-top:8px">
            <button class="secondary passive-refocus-canonical" data-canonical-id="${escapeHtml(event.canonical_event_id)}" data-site-id="${escapeHtml(event.site_id)}">Focus Event</button>
            <button class="secondary passive-open-region-runs" data-region-id="${escapeHtml(event.region_id)}">Region Runs</button>
          </div>
        </div>`).join("");
    }

    async function refreshPassiveCanonicalEvents() {
      const regionQuery = state.selectedPassiveRegionId
        ? `&region_id=${encodeURIComponent(state.selectedPassiveRegionId)}`
        : "";
      try {
        const data = await api(`/v1/passive/canonical-events?limit=10${regionQuery}`);
        renderPassiveCanonicalEvents(data.events || []);
        return data;
      } catch (error) {
        renderEmpty($("canonical-events"), error.message);
        return null;
      }
    }

    async function refreshPassiveOperationalVisibility() {
      try {
        const regionQuery = state.selectedPassiveRegionId
          ? `&region_id=${encodeURIComponent(state.selectedPassiveRegionId)}`
          : "";
        const data = await api(`/v1/passive/operational-visibility?limit=25${regionQuery}`);
        state.passiveOperationalVisibility = data;
        $('operational-visibility').innerHTML = renderOperationalVisibility(data);
        if (state.globeReady) { refreshPassiveMapLayers(); }
        return data;
      } catch (error) {
        renderEmpty($('operational-visibility'), error.message);
        return null;
      }
    }

    function syncSemanticTimelinePanel(timeline, label) {
      const target = $("semantic-timeline");
      const lbl = $("semantic-timeline-label");
      if (!target) return;
      const prevScroll = target.scrollTop;
      state.semanticTimeline = timeline;
      if (lbl) lbl.textContent = label || "";
      target.innerHTML = renderSemanticTimeline(timeline);
      target.scrollTop = prevScroll;
      target.classList.remove("stl-updated");
      void target.offsetWidth; // force reflow to restart animation
      target.classList.add("stl-updated");
    }

    async function refreshSemanticTimeline() {
      const regionId = state.selectedPassiveRegionId;
      const target = $("semantic-timeline");
      const label = $("semantic-timeline-label");
      if (!regionId) {
        renderEmpty(target, "Select a passive region to load its semantic timeline.");
        if (label) label.textContent = "";
        return null;
      }
      try {
        const prevScroll = target.scrollTop;
        const data = await api(`/v1/passive/regions/${encodeURIComponent(regionId)}/semantic-timeline?limit=20`);
        state.semanticTimeline = data;
        if (label) label.textContent = regionId;
        target.innerHTML = renderSemanticTimeline(data);
        target.scrollTop = prevScroll;
        target.classList.remove("stl-updated");
        void target.offsetWidth;
        target.classList.add("stl-updated");
        return data;
      } catch (error) {
        renderEmpty(target, error.message);
        return null;
      }
    }

    async function refreshIngestStatus() {
        const source = $("source-name").value.trim() || "celestrak-active";
        try {
          const data = await api(`/v1/ingest/status?source=${encodeURIComponent(source)}`);
        $("metric-freshness").textContent = formatSeconds(data.freshness_seconds);
        renderCards($("ingest-status"), [data], (item) => `
          <div class="card">
            <h4>${item.source}</h4>
            <div class="meta">
              <span class="${pillClass('ok')}">freshness ${formatSeconds(item.freshness_seconds)}</span>
              <span class="pill">${item.object_count} objects</span>
            </div>
            <div class="meta" style="margin-top:8px">
              <span>request <span class="mono">${item.latest_request_id}</span></span>
              <span>${formatEpoch(item.latest_timestamp_unix_seconds)}</span>
            </div>
          </div>`);
      } catch (error) {
        $("metric-freshness").textContent = "n/a";
        renderEmpty($("ingest-status"), error.message);
        }
      }

    async function refreshApodBriefing() {
        try {
          const data = await api("/v1/briefing/apod");
          $("apod-briefing").innerHTML = `
            <div class="list">
              <div class="card">
                <h4>${data.title}</h4>
                <div class="meta">
                  <span class="pill">${data.media_type}</span>
                  <span>${data.date}</span>
                </div>
                ${data.url ? `<div style="margin-top:12px"><img src="${data.url}" alt="${data.title}" style="width:100%;height:220px;object-fit:cover;border-radius:8px;border:1px solid rgba(151,181,214,0.18)" /></div>` : ""}
                <div class="meta" style="margin-top:12px;line-height:1.6">${data.explanation}</div>
              </div>
            </div>`;
        } catch (error) {
          renderEmpty($("apod-briefing"), error.message);
        }
      }

    async function refreshNeoWsBriefing() {
      try {
        const data = await api("/v1/briefing/neows");
        state.neoBriefing = data;
        renderNeoWsBriefing(data);
      } catch (error) {
        state.neoBriefing = null;
        state.selectedNeoReferenceId = null;
        renderEmpty($("neows-briefing"), error.message);
      }
    }

    async function refreshEventsTimeline() {
      const objectId = $("object-id").value.trim();
      const horizon = Number($("timeline-horizon").value || 72);
      const eventType = $("event-type-filter").value.trim();
      const eventTypeParam = eventType ? `&event_type=${encodeURIComponent(eventType)}` : "";
      const data = await api(`/v1/events/timeline?horizon=${horizon}&limit_per_bucket=6&object_id=${encodeURIComponent(objectId)}${eventTypeParam}`);
      $("metric-events").textContent = String(data.total_events);
      renderCards($("events-timeline"), data.buckets, (bucket) => `
        <div class="card">
          <h4>${bucket.label}</h4>
          <div class="meta"><span class="pill">${bucket.events.length} events</span></div>
          <div class="list" style="margin-top:10px">
            ${bucket.events.map((event) => `
              <div class="card">
                <h4>${event.event_type}</h4>
                <div class="meta">
                  <span class="${pillClass(event.prediction?.event === 'UnstableTrack' ? 'failed' : 'ok')}">risk ${(event.risk * 100).toFixed(0)}%</span>
                  <span>${formatEpoch(event.target_epoch_unix_seconds)}</span>
                </div>
                <div class="meta" style="margin-top:8px">${event.summary}</div>
                <div class="row" style="margin-top:10px">
                  <button class="secondary event-dispatch" data-event-id="${event.event_id}">Dispatch Event</button>
                </div>
              </div>`).join("")}
          </div>
        </div>` );
      return data;
    }

    async function refreshEventQueue() {
      const objectId = $("object-id").value.trim();
      const eventType = $("event-type-filter").value.trim();
      const eventTypeParam = eventType ? `&event_type=${encodeURIComponent(eventType)}` : "";
      const data = await api(`/v1/events?limit=8&object_id=${encodeURIComponent(objectId)}&future_only=true${eventTypeParam}`);
      renderCards($("event-queue"), data, (eventItem) => `
        <div class="card">
          <h4>${eventItem.event_type}</h4>
          <div class="meta">
            <span class="${pillClass(eventItem.prediction?.event === 'UnstableTrack' ? 'failed' : 'ok')}">risk ${(eventItem.risk * 100).toFixed(0)}%</span>
            <span>${formatEpoch(eventItem.target_epoch_unix_seconds)}</span>
            <span class="mono">${eventItem.object_id}</span>
          </div>
          <div class="meta" style="margin-top:8px">${eventItem.summary}</div>
          <div class="row" style="margin-top:10px">
            <button class="secondary event-dispatch" data-event-id="${eventItem.event_id}">Dispatch Event</button>
          </div>
        </div>`);
      return data;
    }

    async function refreshObjectTimeline() {
      const objectId = $("object-id").value.trim();
      const horizon = Number($("timeline-horizon").value || 72);
      const data = await api(`/v1/objects/${encodeURIComponent(objectId)}/timeline?horizon=${horizon}`);
      renderCards($("object-timeline"), data.checkpoints, (item) => `
        <div class="card">
          <h4>t+${item.offset_hours}h</h4>
          <div class="meta">
            <span class="pill">${item.propagation_model}</span>
            <span>${formatEpoch(item.target_epoch_unix_seconds)}</span>
          </div>
          <div class="meta" style="margin-top:8px">${item.summary}</div>
          </div>`);
      return data;
    }

    async function refreshCloseApproaches() {
      const objectId = $("object-id").value.trim();
      const horizon = Number($("timeline-horizon").value || 72);
      const data = await api(`/v1/objects/${encodeURIComponent(objectId)}/close-approaches?horizon=${horizon}&threshold_km=250&limit=6`);
      state.closeApproaches = data;
      renderCards($("close-approaches"), data.close_approaches, (item) => `
        <div class="card">
          <h4>${item.counterpart_object_name}</h4>
          <div class="meta">
            <span class="pill">${item.propagation_model}</span>
            <span>${formatEpoch(item.target_epoch_unix_seconds)}</span>
          </div>
          <div class="meta" style="margin-top:8px">
            <span>${item.miss_distance_km.toFixed(1)} km miss distance</span>
            <span>priority ${(item.priority_score * 100).toFixed(0)}%</span>
          </div>
          <div class="meta" style="margin-top:8px">${item.summary}</div>
        </div>`);
      if (!data.close_approaches.length) {
        renderEmpty($("close-approaches"), "No close approaches inside the selected horizon.");
      }
      return data;
    }

    async function refreshPredictionSnapshots() {
      const objectId = $("object-id").value.trim();
      const data = await api(`/v1/objects/${encodeURIComponent(objectId)}/predictions?limit=6`);
      renderCards($("prediction-snapshots"), data, (item) => `
        <div class="card">
          <h4>${item.endpoint}</h4>
          <div class="meta">
            <span class="pill">${item.propagation_model}</span>
            <span>${formatEpoch(item.generated_at_unix_seconds)}</span>
          </div>
          <div class="meta" style="margin-top:8px">
            <span>horizon ${item.horizon_hours}h</span>
            <span>${item.model_version}</span>
          </div>
          <div class="meta" style="margin-top:8px">
            <span>base ${formatEpoch(item.base_epoch_unix_seconds)}</span>
            <span class="mono">${item.evidence_bundle_hash ?? "no bundle"}</span>
          </div>
        </div>`);
    }

    async function refreshDeliveries() {
      const objectId = $("object-id").value.trim();
      const data = await api(`/v1/notifications?limit=6&object_id=${encodeURIComponent(objectId)}`);
      $("metric-deliveries").textContent = String(data.length);
        renderCards($("deliveries"), data, (item) => `
          <div class="card">
            <h4>${item.recipient}</h4>
            <div class="meta">
              <span class="${pillClass(item.status)}">${item.status}</span>
              <span>${item.channel}</span>
              <span>${item.status_code ?? "n/a"}</span>
              <span>attempt ${item.attempt_number}/${item.max_attempts}</span>
            </div>
            <div class="meta" style="margin-top:8px">${item.notification_reason}</div>
            <div class="meta" style="margin-top:8px"><span class="mono">${item.target}</span></div>
          </div>`);
    }

    async function analyzeAndReplay() {
      const objectId = $("object-id").value.trim();
      const payload = { object_id: objectId, timestamp_unix_seconds: null };
      const analysis = await api("/v1/analyze-object", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(payload),
      });
      state.lastBundleHash = analysis.evidence_bundle.bundle_hash;
      const replayManifest = await api(`/v1/evidence/${state.lastBundleHash}/replay`);
      state.lastManifestHash = replayManifest.manifest_hash;
      const replay = await api(`/v1/replay/${state.lastManifestHash}/execute`);
      renderReplayExecution(replay, state.lastManifestHash);
    }

    async function dispatchAlert() {
      const objectId = $("object-id").value.trim();
      await api("/v1/notifications/dispatch", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ object_id: objectId, timestamp_unix_seconds: null }),
      });
      await refreshDeliveries();
    }

    async function dispatchEvent(eventId) {
      await api("/v1/events/dispatch", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ event_id: eventId }),
      });
      await refreshDeliveries();
    }

    async function refreshAll() {
      $("refresh-status").textContent = "Refreshing...";
      try {
          const [_, __, ___, eventsTimeline, eventQueue, objectTimeline, closeApproaches] = await Promise.all([
              refreshIngestStatus(),
              refreshApodBriefing(),
              refreshNeoWsBriefing(),
              refreshEventsTimeline(),
              refreshEventQueue(),
              refreshObjectTimeline(),
            refreshCloseApproaches(),
            refreshPredictionSnapshots(),
            refreshDeliveries(),
            refreshPassiveDashboardSummary(),
            refreshPassiveOperationalVisibility(),
            refreshPassiveCanonicalEvents(),
            refreshSemanticTimeline(),
            analyzeAndReplay(),
        ]);
        updateGlobe(objectTimeline, eventsTimeline, state.neoBriefing, closeApproaches);
        await refreshPassiveMapLayers();
        $("refresh-status").textContent = `Updated ${new Date().toLocaleTimeString()}`;
      } catch (error) {
        $("refresh-status").textContent = error.message;
      }
    }

    $("refresh-all").addEventListener("click", refreshAll);
    $("dispatch-alert").addEventListener("click", async () => {
      $("refresh-status").textContent = "Dispatching...";
      try {
        await dispatchAlert();
        await refreshAll();
      } catch (error) {
        $("refresh-status").textContent = error.message;
      }
    });
    $("passive-dashboard").addEventListener("click", async (event) => {
      const pressureFilterButton = event.target.closest(".passive-pressure-filter");
      if (pressureFilterButton) {
        state.passivePressureFilter = pressureFilterButton.dataset.filter || "all";
        $("refresh-status").textContent = `Pressure filter: ${state.passivePressureFilter}`;
        await Promise.all([refreshPassiveDashboardSummary(), refreshPassiveMapLayers()]);
        return;
      }
      const attentionKindFilterButton = event.target.closest(".passive-attention-kind-filter");
      if (attentionKindFilterButton) {
        state.passiveAttentionKindFilter = attentionKindFilterButton.dataset.filter || "all";
        $("refresh-status").textContent = `Attention filter: ${state.passiveAttentionKindFilter}`;
        await refreshPassiveDashboardSummary();
        await refreshPassiveMapLayers();
        return;
      }
      const attentionPriorityFilterButton = event.target.closest(".passive-attention-priority-filter");
      if (attentionPriorityFilterButton) {
        state.passiveAttentionPriorityFilter = attentionPriorityFilterButton.dataset.filter || "all";
        $("refresh-status").textContent = `Attention priority: ${state.passiveAttentionPriorityFilter}`;
        await refreshPassiveDashboardSummary();
        await refreshPassiveMapLayers();
        return;
      }
      const attentionMapButton = event.target.closest(".passive-toggle-attention-map");
      if (attentionMapButton) {
        state.passiveAttentionMapMode = !state.passiveAttentionMapMode;
        $("refresh-status").textContent = `Attention map: ${state.passiveAttentionMapMode ? "on" : "off"}`;
        await Promise.all([refreshPassiveDashboardSummary(), refreshPassiveMapLayers()]);
        return;
      }
      const filterButton = event.target.closest(".passive-semantic-filter");
      if (filterButton) {
        state.passiveSemanticFilter = filterButton.dataset.filter || "all";
        $("refresh-status").textContent = `Semantic filter: ${state.passiveSemanticFilter}`;
        await Promise.all([refreshPassiveDashboardSummary(), refreshPassiveMapLayers()]);
        return;
      }
      const resetSemanticButton = event.target.closest(".passive-reset-semantic-filter");
      if (resetSemanticButton) {
        state.passiveSemanticFilter = "all";
        $("refresh-status").textContent = "Semantic filter: all";
        await Promise.all([refreshPassiveDashboardSummary(), refreshPassiveMapLayers()]);
        return;
      }
      const resetPressureButton = event.target.closest(".passive-reset-pressure-filter");
      if (resetPressureButton) {
        state.passivePressureFilter = "all";
        $("refresh-status").textContent = "Pressure filter: all";
        await Promise.all([refreshPassiveDashboardSummary(), refreshPassiveMapLayers()]);
        return;
      }
      const resetAttentionKindButton = event.target.closest(".passive-reset-attention-kind-filter");
      if (resetAttentionKindButton) {
        state.passiveAttentionKindFilter = "all";
        $("refresh-status").textContent = "Attention filter: all";
        await refreshPassiveDashboardSummary();
        await refreshPassiveMapLayers();
        return;
      }
      const resetAttentionPriorityButton = event.target.closest(".passive-reset-attention-priority-filter");
      if (resetAttentionPriorityButton) {
        state.passiveAttentionPriorityFilter = "all";
        $("refresh-status").textContent = "Attention priority: all";
        await refreshPassiveDashboardSummary();
        await refreshPassiveMapLayers();
        return;
      }
      const refocusCanonicalButton = event.target.closest(".passive-refocus-canonical");
      if (refocusCanonicalButton) {
        await handlePassiveRefocusCanonicalButton(refocusCanonicalButton, "Focused canonical event");
        return;
      }
      const refocusSiteButton = event.target.closest(".passive-refocus-site");
      if (refocusSiteButton) {
        await handlePassiveRefocusSiteButton(refocusSiteButton, "Focused site");
        return;
      }
      const attentionItemCard = event.target.closest(".passive-attention-item-card");
      if (attentionItemCard) {
        setPassiveSelectedAttentionFromDataset(attentionItemCard.dataset);
        await focusPassiveAttentionItem(
          attentionItemCard.dataset.attentionKind || attentionItemCard.dataset.kind,
          attentionItemCard.dataset.regionId,
          attentionItemCard.dataset.siteId,
          attentionItemCard.dataset.canonicalId,
        );
        return;
      }
      const openPathButton = event.target.closest(".passive-open-path");
      if (openPathButton) {
        handlePassiveOpenPathButton(openPathButton, "Opened read path", false);
        return;
      }
      const leaseButton = event.target.closest(".passive-open-lease");
      if (leaseButton) {
        setPassiveSelectedAttentionFromDataset(leaseButton.dataset);
        await openPassiveLease(leaseButton.dataset.regionId, "Focused active lease");
        return;
      }
      const heartbeatButton = event.target.closest(".passive-open-heartbeat");
      if (heartbeatButton) {
        await openPassiveHeartbeat(heartbeatButton.dataset.workerId, "Focused worker heartbeat");
        return;
      }
      const workerDiagnosticsButton = event.target.closest(".passive-open-worker-diagnostics");
      if (workerDiagnosticsButton) {
        setPassiveSelectedAttentionFromDataset(workerDiagnosticsButton.dataset);
        await openPassiveWorkerDiagnostics(
          "Opened worker diagnostics",
          workerDiagnosticsButton.dataset.regionId || null,
        );
        return;
      }
      const heartbeatListButton = event.target.closest(".passive-open-heartbeat-list");
      if (heartbeatListButton) {
        setPassiveSelectedAttentionFromDataset(heartbeatListButton.dataset);
        await openPassiveHeartbeatList(false, "Opened worker heartbeats");
        return;
      }
      const staleHeartbeatsButton = event.target.closest(".passive-open-stale-heartbeats");
      if (staleHeartbeatsButton) {
        setPassiveSelectedAttentionFromDataset(staleHeartbeatsButton.dataset);
        await openPassiveHeartbeatList(true, "Opened stale worker heartbeats");
        return;
      }
      const sourceSamplesButton = event.target.closest(".passive-open-source-samples");
      if (sourceSamplesButton) {
        setPassiveSelectedAttentionFromDataset(sourceSamplesButton.dataset);
        await openPassiveSourceSamples(sourceSamplesButton.dataset.sourceKind, "Opened source samples", sourceSamplesButton.dataset.regionId || null);
        return;
      }
      const previewSourcePruneButton = event.target.closest(".passive-preview-source-prune");
      if (previewSourcePruneButton) {
        setPassiveSelectedAttentionFromDataset(previewSourcePruneButton.dataset);
        previewSourcePruneButton.disabled = true;
        try {
          await pruneSourceHealthSamples(
            previewSourcePruneButton.dataset.sourceKind || null,
            previewSourcePruneButton.dataset.regionId || null,
            "Source health prune preview",
            true,
          );
        } catch (error) {
          $("refresh-status").textContent = error.message;
        } finally {
          previewSourcePruneButton.disabled = false;
        }
        return;
      }
      const pruneSourceHealthButton = event.target.closest(".passive-prune-source-health");
      if (pruneSourceHealthButton) {
        setPassiveSelectedAttentionFromDataset(pruneSourceHealthButton.dataset);
        pruneSourceHealthButton.disabled = true;
        try {
          await pruneSourceHealthSamples(
            pruneSourceHealthButton.dataset.sourceKind || null,
            pruneSourceHealthButton.dataset.regionId || null,
            "Pruned old source health samples",
          );
        } catch (error) {
          $("refresh-status").textContent = error.message;
        } finally {
          pruneSourceHealthButton.disabled = false;
        }
        return;
      }
      const openPathButton = event.target.closest(".passive-open-path");
      if (openPathButton) {
        handlePassiveOpenPathButton(openPathButton, "Opened read path");
        return;
      }
      const evidenceButton = event.target.closest(".passive-open-evidence");
      if (evidenceButton) {
        handlePassiveOpenEvidenceButton(evidenceButton, "Opened evidence");
        return;
      }
      const runReplayButton = event.target.closest(".passive-run-replay");
      if (runReplayButton) {
        await handlePassiveRunReplayButton(runReplayButton, "Passive replay");
        return;
      }
      const regionRunsButton = event.target.closest(".passive-open-region-runs");
      if (regionRunsButton) {
        setPassiveSelectedAttentionFromDataset(regionRunsButton.dataset);
        await openPassiveRegionRuns(regionRunsButton.dataset.regionId, "Opened region runs");
        return;
      }
      const regionOverviewButton = event.target.closest(".passive-open-region-overview");
      if (regionOverviewButton) {
        await handlePassiveOpenRegionOverviewButton(regionOverviewButton, "Opened region");
        return;
      }
      const stripEntry = event.target.closest(".passive-strip-entry");
      if (!stripEntry) return;
      const canonicalId = stripEntry.dataset.canonicalId;
      if (!canonicalId) return;
      focusPassiveCanonicalEvent(canonicalId, stripEntry.dataset.siteId, "Focused canonical event");
    });
    $("refresh-operational-visibility").addEventListener("click", async () => {
      $("refresh-status").textContent = "Refreshing operational visibility...";
      try {
        await refreshPassiveOperationalVisibility();
        $("refresh-status").textContent = `Operational visibility updated ${new Date().toLocaleTimeString()}`;
      } catch (error) {
        $("refresh-status").textContent = error.message;
      }
    });
    $("operational-visibility").addEventListener("click", async (event) => {
      const workerDiagnosticsButton = event.target.closest(".passive-open-worker-diagnostics");
      if (workerDiagnosticsButton) {
        await openPassiveWorkerDiagnostics(
          "Opened worker diagnostics",
          workerDiagnosticsButton.dataset.regionId || null,
        );
        return;
      }
      const regionRunsButton = event.target.closest(".passive-open-region-runs");
      if (regionRunsButton) {
        await openPassiveRegionRuns(regionRunsButton.dataset.regionId, "Opened region runs");
        return;
      }
      const heartbeatButton = event.target.closest(".passive-open-heartbeat");
      if (heartbeatButton) {
        await openPassiveHeartbeat(heartbeatButton.dataset.workerId, "Opened worker heartbeat");
        return;
      }
      const openPathButton = event.target.closest(".passive-open-path");
      if (openPathButton) {
        handlePassiveOpenPathButton(openPathButton, "Opened read path");
        return;
      }
    });
    $("neows-briefing").addEventListener("click", async (event) => {
      const focusButton = event.target.closest(".neo-briefing-focus");
      if (focusButton) {
        await focusNeoBriefingItem(focusButton.dataset.neoReferenceId, "Focused NEO");
        return;
      }
      const openPathButton = event.target.closest(".neo-open-path");
      if (openPathButton) {
        if (openPathButton.dataset.neoReferenceId) {
          selectNeoBriefingItem(openPathButton.dataset.neoReferenceId);
        }
        openPassivePath(openPathButton.dataset.path, "Opened NEO read path");
        return;
      }
      const briefingItem = event.target.closest(".neo-briefing-item");
      if (!briefingItem) return;
      selectNeoBriefingItem(briefingItem.dataset.neoReferenceId, "Selected NEO");
    });
    $("events-timeline").addEventListener("click", async (event) => {
      const button = event.target.closest(".event-dispatch");
      if (!button) return;
      const eventId = button.dataset.eventId;
      if (!eventId) return;
      $("refresh-status").textContent = "Dispatching event...";
      button.disabled = true;
      try {
        await dispatchEvent(eventId);
        await refreshAll();
      } catch (error) {
        $("refresh-status").textContent = error.message;
      } finally {
        button.disabled = false;
      }
    });
    $("event-queue").addEventListener("click", async (event) => {
      const button = event.target.closest(".event-dispatch");
      if (!button) return;
      const eventId = button.dataset.eventId;
      if (!eventId) return;
      $("refresh-status").textContent = "Dispatching event...";
      button.disabled = true;
      try {
        await dispatchEvent(eventId);
        await refreshAll();
      } catch (error) {
        $("refresh-status").textContent = error.message;
      } finally {
        button.disabled = false;
      }
    });
    $("passive-focus").addEventListener("click", async (event) => {
      const clearSelectedAttentionButton = event.target.closest(".passive-clear-selected-attention");
      if (clearSelectedAttentionButton) {
        clearPassiveSelectedAttention("Cleared selected attention");
        return;
      }
      const refocusCanonicalButton = event.target.closest(".passive-refocus-canonical");
      if (refocusCanonicalButton) {
        await handlePassiveRefocusCanonicalButton(refocusCanonicalButton, "Focused canonical event");
        return;
      }
      const refocusSiteButton = event.target.closest(".passive-refocus-site");
      if (refocusSiteButton) {
        await handlePassiveRefocusSiteButton(refocusSiteButton, "Focused site");
        return;
      }
      const workerDiagnosticsButton = event.target.closest(".passive-open-worker-diagnostics");
      if (workerDiagnosticsButton) {
        setPassiveSelectedAttentionFromDataset(workerDiagnosticsButton.dataset);
        await openPassiveWorkerDiagnostics(
          "Opened worker diagnostics",
          workerDiagnosticsButton.dataset.regionId || null,
        );
        return;
      }
      const leaseButton = event.target.closest(".passive-open-lease");
      if (leaseButton) {
        await openPassiveLease(leaseButton.dataset.regionId, "Opened region lease");
        return;
      }
      const heartbeatButton = event.target.closest(".passive-open-heartbeat");
      if (heartbeatButton) {
        await openPassiveHeartbeat(heartbeatButton.dataset.workerId, "Opened worker heartbeat");
        return;
      }
      const staleHeartbeatsButton = event.target.closest(".passive-open-stale-heartbeats");
      if (staleHeartbeatsButton) {
        setPassiveSelectedAttentionFromDataset(staleHeartbeatsButton.dataset);
        await openPassiveHeartbeatList(true, "Opened stale worker heartbeats");
        return;
      }
      const heartbeatListButton = event.target.closest(".passive-open-heartbeat-list");
      if (heartbeatListButton) {
        setPassiveSelectedAttentionFromDataset(heartbeatListButton.dataset);
        await openPassiveHeartbeatList(false, "Opened worker heartbeats");
        return;
      }
      const sourceSamplesButton = event.target.closest(".passive-open-source-samples");
      if (sourceSamplesButton) {
        setPassiveSelectedAttentionFromDataset(sourceSamplesButton.dataset);
        await openPassiveSourceSamples(sourceSamplesButton.dataset.sourceKind, "Opened source samples", sourceSamplesButton.dataset.regionId || null);
        return;
      }
      const regionRunsButton = event.target.closest(".passive-open-region-runs");
      if (regionRunsButton) {
        setPassiveSelectedAttentionFromDataset(regionRunsButton.dataset);
        await openPassiveRegionRuns(regionRunsButton.dataset.regionId, "Opened region runs");
        return;
      }
      const regionRunButton = event.target.closest(".passive-open-region-run");
      if (regionRunButton) {
        setPassiveSelectedAttentionFromDataset(regionRunButton.dataset);
        await openPassiveRegionRun(regionRunButton.dataset.runId, "Opened region run");
        return;
      }
      const regionOverviewButton = event.target.closest(".passive-open-region-overview");
      if (regionOverviewButton) {
        await handlePassiveOpenRegionOverviewButton(regionOverviewButton, "Opened region");
        return;
      }
      const pruneHeartbeatsButton = event.target.closest(".passive-prune-stale-heartbeats");
      if (pruneHeartbeatsButton) {
        setPassiveSelectedAttentionFromDataset(pruneHeartbeatsButton.dataset);
        pruneHeartbeatsButton.disabled = true;
        try {
          await pruneStaleHeartbeats("Pruned stale worker heartbeats");
        } catch (error) {
          $("refresh-status").textContent = error.message;
        } finally {
          pruneHeartbeatsButton.disabled = false;
        }
        return;
      }
      const timelineEntry = event.target.closest(".passive-timeline-entry");
      if (timelineEntry) {
        const canonicalId = timelineEntry.dataset.canonicalId;
        if (canonicalId) {
          focusPassiveCanonicalEvent(canonicalId, timelineEntry.dataset.siteId, "Timeline focus");
        }
        return;
      }
      const goMapButton = event.target.closest(".passive-go-map");
      if (goMapButton) {
        focusPassiveCanonicalEvent(goMapButton.dataset.canonicalId, goMapButton.dataset.siteId, "Focused canonical event");
        return;
      }
      const goSiteButton = event.target.closest(".passive-go-site");
      if (goSiteButton) {
        focusPassiveSite(goSiteButton.dataset.siteId, "Focused site");
        return;
      }
      const openPathButton = event.target.closest(".passive-open-path");
      if (openPathButton) {
        handlePassiveOpenPathButton(openPathButton, "Opened read path");
        return;
      }
      const evidenceButton = event.target.closest(".passive-open-evidence");
      if (evidenceButton) {
        handlePassiveOpenEvidenceButton(evidenceButton, "Opened evidence");
        return;
      }
      const runReplayButton = event.target.closest(".passive-run-replay");
      if (runReplayButton) {
        await handlePassiveRunReplayButton(runReplayButton, "Passive replay");
        return;
      }
      const replayButton = event.target.closest(".passive-replay");
      if (!replayButton) return;
      const manifestHash = replayButton.dataset.manifestHash;
      if (!manifestHash) return;
      replayButton.disabled = true;
      try {
        await runPassiveReplay(manifestHash, "Passive replay");
      } catch (error) {
        $("refresh-status").textContent = error.message;
      } finally {
        replayButton.disabled = false;
      }
    });
    $("passive-command-center").addEventListener("click", async (event) => {
      const regionOverviewButton = event.target.closest(".passive-open-region-overview");
      if (regionOverviewButton) {
        await handlePassiveOpenRegionOverviewButton(regionOverviewButton, "Opened region");
        return;
      }
      const regionRunsButton = event.target.closest(".passive-open-region-runs");
      if (regionRunsButton) {
        await openPassiveRegionRuns(regionRunsButton.dataset.regionId, "Opened region runs");
        return;
      }
      const workerDiagnosticsButton = event.target.closest(".passive-open-worker-diagnostics");
      if (workerDiagnosticsButton) {
        await openPassiveWorkerDiagnostics("Opened worker diagnostics", workerDiagnosticsButton.dataset.regionId || null);
        return;
      }
      const refocusCanonicalButton = event.target.closest(".passive-refocus-canonical");
      if (refocusCanonicalButton) {
        await handlePassiveRefocusCanonicalButton(refocusCanonicalButton, "Focused canonical event");
        return;
      }
      const refocusSiteButton = event.target.closest(".passive-refocus-site");
      if (refocusSiteButton) {
        await handlePassiveRefocusSiteButton(refocusSiteButton, "Focused site");
        return;
      }
    });
    $("canonical-events").addEventListener("click", async (event) => {
      const refocusCanonicalButton = event.target.closest(".passive-refocus-canonical");
      if (refocusCanonicalButton) {
        await handlePassiveRefocusCanonicalButton(refocusCanonicalButton, "Focused canonical event");
        return;
      }
      const regionRunsButton = event.target.closest(".passive-open-region-runs");
      if (regionRunsButton) {
        await openPassiveRegionRuns(regionRunsButton.dataset.regionId, "Opened region runs");
        return;
      }
    });
    $("refresh-canonical-events").addEventListener("click", async () => {
      $("refresh-status").textContent = "Refreshing canonical events...";
      try {
        await refreshPassiveCanonicalEvents();
        $("refresh-status").textContent = `Canonical events updated ${new Date().toLocaleTimeString()}`;
      } catch (error) {
        $("refresh-status").textContent = error.message;
      }
    });
    $("refresh-semantic-timeline").addEventListener("click", async () => {
      $("refresh-status").textContent = "Refreshing semantic timeline...";
      try {
        await refreshSemanticTimeline();
        $("refresh-status").textContent = `Semantic timeline updated ${new Date().toLocaleTimeString()}`;
      } catch (error) {
        $("refresh-status").textContent = error.message;
      }
    });
    $("semantic-timeline").addEventListener("click", async (event) => {
      // Quick action: View Event
      const viewBtn = event.target.closest(".stl-qa-view");
      if (viewBtn) {
        const { canonicalId, siteId, regionId } = viewBtn.dataset;
        $("refresh-status").textContent = "Opening event\u2026";
        try {
          const reached = await focusPassiveCanonicalFromAttention(canonicalId, siteId, regionId, "Timeline \u2192 Event");
          if (!reached) await focusPassiveSiteFromAttention(siteId, regionId, "Timeline \u2192 Site");
        } catch (e) { $("refresh-status").textContent = e.message; }
        return;
      }
      // Quick action: Open Site
      const siteBtn = event.target.closest(".stl-qa-site");
      if (siteBtn) {
        $("refresh-status").textContent = "Opening site\u2026";
        try {
          await focusPassiveSiteFromAttention(siteBtn.dataset.siteId, siteBtn.dataset.regionId, "Timeline \u2192 Site");
        } catch (e) { $("refresh-status").textContent = e.message; }
        return;
      }
      // Quick action: Replay
      const replayBtn = event.target.closest(".stl-qa-replay");
      if (replayBtn) {
        const hash = replayBtn.dataset.manifestHash;
        if (hash) {
          $("refresh-status").textContent = "Running replay\u2026";
          try { await handlePassiveRunReplayButton(replayBtn, "Timeline \u2192 Replay"); }
          catch (e) { $("refresh-status").textContent = e.message; }
        }
        return;
      }
      // Click on entry container itself → open event (fallback)
      const entry = event.target.closest(".passive-timeline-entry");
      if (!entry) return;
      const canonicalId = entry.dataset.canonicalId;
      const siteId = entry.dataset.siteId;
      const regionId = entry.dataset.regionId || state.selectedPassiveRegionId;
      $("refresh-status").textContent = "Opening\u2026";
      try {
        if (canonicalId && siteId) {
          const reached = await focusPassiveCanonicalFromAttention(canonicalId, siteId, regionId, "Timeline \u2192 Event");
          if (!reached) await focusPassiveSiteFromAttention(siteId, regionId, "Timeline \u2192 Site");
        } else if (siteId) {
          await focusPassiveSiteFromAttention(siteId, regionId, "Timeline \u2192 Site");
        }
      } catch (error) {
        $("refresh-status").textContent = error.message;
      }
    });
    $("event-type-filter").addEventListener("change", refreshAll);
    $("toggle-neo-layer").addEventListener("change", () => {
      updateGlobe(null, null, state.neoBriefing, state.closeApproaches);
      refreshAll();
    });
    $("toggle-close-approach-layer").addEventListener("change", () => {
      updateGlobe(null, null, state.neoBriefing, state.closeApproaches);
      refreshAll();
    });

    initGlobe();
    renderEmpty($("passive-focus"), "Select a passive region, site, or canonical event on the map.");
    refreshAll();
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
