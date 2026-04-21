use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::{
    assess_drone_defense, assess_sky, authorize_response, classify_track, close_incident,
    create_asset, create_exposure_profile, create_signal, create_site, create_site_incident,
    create_track, create_zone, dispatch_response, escalate_incident, get_evidence, get_incident,
    get_replay, get_site, get_track, health, list_response_dispatches, list_signals,
    list_site_incidents, list_sites, list_tracks, response_audit, site_overview, site_timeline,
    version,
};
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/v1/health", get(health))
        .route("/v1/version", get(version))
        .route("/v1/sites", post(create_site).get(list_sites))
        .route("/v1/sites/:id", get(get_site))
        .route("/v1/sites/:id/zones", post(create_zone))
        .route("/v1/sites/:id/assets", post(create_asset))
        .route(
            "/v1/sites/:id/exposure-profile",
            post(create_exposure_profile),
        )
        .route("/v1/sites/:id/assess-sky", post(assess_sky))
        .route("/v1/sites/:id/overview", get(site_overview))
        .route("/v1/sites/:id/timeline", get(site_timeline))
        .route(
            "/v1/sites/:id/signals",
            post(create_signal).get(list_signals),
        )
        .route("/v1/sites/:id/tracks", post(create_track).get(list_tracks))
        .route("/v1/sites/:id/tracks/:track_id", get(get_track))
        .route(
            "/v1/sites/:id/tracks/:track_id/classify",
            post(classify_track),
        )
        .route(
            "/v1/sites/:id/drone-defense/assess",
            post(assess_drone_defense),
        )
        .route(
            "/v1/sites/:id/incidents",
            post(create_site_incident).get(list_site_incidents),
        )
        .route("/v1/incidents/:incident_id", get(get_incident))
        .route(
            "/v1/incidents/:incident_id/escalate",
            post(escalate_incident),
        )
        .route("/v1/incidents/:incident_id/close", post(close_incident))
        .route(
            "/v1/incidents/:incident_id/authorize-response",
            post(authorize_response),
        )
        .route(
            "/v1/incidents/:incident_id/dispatch-response",
            post(dispatch_response),
        )
        .route(
            "/v1/incidents/:incident_id/response-audit",
            get(response_audit),
        )
        .route(
            "/v1/incidents/:incident_id/response-dispatches",
            get(list_response_dispatches),
        )
        .route("/v1/evidence/:bundle_hash", get(get_evidence))
        .route("/v1/replay/:manifest_hash", get(get_replay))
        .with_state(state)
}
