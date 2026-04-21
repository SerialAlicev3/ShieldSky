use std::sync::atomic::{AtomicU64, Ordering};

use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use sss_response::{PolicyEvaluation, ResponseDispatch};
use sss_site_registry::{ProtectedAsset, ProtectedZone, Site, SiteExposureProfile, SiteType};
use sss_skyshield::{
    assess_site_sky, AerialTrack, AssessSiteSkyInput, IncidentStatus, IncidentType, ObjectType,
    ResponsePolicy, Severity, SignalSource, SignalType, SiteContext, SiteIncident,
    SiteOperationalDecision, SkySignal, SkyThreatAssessment, ThreatType,
};

use crate::state::{
    AppState, AuthorizeResponseRequest, IncidentAssessmentRecord, SiteOverview, StateError,
};
use crate::types::{ApiEnvelope, ApiError, PlaceholderResponse};

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Deserialize)]
pub struct TimelineQuery {
    pub horizon: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DroneDefenseAssessmentRequest {
    pub track: AerialTrack,
    pub confidence: f32,
    pub impact_probability: f32,
    pub impact_window_seconds: i64,
    pub current_owner: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DroneDefenseAssessmentResponse {
    pub threat_assessment: SkyThreatAssessment,
    pub operational_decision: SiteOperationalDecision,
    pub response_policy: ResponsePolicy,
    pub policy_evaluation: PolicyEvaluation,
    pub incident: SiteIncident,
    pub bundle_hash: Option<String>,
    pub manifest_hash: Option<String>,
    pub ranked_event_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrackClassificationResponse {
    pub track_id: Uuid,
    pub site_id: Uuid,
    pub threat_type: ThreatType,
    pub classification_summary: String,
}

pub async fn health(headers: HeaderMap) -> Json<ApiEnvelope<PlaceholderResponse>> {
    placeholder(&headers, "/v1/health")
}

pub async fn version(headers: HeaderMap) -> Json<ApiEnvelope<PlaceholderResponse>> {
    placeholder(&headers, "/v1/version")
}

pub async fn create_site(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(site): Json<Site>,
) -> Result<Json<ApiEnvelope<Site>>, ApiError> {
    let request_id = request_id(&headers);
    let site = state
        .create_site(site)
        .map_err(|error| state_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, site)))
}

pub async fn list_sites(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiEnvelope<Vec<Site>>> {
    Json(ApiEnvelope::new(request_id(&headers), state.sites()))
}

pub async fn get_site(
    State(state): State<AppState>,
    Path(site_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<Site>>, ApiError> {
    let request_id = request_id(&headers);
    let site_id = parse_uuid(&request_id, &site_id)?;
    let Some(site) = state.site(site_id) else {
        return Err(ApiError::not_found(
            request_id,
            "site_not_found",
            format!("site not found: {site_id}"),
        ));
    };
    Ok(Json(ApiEnvelope::new(request_id, site)))
}

pub async fn create_zone(
    State(state): State<AppState>,
    Path(site_id): Path<String>,
    headers: HeaderMap,
    Json(zone): Json<ProtectedZone>,
) -> Result<Json<ApiEnvelope<ProtectedZone>>, ApiError> {
    let request_id = request_id(&headers);
    let site_id = parse_uuid(&request_id, &site_id)?;
    let zone = state
        .create_zone(site_id, zone)
        .map_err(|error| state_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, zone)))
}

pub async fn create_asset(
    State(state): State<AppState>,
    Path(site_id): Path<String>,
    headers: HeaderMap,
    Json(asset): Json<ProtectedAsset>,
) -> Result<Json<ApiEnvelope<ProtectedAsset>>, ApiError> {
    let request_id = request_id(&headers);
    let site_id = parse_uuid(&request_id, &site_id)?;
    let asset = state
        .create_asset(site_id, asset)
        .map_err(|error| state_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, asset)))
}

pub async fn create_exposure_profile(
    State(state): State<AppState>,
    Path(site_id): Path<String>,
    headers: HeaderMap,
    Json(profile): Json<SiteExposureProfile>,
) -> Result<Json<ApiEnvelope<SiteExposureProfile>>, ApiError> {
    let request_id = request_id(&headers);
    let site_id = parse_uuid(&request_id, &site_id)?;
    let profile = state
        .upsert_exposure_profile(site_id, profile)
        .map_err(|error| state_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, profile)))
}

