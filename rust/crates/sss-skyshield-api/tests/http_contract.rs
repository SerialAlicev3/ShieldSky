use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

use sss_skyshield_api::{build_router, AppState};

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn site_crud_incident_authorization_and_audit_flow() {
    let app = build_router(AppState::in_memory());

    let site_id = Uuid::new_v4();
    let zone_id = Uuid::new_v4();
    let asset_id = Uuid::new_v4();
    let incident_id = Uuid::new_v4();

    let site = json!({
        "site_id": site_id,
        "name": "Solar Alpha",
        "site_type": "SolarPlant",
        "latitude": 38.7223,
        "longitude": -9.1393,
        "elevation_m": 90.0,
        "timezone": "Europe/Lisbon",
        "country_code": "PT",
        "criticality": "High",
        "operator_name": "Serial Alice Energy"
    });
    let app = assert_json_status(app, Method::POST, "/v1/sites", site, StatusCode::OK)
        .await
        .0;

    let zone = json!({
        "zone_id": zone_id,
        "site_id": Uuid::new_v4(),
        "name": "Transformer yard",
        "zone_type": "Substation",
        "polygon": [
            {"latitude": 38.7220, "longitude": -9.1390, "altitude_m": 0.0},
            {"latitude": 38.7225, "longitude": -9.1390, "altitude_m": 0.0},
            {"latitude": 38.7225, "longitude": -9.1385, "altitude_m": 0.0}
        ],
        "criticality": "Critical",
        "allowed_airspace_policy": "NoFly"
    });
    let app = assert_json_status(
        app,
        Method::POST,
        &format!("/v1/sites/{site_id}/zones"),
        zone,
        StatusCode::OK,
    )
    .await
    .0;

    let asset = json!({
        "asset_id": asset_id,
        "site_id": Uuid::new_v4(),
        "zone_id": zone_id,
        "asset_type": "Transformer",
        "criticality": "Critical",
        "operational_dependency_score": 0.96
    });
    let app = assert_json_status(
        app,
        Method::POST,
        &format!("/v1/sites/{site_id}/assets"),
        asset,
        StatusCode::OK,
    )
    .await
    .0;

    let exposure = json!({
        "site_id": Uuid::new_v4(),
        "drone_risk_sensitivity": 0.95,
        "rf_sensitivity": 0.8,
        "hail_sensitivity": 0.7,
        "wind_sensitivity": 0.6,
        "smoke_sensitivity": 0.5,
        "dust_sensitivity": 0.4,
        "irradiance_variability_sensitivity": 0.75
    });
    let app = assert_json_status(
        app,
        Method::POST,
        &format!("/v1/sites/{site_id}/exposure-profile"),
        exposure,
        StatusCode::OK,
    )
    .await
    .0;

    let signal_id = Uuid::new_v4();
    let signal = json!({
        "signal_id": signal_id,
        "site_id": Uuid::new_v4(),
        "source": "Rf",
        "signal_type": "RfBurst",
        "observed_at_unix_seconds": 1_800_000_010,
        "confidence_hint": 0.82,
        "payload": {
            "band": "2.4GHz",
            "summary": "short burst near no-fly zone"
        }
    });
    let app = assert_json_status(
        app,
        Method::POST,
        &format!("/v1/sites/{site_id}/signals"),
        signal,
        StatusCode::OK,
    )
    .await
    .0;
    let (app, signals) = assert_json_status(
        app,
        Method::GET,
        &format!("/v1/sites/{site_id}/signals"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(signals["data"][0]["signal_id"], signal_id.to_string());
    assert_eq!(signals["data"][0]["site_id"], site_id.to_string());

    let track_id = Uuid::new_v4();
    let track = json!({
        "track_id": track_id,
        "site_id": Uuid::new_v4(),
        "object_type": "Drone",
        "first_seen_unix_seconds": 1_800_000_000,
        "last_seen_unix_seconds": 1_800_000_120,
        "current_position": {
            "latitude": 38.7222,
            "longitude": -9.1388,
            "altitude_m": 42.0
        },
        "velocity_vector": {"x": 0.2, "y": 0.1, "z": 0.0},
        "altitude_m": 42.0,
        "rf_signature": "rf-test",
        "remote_id": null,
        "source_fusion_summary": "radar + RF confirmed low altitude drone"
    });
    let app = assert_json_status(
        app,
        Method::POST,
        &format!("/v1/sites/{site_id}/tracks"),
        track.clone(),
        StatusCode::OK,
    )
    .await
    .0;
    let (app, saved_track) = assert_json_status(
        app,
        Method::GET,
        &format!("/v1/sites/{site_id}/tracks/{track_id}"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(saved_track["data"]["track_id"], track_id.to_string());
    assert_eq!(saved_track["data"]["site_id"], site_id.to_string());
    let (app, classification) = assert_json_status(
        app,
        Method::POST,
        &format!("/v1/sites/{site_id}/tracks/{track_id}/classify"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        classification["data"]["threat_type"],
        "UnauthorizedOverflight"
    );

    let drone_assessment = json!({
        "track": track,
        "confidence": 0.88,
        "impact_probability": 0.86,
        "impact_window_seconds": 720,
        "current_owner": "site-soc"
    });
    let (app, drone_response) = assert_json_status(
        app,
        Method::POST,
        &format!("/v1/sites/{site_id}/drone-defense/assess"),
        drone_assessment,
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        drone_response["data"]["incident"]["incident_type"],
        "DroneDefense"
    );
    assert_eq!(drone_response["data"]["incident"]["status"], "Confirmed");
    assert_eq!(
        drone_response["data"]["incident"]["linked_track_ids"][0],
        track_id.to_string()
    );
    let assessment_id = drone_response["data"]["threat_assessment"]["assessment_id"]
        .as_str()
        .expect("assessment id");
    assessment_id
        .parse::<Uuid>()
        .expect("assessment id should be a uuid");
    assert_eq!(
        drone_response["data"]["threat_assessment"]["site_id"],
        site_id.to_string()
    );
    assert_eq!(
        drone_response["data"]["operational_decision"]["site_id"],
        site_id.to_string()
    );
    assert_eq!(
        drone_response["data"]["response_policy"]["site_id"],
        site_id.to_string()
    );
    let assessed_incident_id = drone_response["data"]["incident"]["incident_id"]
        .as_str()
        .expect("incident id")
        .to_string();
    assessed_incident_id
        .parse::<Uuid>()
        .expect("incident id should be a uuid");
    assert_eq!(
        drone_response["data"]["incident"]["site_id"],
        site_id.to_string()
    );
    let assessed_bundle_hashes = drone_response["data"]["incident"]["linked_bundle_hashes"]
        .as_array()
        .expect("bundle hashes");
    assert_eq!(assessed_bundle_hashes.len(), 1);
    let assessed_bundle_hash = assessed_bundle_hashes[0]
        .as_str()
        .expect("bundle hash")
        .to_string();
    assert!(!assessed_bundle_hash.is_empty());
    assert_eq!(
        drone_response["data"]["bundle_hash"],
        assessed_bundle_hash.as_str()
    );
    assert!(drone_response["data"]["manifest_hash"].is_string());
    assert!(drone_response["data"]["ranked_event_id"].is_string());
    assert_eq!(
        drone_response["data"]["operational_decision"]["escalation_policy"],
        "ExternalAuthorityRequired"
    );
    assert_eq!(
        drone_response["data"]["response_policy"]["requires_human_authorization"],
        true
    );
    assert!(drone_response["data"]["response_policy"]["allowed_actions"].is_array());
    assert!(drone_response["data"]["policy_evaluation"]["allowed"].is_boolean());
    assert!(drone_response["data"]["policy_evaluation"]["reason"].is_string());
    let (app, assessed_incident) = assert_json_status(
        app,
        Method::GET,
        &format!("/v1/incidents/{assessed_incident_id}"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        assessed_incident["data"]["incident_id"],
        assessed_incident_id
    );
    assert_eq!(assessed_incident["data"]["site_id"], site_id.to_string());
    assert_eq!(
        assessed_incident["data"]["linked_track_ids"][0],
        track_id.to_string()
    );
    assert_eq!(
        assessed_incident["data"]["linked_bundle_hashes"][0],
        assessed_bundle_hash
    );
    let assessed_authorization = json!({
        "action_type": "DispatchDroneInspection",
        "approved_by": "operator-assessed",
        "approval_basis": "confirmed track requires inspection route"
    });
    let app = assert_json_status(
        app,
        Method::POST,
        &format!("/v1/incidents/{assessed_incident_id}/authorize-response"),
        assessed_authorization,
        StatusCode::OK,
    )
    .await
    .0;
    let (app, assessed_dispatch) = assert_json_status(
        app,
        Method::POST,
        &format!("/v1/incidents/{assessed_incident_id}/dispatch-response"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(assessed_dispatch["data"]["status"], "ResponseInProgress");
    let (app, assessed_dispatches) = assert_json_status(
        app,
        Method::GET,
        &format!("/v1/incidents/{assessed_incident_id}/response-dispatches"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        assessed_dispatches["data"][0]["selected_action"],
        "DispatchDroneInspection"
    );
    assert_eq!(assessed_dispatches["data"][0]["route"], "AntiDrone");

    let incident = json!({
        "incident_id": incident_id,
        "site_id": Uuid::new_v4(),
        "incident_type": "DroneDefense",
        "status": "Confirmed",
        "created_at_unix_seconds": 1_800_000_000,
        "updated_at_unix_seconds": 1_800_000_000,
        "linked_track_ids": [],
        "linked_event_ids": [],
        "linked_bundle_hashes": ["bundle-test"],
        "severity": "Critical",
        "current_owner": "site-soc"
    });
    let app = assert_json_status(
        app,
        Method::POST,
        &format!("/v1/sites/{site_id}/incidents"),
        incident,
        StatusCode::OK,
    )
    .await
    .0;

    let (app, blocked_dispatch) = assert_json_status(
        app,
        Method::POST,
        &format!("/v1/incidents/{incident_id}/dispatch-response"),
        Value::Null,
        StatusCode::FORBIDDEN,
    )
    .await;
    assert_eq!(blocked_dispatch["error"]["code"], "response_not_authorized");
    let (app, empty_dispatches) = assert_json_status(
        app,
        Method::GET,
        &format!("/v1/incidents/{incident_id}/response-dispatches"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert!(empty_dispatches["data"]
        .as_array()
        .expect("dispatches")
        .is_empty());

    let authorization = json!({
        "action_type": "DispatchDroneIntercept",
        "approved_by": "operator-1",
        "approval_basis": "Confirmed no-fly violation over critical transformer yard"
    });
    let app = assert_json_status(
        app,
        Method::POST,
        &format!("/v1/incidents/{incident_id}/authorize-response"),
        authorization,
        StatusCode::OK,
    )
    .await
    .0;

    let (app, audit) = assert_json_status(
        app,
        Method::GET,
        &format!("/v1/incidents/{incident_id}/response-audit"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(audit["data"][0]["status"], "Approved");
    assert_eq!(audit["data"][0]["action_type"], "DispatchDroneIntercept");
    assert_eq!(audit["data"][0]["incident_id"], incident_id.to_string());

    let second_incident_id = Uuid::new_v4();
    let second_incident = json!({
        "incident_id": second_incident_id,
        "site_id": Uuid::new_v4(),
        "incident_type": "DroneDefense",
        "status": "Confirmed",
        "created_at_unix_seconds": 1_800_000_030,
        "updated_at_unix_seconds": 1_800_000_030,
        "linked_track_ids": [],
        "linked_event_ids": [],
        "linked_bundle_hashes": [],
        "severity": "Medium",
        "current_owner": "soc"
    });
    let app = assert_json_status(
        app,
        Method::POST,
        &format!("/v1/sites/{site_id}/incidents"),
        second_incident,
        StatusCode::OK,
    )
    .await
    .0;
    let (app, second_blocked_dispatch) = assert_json_status(
        app,
        Method::POST,
        &format!("/v1/incidents/{second_incident_id}/dispatch-response"),
        Value::Null,
        StatusCode::FORBIDDEN,
    )
    .await;
    assert_eq!(
        second_blocked_dispatch["error"]["code"],
        "response_not_authorized"
    );
    let (app, second_empty_dispatches) = assert_json_status(
        app,
        Method::GET,
        &format!("/v1/incidents/{second_incident_id}/response-dispatches"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert!(second_empty_dispatches["data"]
        .as_array()
        .expect("dispatches")
        .is_empty());
    let second_authorization = json!({
        "action_type": "ObserveOnly",
        "approved_by": "operator-2",
        "approval_basis": "separate incident"
    });
    let (app, audit_after_second_gate) = assert_json_status(
        assert_json_status(
            app,
            Method::POST,
            &format!("/v1/incidents/{second_incident_id}/authorize-response"),
            second_authorization,
            StatusCode::OK,
        )
        .await
        .0,
        Method::GET,
        &format!("/v1/incidents/{incident_id}/response-audit"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        audit_after_second_gate["data"]
            .as_array()
            .expect("audit array")
            .len(),
        1
    );
    assert_eq!(
        audit_after_second_gate["data"][0]["incident_id"],
        incident_id.to_string()
    );
    let (app, second_audit) = assert_json_status(
        app,
        Method::GET,
        &format!("/v1/incidents/{second_incident_id}/response-audit"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        second_audit["data"].as_array().expect("audit array").len(),
        1
    );
    assert_eq!(
        second_audit["data"][0]["incident_id"],
        second_incident_id.to_string()
    );
    assert_eq!(second_audit["data"][0]["action_type"], "ObserveOnly");

    let (app, overview) = assert_json_status(
        app,
        Method::GET,
        &format!("/v1/sites/{site_id}/overview"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(overview["data"]["site"]["site_id"], site_id.to_string());
    assert_eq!(overview["data"]["zones"][0]["site_id"], site_id.to_string());
    assert_eq!(
        overview["data"]["assets"][0]["site_id"],
        site_id.to_string()
    );
    assert_eq!(
        overview["data"]["exposure_profile"]["site_id"],
        site_id.to_string()
    );
    assert_eq!(
        overview["data"]["open_incidents"][0]["site_id"],
        site_id.to_string()
    );
    assert_eq!(
        overview["data"]["authorization_gates"][0]["site_id"],
        site_id.to_string()
    );
    assert_eq!(
        overview["data"]["recent_signals"][0]["site_id"],
        site_id.to_string()
    );
    assert_eq!(
        overview["data"]["active_tracks"][0]["site_id"],
        site_id.to_string()
    );
    let assessed_overview_incident = overview["data"]["open_incidents"]
        .as_array()
        .expect("open incidents")
        .iter()
        .find(|incident| incident["incident_id"].as_str() == Some(assessed_incident_id.as_str()))
        .expect("assessed incident should be present in overview");
    assert_eq!(
        assessed_overview_incident["linked_bundle_hashes"][0],
        assessed_bundle_hash
    );
    assert!(overview["data"]["response_dispatches"].is_array());

    let (app, dispatch_response_body) = assert_json_status(
        app,
        Method::POST,
        &format!("/v1/incidents/{incident_id}/dispatch-response"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        dispatch_response_body["data"]["status"],
        "ResponseInProgress"
    );
    let (app, dispatch_records) = assert_json_status(
        app,
        Method::GET,
        &format!("/v1/incidents/{incident_id}/response-dispatches"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        dispatch_records["data"][0]["incident_id"],
        incident_id.to_string()
    );
    assert_eq!(
        dispatch_records["data"][0]["selected_action"],
        "DispatchDroneIntercept"
    );
    assert_eq!(dispatch_records["data"][0]["route"], "AntiDrone");
    assert_eq!(
        dispatch_records["data"][0]["rationale"],
        "authorized defensive handoff route"
    );
    let (app, second_dispatch_records) = assert_json_status(
        app,
        Method::GET,
        &format!("/v1/incidents/{second_incident_id}/response-dispatches"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert!(second_dispatch_records["data"]
        .as_array()
        .expect("dispatches")
        .is_empty());
    let (_app, overview_after_dispatch) = assert_json_status(
        app,
        Method::GET,
        &format!("/v1/sites/{site_id}/overview"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    let manual_incident_id = incident_id.to_string();
    let dispatched_incident = overview_after_dispatch["data"]["response_dispatches"]
        .as_array()
        .expect("dispatches")
        .iter()
        .find(|dispatch| dispatch["incident_id"].as_str() == Some(manual_incident_id.as_str()))
        .expect("manual incident dispatch should be present");
    assert_eq!(dispatched_incident["incident_id"], manual_incident_id);
    assert_eq!(dispatched_incident["route"], "AntiDrone");
}

#[tokio::test]
async fn invalid_uuid_returns_contract_error() {
    let app = build_router(AppState::in_memory());
    let (_app, body) = assert_json_status(
        app,
        Method::GET,
        "/v1/sites/not-a-uuid",
        Value::Null,
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(body["error"]["code"], "invalid_request");
}

async fn assert_json_status(
    app: axum::Router,
    method: Method,
    uri: &str,
    payload: Value,
    expected_status: StatusCode,
) -> (axum::Router, Value) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    let body = if payload.is_null() {
        Body::empty()
    } else {
        Body::from(payload.to_string())
    };
    let response = app
        .clone()
        .oneshot(request.body(body).expect("request should build"))
        .await
        .expect("request should complete");
    assert_eq!(response.status(), expected_status);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should collect");
    let value = serde_json::from_slice(&body).expect("response should be json");
    (app, value)
}