pub async fn assess_sky(
    Path(_site_id): Path<String>,
    headers: HeaderMap,
) -> Json<ApiEnvelope<PlaceholderResponse>> {
    placeholder(&headers, "/v1/sites/:id/assess-sky")
}

pub async fn site_overview(
    State(state): State<AppState>,
    Path(site_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<SiteOverview>>, ApiError> {
    let request_id = request_id(&headers);
    let site_id = parse_uuid(&request_id, &site_id)?;
    let Some(overview) = state.site_overview(site_id) else {
        return Err(ApiError::not_found(
            request_id,
            "site_not_found",
            format!("site not found: {site_id}"),
        ));
    };
    Ok(Json(ApiEnvelope::new(request_id, overview)))
}

pub async fn site_timeline(
    Path(_site_id): Path<String>,
    Query(_query): Query<TimelineQuery>,
    headers: HeaderMap,
) -> Json<ApiEnvelope<PlaceholderResponse>> {
    placeholder(&headers, "/v1/sites/:id/timeline")
}

pub async fn create_signal(
    State(state): State<AppState>,
    Path(site_id): Path<String>,
    headers: HeaderMap,
    Json(signal): Json<SkySignal>,
) -> Result<Json<ApiEnvelope<SkySignal>>, ApiError> {
    let request_id = request_id(&headers);
    let site_id = parse_uuid(&request_id, &site_id)?;
    ensure_site_exists(&state, &request_id, site_id)?;
    let signal = state
        .create_signal(site_id, signal)
        .map_err(|error| state_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, signal)))
}

pub async fn list_signals(
    State(state): State<AppState>,
    Path(site_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<Vec<SkySignal>>>, ApiError> {
    let request_id = request_id(&headers);
    let site_id = parse_uuid(&request_id, &site_id)?;
    ensure_site_exists(&state, &request_id, site_id)?;
    Ok(Json(ApiEnvelope::new(
        request_id,
        state.site_signals(site_id),
    )))
}

pub async fn create_track(
    State(state): State<AppState>,
    Path(site_id): Path<String>,
    headers: HeaderMap,
    Json(track): Json<AerialTrack>,
) -> Result<Json<ApiEnvelope<AerialTrack>>, ApiError> {
    let request_id = request_id(&headers);
    let site_id = parse_uuid(&request_id, &site_id)?;
    ensure_site_exists(&state, &request_id, site_id)?;
    let track = state
        .create_track(site_id, track)
        .map_err(|error| state_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, track)))
}

pub async fn list_tracks(
    State(state): State<AppState>,
    Path(site_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<Vec<AerialTrack>>>, ApiError> {
    let request_id = request_id(&headers);
    let site_id = parse_uuid(&request_id, &site_id)?;
    ensure_site_exists(&state, &request_id, site_id)?;
    Ok(Json(ApiEnvelope::new(
        request_id,
        state.site_tracks(site_id),
    )))
}

pub async fn get_track(
    State(state): State<AppState>,
    Path((site_id, track_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<AerialTrack>>, ApiError> {
    let request_id = request_id(&headers);
    let site_id = parse_uuid(&request_id, &site_id)?;
    let track_id = parse_uuid(&request_id, &track_id)?;
    let Some(track) = state.track(site_id, track_id) else {
        return Err(ApiError::not_found(
            request_id,
            "track_not_found",
            format!("track not found: {track_id}"),
        ));
    };
    Ok(Json(ApiEnvelope::new(request_id, track)))
}

pub async fn classify_track(
    State(state): State<AppState>,
    Path((site_id, track_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<TrackClassificationResponse>>, ApiError> {
    let request_id = request_id(&headers);
    let site_id = parse_uuid(&request_id, &site_id)?;
    let track_id = parse_uuid(&request_id, &track_id)?;
    let Some(site) = state.site(site_id) else {
        return Err(ApiError::not_found(
            request_id,
            "site_not_found",
            format!("site not found: {site_id}"),
        ));
    };
    let Some(track) = state.track(site_id, track_id) else {
        return Err(ApiError::not_found(
            request_id,
            "track_not_found",
            format!("track not found: {track_id}"),
        ));
    };
    let threat_type =
        sss_skyshield::engines::classification::classify_track(&track, &site.site_type);
    Ok(Json(ApiEnvelope::new(
        request_id,
        TrackClassificationResponse {
            track_id,
            site_id,
            threat_type,
            classification_summary: format!(
                "{:?} classified as {:?} for {:?}",
                track.object_type, threat_type, site.site_type
            ),
        },
    )))
}

#[allow(clippy::too_many_lines)]
pub async fn assess_drone_defense(
    State(state): State<AppState>,
    Path(site_id): Path<String>,
    headers: HeaderMap,
    Json(mut request): Json<DroneDefenseAssessmentRequest>,
) -> Result<Json<ApiEnvelope<DroneDefenseAssessmentResponse>>, ApiError> {
    let request_id = request_id(&headers);
    let site_id = parse_uuid(&request_id, &site_id)?;
    let Some(overview) = state.site_overview(site_id) else {
        return Err(ApiError::not_found(
            request_id,
            "site_not_found",
            format!("site not found: {site_id}"),
        ));
    };
    let Some(zone) = overview.zones.iter().max_by_key(|zone| zone.criticality) else {
        return Err(ApiError::insufficient_evidence(
            request_id,
            "site has no protected zones for drone-defense impact assessment",
        ));
    };
    let Some(asset) = overview.assets.iter().max_by_key(|asset| asset.criticality) else {
        return Err(ApiError::insufficient_evidence(
            request_id,
            "site has no protected assets for drone-defense impact assessment",
        ));
    };
    let Some(profile) = overview.exposure_profile.as_ref() else {
        return Err(ApiError::insufficient_evidence(
            request_id,
            "site has no exposure profile for drone-defense impact assessment",
        ));
    };

    request.track.site_id = site_id;
    let track = state
        .create_track(site_id, request.track.clone())
        .map_err(|error| state_error(request_id.clone(), error))?;
    let threat_type =
        sss_skyshield::engines::classification::classify_track(&track, &overview.site.site_type);
    let exposure_sensitivity =
        sss_skyshield::application::evaluate_impact::zone_exposure_sensitivity(
            profile, zone, asset,
        );
    let risk_score = sss_skyshield::application::evaluate_impact::risk_score(
        request.confidence,
        request.impact_probability,
        zone.criticality,
        asset.criticality,
        exposure_sensitivity,
        request.impact_window_seconds,
    );
    let severity = severity_from_score(risk_score);
    let behavior = sss_skyshield::engines::behavior::behavior_summary(&track);
    let impact = sss_skyshield::engines::impact::impact_summary(
        &overview.site.name,
        threat_type,
        zone,
        asset,
    );
    let predicted_path_summary = sss_skyshield::engines::prediction::predicted_path_summary(&track);

    let core = core_artifacts_for_assessment(&state, &overview, &request, &track);
    let linked_bundle_hashes = core
        .as_ref()
        .map(|output| vec![output.core.evidence_bundle.bundle_hash.clone()])
        .unwrap_or_default();
    let threat_assessment = SkyThreatAssessment {
        assessment_id: Uuid::new_v4(),
        site_id,
        threat_type,
        confidence: request.confidence,
        impact_probability: request.impact_probability,
        impact_window_seconds: request.impact_window_seconds,
        affected_zones: vec![zone.zone_id],
        affected_assets: vec![asset.asset_id],
        explanation: format!("{behavior}; {impact}; risk_score={risk_score:.2}"),
        predicted_path_summary,
    };
    let operational_decision = SiteOperationalDecision {
        decision_id: Uuid::new_v4(),
        site_id,
        severity,
        priority: priority_from_score(risk_score),
        recommendation: "Defensive response requires human authorization gate before handoff."
            .to_string(),
        recipients: sss_skyshield::application::decide_recipients::recipients_for(
            threat_type,
            severity,
        ),
        escalation_policy: sss_skyshield::application::decide_recipients::escalation_policy_for(
            threat_type,
            severity,
        ),
        authorized_response_options:
            sss_skyshield::application::decide_recipients::response_options_for(
                threat_type,
                severity,
            ),
    };
    let now = chrono_like_now();
    let incident = SiteIncident {
        incident_id: Uuid::new_v4(),
        site_id,
        incident_type: IncidentType::DroneDefense,
        status: IncidentStatus::Confirmed,
        created_at_unix_seconds: now,
        updated_at_unix_seconds: now,
        linked_track_ids: vec![track.track_id],
        linked_event_ids: Vec::new(),
        linked_bundle_hashes,
        severity,
        current_owner: request.current_owner,
    };
    let response_policy = default_response_policy(site_id, overview.site.site_type, threat_type);
    let policy_evaluation =
        sss_response::policy_allows(&response_policy, &threat_assessment, &incident);
    let incident = state
        .create_incident(incident)
        .map_err(|error| state_error(request_id.clone(), error))?;
    state
        .store_incident_assessment(IncidentAssessmentRecord {
            incident_id: incident.incident_id,
            threat_assessment: threat_assessment.clone(),
            operational_decision: operational_decision.clone(),
            response_policy: response_policy.clone(),
            policy_evaluation: policy_evaluation.clone(),
            bundle_hash: core
                .as_ref()
                .map(|output| output.core.evidence_bundle.bundle_hash.clone()),
            manifest_hash: core
                .as_ref()
                .map(|output| output.core.replay_manifest.manifest_hash.clone()),
            ranked_event_id: core
                .as_ref()
                .map(|output| output.core.ranked_event.event_id.clone()),
        })
        .map_err(|error| state_error(request_id.clone(), error))?;

    Ok(Json(ApiEnvelope::new(
        request_id,
        DroneDefenseAssessmentResponse {
            threat_assessment,
            operational_decision,
            response_policy,
            policy_evaluation,
            incident,
            bundle_hash: core
                .as_ref()
                .map(|output| output.core.evidence_bundle.bundle_hash.clone()),
            manifest_hash: core
                .as_ref()
                .map(|output| output.core.replay_manifest.manifest_hash.clone()),
            ranked_event_id: core
                .as_ref()
                .map(|output| output.core.ranked_event.event_id.clone()),
        },
    )))
}

pub async fn create_site_incident(
    State(state): State<AppState>,
    Path(site_id): Path<String>,
    headers: HeaderMap,
    Json(mut incident): Json<SiteIncident>,
) -> Result<Json<ApiEnvelope<SiteIncident>>, ApiError> {
    let request_id = request_id(&headers);
    incident.site_id = parse_uuid(&request_id, &site_id)?;
    let incident = state
        .create_incident(incident)
        .map_err(|error| state_error(request_id.clone(), error))?;
    Ok(Json(ApiEnvelope::new(request_id, incident)))
}

pub async fn list_site_incidents(
    State(state): State<AppState>,
    Path(site_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<Vec<SiteIncident>>>, ApiError> {
    let request_id = request_id(&headers);
    let site_id = parse_uuid(&request_id, &site_id)?;
    Ok(Json(ApiEnvelope::new(
        request_id,
        state.site_incidents(site_id),
    )))
}

pub async fn get_incident(
    State(state): State<AppState>,
    Path(incident_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<SiteIncident>>, ApiError> {
    let request_id = request_id(&headers);
    let incident_id = parse_uuid(&request_id, &incident_id)?;
    let Some(incident) = state.incident(incident_id) else {
        return Err(ApiError::not_found(
            request_id,
            "incident_not_found",
            format!("incident not found: {incident_id}"),
        ));
    };
    Ok(Json(ApiEnvelope::new(request_id, incident)))
}

pub async fn escalate_incident(
    State(state): State<AppState>,
    Path(incident_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<SiteIncident>>, ApiError> {
    update_incident_status(&state, &headers, &incident_id, IncidentStatus::Escalated)
}

pub async fn close_incident(
    State(state): State<AppState>,
    Path(incident_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<SiteIncident>>, ApiError> {
    update_incident_status(&state, &headers, &incident_id, IncidentStatus::Closed)
}

pub async fn authorize_response(
    State(state): State<AppState>,
    Path(incident_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<AuthorizeResponseRequest>,
) -> Result<Json<ApiEnvelope<sss_skyshield::AuthorizationGate>>, ApiError> {
    let request_id = request_id(&headers);
    let incident_id = parse_uuid(&request_id, &incident_id)?;
    if let Some(assessment) = state.incident_assessment(incident_id) {
        if !assessment.policy_evaluation.allowed {
            return Err(ApiError::response_not_authorized(
                request_id,
                format!(
                    "response policy is not satisfied for incident {}: {}",
                    incident_id, assessment.policy_evaluation.reason
                ),
            ));
        }
        if !assessment
            .response_policy
            .allowed_actions
            .contains(&request.action_type)
        {
            return Err(ApiError::response_not_authorized(
                request_id,
                format!(
                    "action {:?} is not allowed by the persisted response policy",
                    request.action_type
                ),
            ));
        }
    }
    let Some(gate) = state
        .authorize_response(incident_id, request)
        .map_err(|error| state_error(request_id.clone(), error))?
    else {
        return Err(ApiError::not_found(
            request_id,
            "incident_not_found",
            format!("incident not found: {incident_id}"),
        ));
    };
    Ok(Json(ApiEnvelope::new(request_id, gate)))
}

pub async fn dispatch_response(
    State(state): State<AppState>,
    Path(incident_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<SiteIncident>>, ApiError> {
    let request_id = request_id(&headers);
    let parsed_incident_id = parse_uuid(&request_id, &incident_id)?;
    let Some(incident) = state.incident(parsed_incident_id) else {
        return Err(ApiError::not_found(
            request_id,
            "incident_not_found",
            format!("incident not found: {parsed_incident_id}"),
        ));
    };
    if incident.status != IncidentStatus::ResponseAuthorized {
        return Err(ApiError::response_not_authorized(
            request_id,
            "response dispatch requires an approved authorization gate",
        ));
    }
    let Some(site) = state.site(incident.site_id) else {
        return Err(ApiError::not_found(
            request_id,
            "site_not_found",
            format!("site not found: {}", incident.site_id),
        ));
    };
    let Some(gate) = state.response_audit(parsed_incident_id).and_then(|gates| {
        gates
            .into_iter()
            .rev()
            .find(|gate| gate.status == sss_skyshield::GateStatus::Approved)
    }) else {
        return Err(ApiError::response_not_authorized(
            request_id,
            "response dispatch requires an approved authorization gate",
        ));
    };
    let policy = state.incident_assessment(parsed_incident_id).map_or_else(
        || ResponsePolicy {
            allowed_actions: vec![gate.action_type],
            ..default_response_policy(
                site.site_id,
                site.site_type,
                ThreatType::UnauthorizedOverflight,
            )
        },
        |record| record.response_policy,
    );
    if !policy.allowed_actions.contains(&gate.action_type) {
        return Err(ApiError::response_not_authorized(
            request_id,
            "approved authorization gate action is outside the persisted response policy",
        ));
    }
    let Some(dispatch) =
        sss_response::action_router::route_for_action(&policy, &incident, gate.action_type)
    else {
        return Err(ApiError::response_not_authorized(
            request_id,
            "approved authorization gate has no defensive route",
        ));
    };
    state
        .record_response_dispatch(dispatch)
        .map_err(|error| state_error(request_id.clone(), error))?;
    update_incident_status(
        &state,
        &headers,
        &incident_id,
        IncidentStatus::ResponseInProgress,
    )
}

pub async fn response_audit(
    State(state): State<AppState>,
    Path(incident_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<Vec<sss_skyshield::AuthorizationGate>>>, ApiError> {
    let request_id = request_id(&headers);
    let incident_id = parse_uuid(&request_id, &incident_id)?;
    let Some(audit) = state.response_audit(incident_id) else {
        return Err(ApiError::not_found(
            request_id,
            "incident_not_found",
            format!("incident not found: {incident_id}"),
        ));
    };
    Ok(Json(ApiEnvelope::new(request_id, audit)))
}

pub async fn list_response_dispatches(
    State(state): State<AppState>,
    Path(incident_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<ApiEnvelope<Vec<ResponseDispatch>>>, ApiError> {
    let request_id = request_id(&headers);
    let incident_id = parse_uuid(&request_id, &incident_id)?;
    let Some(dispatches) = state.response_dispatches(incident_id) else {
        return Err(ApiError::not_found(
            request_id,
            "incident_not_found",
            format!("incident not found: {incident_id}"),
        ));
    };
    Ok(Json(ApiEnvelope::new(request_id, dispatches)))
}

pub async fn get_evidence(
    Path(_bundle_hash): Path<String>,
    headers: HeaderMap,
) -> Json<ApiEnvelope<PlaceholderResponse>> {
    placeholder(&headers, "/v1/evidence/:bundle_hash")
}

pub async fn get_replay(
    Path(_manifest_hash): Path<String>,
    headers: HeaderMap,
) -> Json<ApiEnvelope<PlaceholderResponse>> {
    placeholder(&headers, "/v1/replay/:manifest_hash")
}

fn update_incident_status(
    state: &AppState,
    headers: &HeaderMap,
    incident_id: &str,
    status: IncidentStatus,
) -> Result<Json<ApiEnvelope<SiteIncident>>, ApiError> {
    let request_id = request_id(headers);
    let incident_id = parse_uuid(&request_id, incident_id)?;
    let Some(incident) = state
        .update_incident_status(incident_id, status)
        .map_err(|error| state_error(request_id.clone(), error))?
    else {
        return Err(ApiError::not_found(
            request_id,
            "incident_not_found",
            format!("incident not found: {incident_id}"),
        ));
    };
    Ok(Json(ApiEnvelope::new(request_id, incident)))
}

fn placeholder(
    headers: &HeaderMap,
    endpoint: &'static str,
) -> Json<ApiEnvelope<PlaceholderResponse>> {
    Json(ApiEnvelope::new(
        request_id(headers),
        PlaceholderResponse {
            status: "scaffolded",
            endpoint,
        },
    ))
}

fn parse_uuid(request_id: &str, value: &str) -> Result<Uuid, ApiError> {
    value
        .parse()
        .map_err(|error| ApiError::invalid_request(request_id.to_string(), error))
}

fn core_artifacts_for_assessment(
    state: &AppState,
    overview: &SiteOverview,
    request: &DroneDefenseAssessmentRequest,
    track: &AerialTrack,
) -> Option<sss_skyshield::AssessSiteSkyOutput> {
    let exposure_profile = overview.exposure_profile.clone()?;
    let mut signals = state.site_signals(overview.site.site_id);
    if signals.is_empty() {
        signals.push(SkySignal {
            signal_id: Uuid::new_v4(),
            site_id: overview.site.site_id,
            source: if track.rf_signature.is_some() {
                SignalSource::Rf
            } else {
                SignalSource::Radar
            },
            signal_type: if track.object_type == ObjectType::Drone {
                SignalType::TrackDetection
            } else {
                SignalType::RemoteId
            },
            observed_at_unix_seconds: track.last_seen_unix_seconds,
            confidence_hint: request.confidence,
            payload: serde_json::json!({
                "track_id": track.track_id,
                "object_type": format!("{:?}", track.object_type),
                "rf_signature": track.rf_signature,
                "remote_id": track.remote_id,
                "source_fusion_summary": track.source_fusion_summary,
            }),
        });
    }
    assess_site_sky(&AssessSiteSkyInput {
        site: SiteContext {
            site: overview.site.clone(),
            zones: overview.zones.clone(),
            assets: overview.assets.clone(),
            exposure_profile,
        },
        signals,
    })
}

fn ensure_site_exists(state: &AppState, request_id: &str, site_id: Uuid) -> Result<(), ApiError> {
    if state.site(site_id).is_some() {
        Ok(())
    } else {
        Err(ApiError::not_found(
            request_id.to_string(),
            "site_not_found",
            format!("site not found: {site_id}"),
        ))
    }
}

fn state_error(request_id: String, error: StateError) -> ApiError {
    let message = match error {
        StateError::Io(error) => error.to_string(),
        StateError::Serde(error) => error.to_string(),
    };
    ApiError::storage(request_id, message)
}

fn severity_from_score(score: f32) -> Severity {
    if score >= 0.85 {
        Severity::Critical
    } else if score >= 0.55 {
        Severity::High
    } else if score >= 0.25 {
        Severity::Medium
    } else {
        Severity::Low
    }
}

fn default_response_policy(
    site_id: Uuid,
    site_type: SiteType,
    threat_type: ThreatType,
) -> ResponsePolicy {
    match site_type {
        SiteType::DataCenter => {
            sss_skyshield::policies::datacenter::default_datacenter_policy(site_id, threat_type)
        }
        SiteType::SolarPlant | SiteType::Substation => {
            sss_skyshield::policies::solar::default_solar_policy(site_id, threat_type)
        }
    }
}

fn priority_from_score(score: f32) -> u8 {
    let target = (score.clamp(0.01, 1.0) * 100.0).round();
    let mut priority = 1;
    while f32::from(priority) < target && priority < 100 {
        priority += 1;
    }
    priority
}

fn chrono_like_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or_default()
}

fn request_id(headers: &HeaderMap) -> String {
    headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map_or_else(
            || {
                let id = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
                format!("skyshield-req-{id}")
            },
            str::to_string,
        )
}
