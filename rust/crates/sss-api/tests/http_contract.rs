use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{OriginalUri, State};
use axum::http::{Request, StatusCode};
use axum::{
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tower::ServiceExt;

use sss_api::state::PassiveRegionTargetRequest;
use sss_api::{build_router, AppState, WebhookDeliveryPolicy};
use sss_core::{
    AnticipationEngine, BehaviorSignature, DigitalTwin, MissionClass, Observation,
    ObservationSource, OrbitRegime, OrbitalState, RiskProfile, SpaceObject, Vector3,
};
use sss_passive_scanner::PassiveSourceKind;
use sss_storage::{
    PassiveRegionRunLog, PassiveRegionRunStatus, PassiveRunOrigin, PassiveSourceHealthSample,
    PassiveWorkerHeartbeat, SqliteStore,
};

const CELESTRAK_ISS_SNAPSHOT: &str = "\
ISS (ZARYA)
1 25544U 98067A   20194.88612269 -.00002218  00000-0 -31515-4 0  9992
2 25544  51.6461 221.2784 0001413  89.1723 280.4612 15.49507896236008
";

#[derive(Clone)]
struct WebhookTestState {
    received: Arc<Mutex<Vec<Value>>>,
    attempts: Arc<AtomicUsize>,
    fail_first: bool,
}

#[derive(Clone)]
struct NasaApodTestState {
    requests: Arc<Mutex<Vec<String>>>,
}

#[derive(Clone)]
struct NasaNeoWsTestState {
    requests: Arc<Mutex<Vec<String>>>,
}

#[derive(Clone)]
struct PassiveFeedsTestState {
    requests: Arc<Mutex<Vec<String>>>,
}

#[derive(Clone)]
struct OverpassTestState {
    bodies: Arc<Mutex<Vec<String>>>,
}

#[derive(Clone)]
struct PassiveDiscoveryAndFeedsState {
    requests: Arc<Mutex<Vec<String>>>,
    bodies: Arc<Mutex<Vec<String>>>,
}

#[tokio::test]
async fn health_returns_versioned_envelope() {
    let response = build_router(AppState::demo().expect("state"))
        .oneshot(
            Request::builder()
                .uri("/v1/health")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["api_version"], "sss-api-v1");
    assert_eq!(body["data"]["status"], "ok");
}

#[tokio::test]
async fn passive_scan_persists_and_exposes_site_corpus() {
    let app = build_router(AppState::demo().expect("state"));

    let scan_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/passive/scan")
                .header("content-type", "application/json")
                .body(Body::from(sample_passive_scan_request().to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(scan_response.status(), StatusCode::OK);
    let scan_body = body_json(scan_response).await;
    assert_eq!(
        scan_body["data"]["sites"][0]["site"]["name"],
        "Seville Solar South"
    );
    assert!(!scan_body["data"]["core_envelopes"]
        .as_array()
        .expect("core envelopes")
        .is_empty());

    let sites_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/sites")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(sites_response.status(), StatusCode::OK);
    let sites_body = body_json(sites_response).await;
    let site_id = sites_body["data"][0]["site"]["site_id"]
        .as_str()
        .expect("site id");

    let events_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/v1/passive/sites/{site_id}/events?limit=5"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(events_response.status(), StatusCode::OK);
    let events_body = body_json(events_response).await;
    assert!(!events_body["data"].as_array().expect("events").is_empty());

    let risk_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/v1/passive/sites/{site_id}/risk-history?limit=5"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(risk_response.status(), StatusCode::OK);
    let risk_body = body_json(risk_response).await;
    assert_eq!(risk_body["data"][0]["site_name"], "Seville Solar South");

    let pattern_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/v1/passive/sites/{site_id}/patterns?limit=5"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(pattern_response.status(), StatusCode::OK);
    let pattern_body = body_json(pattern_response).await;
    assert!(!pattern_body["data"]
        .as_array()
        .expect("patterns")
        .is_empty());

    let observations_response = app
        .oneshot(
            Request::builder()
                .uri("/v1/passive/observations?limit=5&source_kind=Adsb")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(observations_response.status(), StatusCode::OK);
    let observations_body = body_json(observations_response).await;
    assert!(observations_body["data"]
        .as_array()
        .expect("observations")
        .iter()
        .all(|observation| observation["source_kind"] == "Adsb"));
}

#[tokio::test]
async fn passive_worker_heartbeats_can_filter_stale_entries() {
    let state = AppState::demo().expect("state");
    state
        .store_passive_worker_heartbeat(&PassiveWorkerHeartbeat {
            worker_id: "worker-stale".to_string(),
            started_at_unix_seconds: 100,
            last_heartbeat_unix_seconds: 120,
            status: "idle".to_string(),
            current_region_id: None,
            current_phase: "cycle_complete".to_string(),
            last_error: None,
            version: "test".to_string(),
        })
        .expect("stale heartbeat stored");
    state
        .store_passive_worker_heartbeat(&PassiveWorkerHeartbeat {
            worker_id: "worker-fresh".to_string(),
            started_at_unix_seconds: 200,
            last_heartbeat_unix_seconds: sss_api::state::now_unix_seconds(),
            status: "running".to_string(),
            current_region_id: Some("region-a".to_string()),
            current_phase: "region_run".to_string(),
            last_error: None,
            version: "test".to_string(),
        })
        .expect("fresh heartbeat stored");

    let response = build_router(state)
        .oneshot(
            Request::builder()
                .uri("/v1/passive/worker/heartbeats?limit=10&stale_only=true")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    let items = body["data"].as_array().expect("stale heartbeats array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["worker_id"], "worker-stale");
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn passive_worker_diagnostics_and_prune_work() {
    let state = AppState::demo().expect("state");
    let now = sss_api::state::now_unix_seconds();
    state
        .upsert_passive_region_target(&PassiveRegionTargetRequest {
            region_id: Some("region-a".to_string()),
            name: "Region A".to_string(),
            south: 38.0,
            west: -8.5,
            north: 38.5,
            east: -8.0,
            site_types: None,
            timezone: Some("Europe/Lisbon".to_string()),
            country_code: Some("PT".to_string()),
            default_operator_name: Some("Ops".to_string()),
            default_criticality: None,
            observation_radius_km: Some(25.0),
            discovery_cadence_seconds: Some(86_400),
            scan_limit: Some(10),
            minimum_priority: Some(0.2),
            enabled: Some(true),
        })
        .expect("region stored");
    state
        .store_passive_worker_heartbeat(&PassiveWorkerHeartbeat {
            worker_id: "worker-stale".to_string(),
            started_at_unix_seconds: 100,
            last_heartbeat_unix_seconds: 120,
            status: "idle".to_string(),
            current_region_id: None,
            current_phase: "cycle_complete".to_string(),
            last_error: Some("network drift".to_string()),
            version: "test".to_string(),
        })
        .expect("stale heartbeat stored");
    state
        .store_passive_worker_heartbeat(&PassiveWorkerHeartbeat {
            worker_id: "worker-region-a".to_string(),
            started_at_unix_seconds: now.saturating_sub(120),
            last_heartbeat_unix_seconds: now.saturating_sub(10),
            status: "running".to_string(),
            current_region_id: Some("region-a".to_string()),
            current_phase: "region_run".to_string(),
            last_error: None,
            version: "test".to_string(),
        })
        .expect("region heartbeat stored");
    state
        .acquire_passive_region_lease(
            "region-a",
            "worker-region-a",
            "run-region-a",
            now.saturating_sub(30),
            300,
        )
        .expect("lease store")
        .expect("lease acquired");
    state
        .store_passive_region_run_log_entry(&PassiveRegionRunLog {
            run_id: "run-region-a".to_string(),
            request_id: "req-region-a".to_string(),
            origin: PassiveRunOrigin::Worker,
            started_at_unix_seconds: now.saturating_sub(90),
            finished_at_unix_seconds: now.saturating_sub(30),
            duration_ms: 60_000,
            status: PassiveRegionRunStatus::Completed,
            region_ids: vec!["region-a".to_string()],
            discovery_triggered: true,
            evaluated_region_count: 1,
            discovered_region_count: 1,
            skipped_region_count: 0,
            discovered_seed_count: 3,
            selected_seed_count: 2,
            event_count: 4,
            sources_used: vec!["Weather".to_string(), "Adsb".to_string()],
            source_errors: vec!["Weather timeout".to_string()],
            next_run_at_unix_seconds: Some(now.saturating_add(600)),
        })
        .expect("run log stored");
    state
        .store_passive_source_health_sample_entries(&[
            PassiveSourceHealthSample {
                sample_id: "sample-weather-a".to_string(),
                request_id: "req-region-a".to_string(),
                region_id: Some("region-a".to_string()),
                source_kind: PassiveSourceKind::Weather,
                fetched: true,
                observations_collected: 3,
                detail: "forecast ok".to_string(),
                generated_at_unix_seconds: now.saturating_sub(20),
                window_start_unix_seconds: now.saturating_sub(3_600),
                window_end_unix_seconds: now,
            },
            PassiveSourceHealthSample {
                sample_id: "sample-adsb-a".to_string(),
                request_id: "req-region-a".to_string(),
                region_id: Some("region-a".to_string()),
                source_kind: PassiveSourceKind::Adsb,
                fetched: false,
                observations_collected: 0,
                detail: "adsb timeout".to_string(),
                generated_at_unix_seconds: now.saturating_sub(10),
                window_start_unix_seconds: now.saturating_sub(3_600),
                window_end_unix_seconds: now,
            },
        ])
        .expect("source samples stored");

    let app = build_router(state.clone());

    let diagnostics_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/worker/diagnostics?limit=10&stale_after_seconds=180&region_id=region-a")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(diagnostics_response.status(), StatusCode::OK);
    let diagnostics_body = body_json(diagnostics_response).await;
    assert_eq!(diagnostics_body["data"]["region_id"], "region-a");
    assert_eq!(diagnostics_body["data"]["stale_worker_heartbeat_count"], 0);
    assert_eq!(diagnostics_body["data"]["active_region_lease_count"], 1);
    assert_eq!(
        diagnostics_body["data"]["active_region_leases"][0]["region_id"],
        "region-a"
    );
    assert_eq!(
        diagnostics_body["data"]["region_metrics"][0]["region_id"],
        "region-a"
    );
    assert_eq!(
        diagnostics_body["data"]["region_metrics"][0]["run_count"],
        1
    );
    assert_eq!(
        diagnostics_body["data"]["region_metrics"][0]["event_count"],
        4
    );
    assert_eq!(
        diagnostics_body["data"]["source_metrics"][0]["region_id"],
        "region-a"
    );
    let remediation_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/regions/region-a/remediation")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(remediation_response.status(), StatusCode::OK);
    let remediation_body = body_json(remediation_response).await;
    assert_eq!(remediation_body["data"]["region_id"], "region-a");
    assert!(remediation_body["data"]["summary"].is_string());
    assert!(remediation_body["data"]["actions"].is_array());
    assert!(remediation_body["data"]["actions"][0]["suggested_read_paths"].is_array());
    assert!(remediation_body["data"]["evidence"]["diagnostics_recommendation"].is_string());
    assert!(diagnostics_body["data"]["source_metrics"]
        .as_array()
        .expect("source metrics")
        .iter()
        .any(|metric| metric["source"] == "Adsb" && metric["failure_count"] == 1));
    assert!(diagnostics_body["data"]["source_metrics"]
        .as_array()
        .expect("source metrics")
        .iter()
        .any(|metric| metric["source"] == "Adsb"
            && metric["health_status"] == "degraded"
            && metric["reliability_score"].is_number()
            && metric["staleness_seconds"].is_number()));
    let region_source_samples_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/source-health/samples?limit=10&source_kind=Adsb&region_id=region-a")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(region_source_samples_response.status(), StatusCode::OK);
    let region_source_samples_body = body_json(region_source_samples_response).await;
    let region_source_samples = region_source_samples_body["data"]
        .as_array()
        .expect("region source samples");
    assert_eq!(region_source_samples.len(), 1);
    assert_eq!(region_source_samples[0]["region_id"], "region-a");
    assert_eq!(region_source_samples[0]["source_kind"], "Adsb");
    let maintenance_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/maintenance/summary?heartbeat_retention_seconds=180&source_retention_seconds=1&source_kind=Adsb&region_id=region-a")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(maintenance_response.status(), StatusCode::OK);
    let maintenance_body = body_json(maintenance_response).await;
    assert_eq!(maintenance_body["data"]["region_id"], "region-a");
    assert_eq!(maintenance_body["data"]["source_kind"], "Adsb");
    assert_eq!(
        maintenance_body["data"]["source_health_prune_candidate_count"],
        1
    );
    assert!(maintenance_body["data"]["suggested_actions"]
        .as_array()
        .expect("maintenance actions")
        .iter()
        .any(|action| action["path"] == "/v1/passive/source-health/samples/prune"));
    assert!(maintenance_body["data"]["suggested_actions"]
        .as_array()
        .expect("maintenance actions")
        .iter()
        .flat_map(|action| {
            action["confirmation_read_paths"]
                .as_array()
                .into_iter()
                .flatten()
        })
        .any(|path| path
            .as_str()
            .unwrap_or_default()
            .contains("/v1/passive/source-health/samples?limit=20")));
    let source_prune_dry_run_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/passive/source-health/samples/prune")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "older_than_seconds": 1,
                        "source_kind": "Adsb",
                        "region_id": "region-a",
                        "dry_run": true
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(source_prune_dry_run_response.status(), StatusCode::OK);
    let source_prune_dry_run_body = body_json(source_prune_dry_run_response).await;
    assert_eq!(source_prune_dry_run_body["data"]["dry_run"], true);
    assert_eq!(source_prune_dry_run_body["data"]["pruned_count"], 1);
    assert_eq!(
        state
            .passive_source_health_samples(10, Some(PassiveSourceKind::Adsb), Some("region-a"))
            .expect("region source samples after dry run")
            .len(),
        1
    );
    let source_prune_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/passive/source-health/samples/prune")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "older_than_seconds": 1,
                        "source_kind": "Adsb",
                        "region_id": "region-a",
                        "dry_run": false
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(source_prune_response.status(), StatusCode::OK);
    let source_prune_body = body_json(source_prune_response).await;
    assert_eq!(source_prune_body["data"]["region_id"], "region-a");
    assert_eq!(source_prune_body["data"]["source_kind"], "Adsb");
    assert_eq!(source_prune_body["data"]["dry_run"], false);
    assert_eq!(source_prune_body["data"]["pruned_count"], 1);
    assert!(state
        .passive_source_health_samples(10, Some(PassiveSourceKind::Adsb), Some("region-a"))
        .expect("region source samples after prune")
        .is_empty());

    let global_diagnostics_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/worker/diagnostics?limit=10&stale_after_seconds=180")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(global_diagnostics_response.status(), StatusCode::OK);
    let global_diagnostics_body = body_json(global_diagnostics_response).await;
    assert_eq!(
        global_diagnostics_body["data"]["stale_worker_heartbeat_count"],
        1
    );
    assert_eq!(
        global_diagnostics_body["data"]["stale_worker_heartbeats"][0]["worker_id"],
        "worker-stale"
    );

    let prune_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/passive/worker/heartbeats/prune")
                .header("content-type", "application/json")
                .body(Body::from(json!({ "older_than_seconds": 180 }).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(prune_response.status(), StatusCode::OK);
    let prune_body = body_json(prune_response).await;
    assert_eq!(prune_body["data"]["pruned_count"], 1);

    let remaining_heartbeats = state
        .passive_worker_heartbeats(10)
        .expect("remaining heartbeats");
    assert_eq!(remaining_heartbeats.len(), 1);
    assert_eq!(remaining_heartbeats[0].worker_id, "worker-region-a");
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn passive_map_regions_expose_operational_pressure() {
    let state = AppState::demo().expect("state");
    let now = sss_api::state::now_unix_seconds();
    let region = state
        .upsert_passive_region_target(&PassiveRegionTargetRequest {
            region_id: Some("pressure-region".to_string()),
            name: "Pressure Region".to_string(),
            south: 36.0,
            west: -6.0,
            north: 36.5,
            east: -5.5,
            site_types: None,
            timezone: Some("Europe/Lisbon".to_string()),
            country_code: Some("PT".to_string()),
            default_operator_name: Some("Pressure Ops".to_string()),
            default_criticality: None,
            observation_radius_km: Some(25.0),
            discovery_cadence_seconds: Some(86_400),
            scan_limit: Some(10),
            minimum_priority: Some(0.2),
            enabled: Some(true),
        })
        .expect("region stored");
    assert_eq!(region.region_id, "pressure-region");

    state
        .acquire_passive_region_lease(
            "pressure-region",
            "worker-stale-lease",
            "run-stale-lease",
            now.saturating_sub(600),
            60,
        )
        .expect("stale lease store")
        .expect("stale lease acquired");
    state
        .store_passive_worker_heartbeat(&PassiveWorkerHeartbeat {
            worker_id: "worker-pressure-active".to_string(),
            started_at_unix_seconds: now.saturating_sub(120),
            last_heartbeat_unix_seconds: now.saturating_sub(10),
            status: "running".to_string(),
            current_region_id: Some("pressure-region".to_string()),
            current_phase: "region_run".to_string(),
            last_error: None,
            version: "test".to_string(),
        })
        .expect("active heartbeat stored");
    state
        .store_passive_worker_heartbeat(&PassiveWorkerHeartbeat {
            worker_id: "worker-pressure-stale".to_string(),
            started_at_unix_seconds: now.saturating_sub(900),
            last_heartbeat_unix_seconds: now.saturating_sub(600),
            status: "degraded".to_string(),
            current_region_id: Some("pressure-region".to_string()),
            current_phase: "lease_lost".to_string(),
            last_error: Some("region lease could not be renewed".to_string()),
            version: "test".to_string(),
        })
        .expect("stale heartbeat stored");
    state
        .store_passive_region_run_log_entry(&PassiveRegionRunLog {
            run_id: "pressure-run-failed".to_string(),
            request_id: "pressure-req-failed".to_string(),
            origin: PassiveRunOrigin::Worker,
            started_at_unix_seconds: now.saturating_sub(180),
            finished_at_unix_seconds: now.saturating_sub(20),
            duration_ms: 160_000,
            status: PassiveRegionRunStatus::Failed,
            region_ids: vec!["pressure-region".to_string()],
            discovery_triggered: false,
            evaluated_region_count: 1,
            discovered_region_count: 0,
            skipped_region_count: 0,
            discovered_seed_count: 0,
            selected_seed_count: 0,
            event_count: 0,
            sources_used: vec!["Weather".to_string()],
            source_errors: vec!["lease_lost: region lease lost during execution".to_string()],
            next_run_at_unix_seconds: Some(now.saturating_add(600)),
        })
        .expect("failed run stored");
    state
        .store_passive_region_run_log_entry(&PassiveRegionRunLog {
            run_id: "pressure-run-partial".to_string(),
            request_id: "pressure-req-partial".to_string(),
            origin: PassiveRunOrigin::Worker,
            started_at_unix_seconds: now.saturating_sub(300),
            finished_at_unix_seconds: now.saturating_sub(40),
            duration_ms: 240_000,
            status: PassiveRegionRunStatus::Partial,
            region_ids: vec!["pressure-region".to_string()],
            discovery_triggered: false,
            evaluated_region_count: 1,
            discovered_region_count: 0,
            skipped_region_count: 0,
            discovered_seed_count: 0,
            selected_seed_count: 0,
            event_count: 0,
            sources_used: vec!["Adsb".to_string()],
            source_errors: vec!["adsb timeout".to_string()],
            next_run_at_unix_seconds: Some(now.saturating_add(600)),
        })
        .expect("partial run stored");
    state
        .store_passive_source_health_sample_entries(&[PassiveSourceHealthSample {
            sample_id: "pressure-source-weather".to_string(),
            request_id: "pressure-req-partial".to_string(),
            region_id: Some("pressure-region".to_string()),
            source_kind: PassiveSourceKind::Weather,
            fetched: false,
            observations_collected: 0,
            detail: "weather timeout".to_string(),
            generated_at_unix_seconds: now.saturating_sub(30),
            window_start_unix_seconds: now.saturating_sub(3_600),
            window_end_unix_seconds: now,
        }])
        .expect("source sample stored");

    let app = build_router(state);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/map/regions")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    let region = body["data"]["regions"]
        .as_array()
        .expect("regions")
        .iter()
        .find(|item| item["region_id"] == "pressure-region")
        .expect("pressure region");

    assert_eq!(region["status"], "degraded");
    assert_eq!(region["active_lease_count"], 0);
    assert_eq!(region["stale_lease_count"], 1);
    assert_eq!(region["active_worker_count"], 1);
    assert_eq!(region["stale_worker_count"], 1);
    assert_eq!(region["recent_failed_run_count"], 1);
    assert_eq!(region["recent_partial_run_count"], 1);
    assert_eq!(region["recent_lease_loss_count"], 1);
    assert_eq!(region["latest_run_status"], "Failed");
    assert_eq!(region["operational_pressure_priority"], "critical");
    assert!(region["operational_summary"]
        .as_str()
        .unwrap_or_default()
        .contains("lease-loss signals"));

    let remediation_app = app.clone();
    let timeline_response = app
        .oneshot(
            Request::builder()
                .uri("/v1/passive/regions/pressure-region/operational-timeline?window_hours=24&bucket_hours=6&include_empty_buckets=false&min_priority=high")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(timeline_response.status(), StatusCode::OK);
    let timeline_body = body_json(timeline_response).await;
    assert_eq!(timeline_body["data"]["region_id"], "pressure-region");
    assert_eq!(timeline_body["data"]["window_hours"], 24);
    assert_eq!(timeline_body["data"]["bucket_hours"], 6);
    assert!(
        timeline_body["data"]["current_snapshot"]["pressure_score"]
            .as_f64()
            .unwrap_or_default()
            >= 0.85
    );
    assert_eq!(
        timeline_body["data"]["current_snapshot"]["pressure_priority"],
        "critical"
    );
    assert!(timeline_body["data"]["buckets"]
        .as_array()
        .expect("buckets")
        .iter()
        .any(|bucket| bucket["lease_loss_count"] == 1 && bucket["source_failure_count"] == 1));
    assert!(timeline_body["data"]["buckets"]
        .as_array()
        .expect("buckets")
        .iter()
        .all(|bucket| bucket["pressure_priority"] == "high"
            || bucket["pressure_priority"] == "critical"));
    assert!(timeline_body["data"]["buckets"]
        .as_array()
        .expect("buckets")
        .iter()
        .any(|bucket| bucket["suggested_read_paths"]
            .as_array()
            .expect("bucket read paths")
            .iter()
            .any(|path| path.as_str().unwrap_or_default().contains(
                "/v1/passive/source-health/samples?limit=20&region_id=pressure-region"
            ))));

    let remediation_response = remediation_app
        .oneshot(
            Request::builder()
                .uri("/v1/passive/regions/pressure-region/remediation")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(remediation_response.status(), StatusCode::OK);
    let remediation_body = body_json(remediation_response).await;
    assert_eq!(remediation_body["data"]["region_id"], "pressure-region");
    assert!(remediation_body["data"]["evidence"]["pressure_buckets"]
        .as_array()
        .expect("pressure buckets")
        .iter()
        .any(|bucket| bucket["pressure_priority"] == "high"
            || bucket["pressure_priority"] == "critical"));
    assert!(remediation_body["data"]["evidence"]["pressure_buckets"]
        .as_array()
        .expect("pressure buckets")
        .iter()
        .any(|bucket| bucket["suggested_read_paths"].is_array()));
    assert!(remediation_body["data"]["actions"]
        .as_array()
        .expect("remediation actions")
        .iter()
        .any(|action| action["source"] == "operational_timeline"
            && action["suggested_read_paths"]
                .as_array()
                .expect("read paths")
                .iter()
                .any(|path| path
                    .as_str()
                    .unwrap_or_default()
                    .contains("/operational-timeline?window_hours=72&bucket_hours=6&include_empty_buckets=false"))));
}

#[tokio::test]
async fn passive_region_remediation_returns_healthy_monitor_only_when_region_is_clean() {
    let state = AppState::demo().expect("state");
    state
        .upsert_passive_region_target(&PassiveRegionTargetRequest {
            region_id: Some("healthy-region".to_string()),
            name: "Healthy Region".to_string(),
            south: 40.0,
            west: -8.0,
            north: 40.5,
            east: -7.5,
            site_types: None,
            timezone: Some("Europe/Lisbon".to_string()),
            country_code: Some("PT".to_string()),
            default_operator_name: Some("Healthy Ops".to_string()),
            default_criticality: None,
            observation_radius_km: Some(20.0),
            discovery_cadence_seconds: Some(86_400),
            scan_limit: Some(10),
            minimum_priority: Some(0.1),
            enabled: Some(true),
        })
        .expect("region stored");

    let response = build_router(state)
        .oneshot(
            Request::builder()
                .uri("/v1/passive/regions/healthy-region/remediation")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["data"]["region_id"], "healthy-region");
    assert_eq!(body["data"]["posture"], "healthy");
    assert_eq!(body["data"]["actions"].as_array().map_or(0, Vec::len), 1);
    assert_eq!(body["data"]["actions"][0]["action_id"], "monitor-region");
    assert_eq!(body["data"]["actions"][0]["source"], "overview");
    assert!(body["data"]["actions"][0]["suggested_read_paths"].is_array());
    assert_eq!(body["data"]["evidence"]["stale_lease_count"], 0);
    assert_eq!(body["data"]["evidence"]["stale_worker_count"], 0);
    assert_eq!(body["data"]["evidence"]["recent_failed_run_count"], 0);
    assert_eq!(body["data"]["evidence"]["recent_partial_run_count"], 0);
    assert_eq!(body["data"]["evidence"]["recent_lease_loss_count"], 0);
    }

    #[tokio::test]
    async fn passive_region_semantic_timeline_smoke() {
        let app = build_router(AppState::demo().expect("state"));
        // Cria uma região para garantir region_id válido
        let region_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/passive/regions")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "name": "Iberian Peninsula",
                            "south": 35.0,
                            "west": -10.0,
                            "north": 44.0,
                            "east": 5.0,
                            "country_code": "ES",
                            "timezone": "Europe/Madrid"
                        })
                        .to_string(),
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(region_response.status(), StatusCode::OK);
        let region_body = body_json(region_response).await;
        let region_id = region_body["data"]["region_id"]
            .as_str()
            .expect("region_id in creation response");

        // Consulta a semantic timeline da região
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/v1/passive/regions/{region_id}/semantic-timeline?limit=10"
                    ))
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        let body = body_json(response).await;
        assert!(body["data"].is_object() || body["data"].is_array(), "semantic timeline returns object or array");
        // Não exige entradas, mas garante shape e ausência de erro
    }

#[tokio::test]
async fn passive_region_remediation_returns_not_found_for_unknown_region() {
    let app = build_router(AppState::demo().expect("state"));
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/regions/missing-region/remediation")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = body_json(response).await;
    assert_eq!(body["error"]["code"], "passive_region_not_found");
    assert!(body["error"]["message"]
        .as_str()
        .unwrap_or_default()
        .contains("passive region not found: missing-region"));

    let timeline_response = app
        .oneshot(
            Request::builder()
                .uri("/v1/passive/regions/missing-region/operational-timeline")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(timeline_response.status(), StatusCode::NOT_FOUND);
    let timeline_body = body_json(timeline_response).await;
    assert_eq!(timeline_body["error"]["code"], "passive_region_not_found");
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn passive_live_scan_fetches_real_feed_shapes_and_builds_narrative() {
    let requests = Arc::new(Mutex::new(Vec::<String>::new()));
    let base_url = spawn_passive_feeds_server(requests.clone()).await;
    let app = build_router(
        AppState::demo()
            .expect("state")
            .with_open_meteo_base_url(base_url.clone())
            .with_firms_api_base_url(base_url.clone())
            .with_firms_map_key("test-firms-key")
            .with_opensky_api_base_url(base_url)
            .with_opensky_bearer_token("test-opensky-token"),
    );

    let live_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/passive/scan/live")
                .header("content-type", "application/json")
                .body(Body::from(sample_passive_live_scan_request().to_string()))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(live_response.status(), StatusCode::OK);
    let live_body = body_json(live_response).await;
    assert_eq!(
        live_body["data"]["source_statuses"][0]["source_kind"],
        "Adsb"
    );
    assert_eq!(
        live_body["data"]["source_statuses"][1]["source_kind"],
        "Weather"
    );
    assert_eq!(
        live_body["data"]["source_statuses"][2]["source_kind"],
        "FireSmoke"
    );
    let source_samples_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/source-health/samples?limit=10")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(source_samples_response.status(), StatusCode::OK);
    let source_samples_body = body_json(source_samples_response).await;
    let samples = source_samples_body["data"].as_array().expect("samples");
    assert!(!samples.is_empty());
    assert!(samples.iter().any(|sample| sample["source_kind"] == "Adsb"));
    assert!(samples
        .iter()
        .any(|sample| sample["source_kind"] == "Weather"));
    assert!(samples
        .iter()
        .any(|sample| sample["source_kind"] == "FireSmoke"));
    assert!(samples[0]["sample_id"].is_string());
    assert!(samples[0]["request_id"].is_string());
    assert!(samples[0]["fetched"].is_boolean());
    assert!(samples[0]["observations_collected"].is_number());
    assert!(samples[0]["detail"].is_string());
    assert!(samples[0]["generated_at_unix_seconds"].is_number());
    assert!(samples[0]["window_start_unix_seconds"].is_number());
    assert!(samples[0]["window_end_unix_seconds"].is_number());
    let weather_samples_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/source-health/samples?limit=10&source_kind=Weather")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(weather_samples_response.status(), StatusCode::OK);
    let weather_samples_body = body_json(weather_samples_response).await;
    assert!(weather_samples_body["data"]
        .as_array()
        .expect("weather samples")
        .iter()
        .all(|sample| sample["source_kind"] == "Weather"));
    assert_eq!(
        live_body["data"]["scan"]["sites"][0]["site"]["name"],
        "Seville Solar South"
    );
    assert!(!live_body["data"]["scan"]["events"]
        .as_array()
        .expect("events")
        .is_empty());

    let sites_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/sites")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    let sites_body = body_json(sites_response).await;
    let site_id = sites_body["data"][0]["site"]["site_id"]
        .as_str()
        .expect("site id");

    let overview_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/v1/passive/sites/{site_id}/overview?limit=10"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(overview_response.status(), StatusCode::OK);
    let overview_body = body_json(overview_response).await;
    assert_eq!(
        overview_body["data"]["site"]["site"]["name"],
        "Seville Solar South"
    );
    assert!(overview_body["data"]["seed_lifecycle"]["scan_priority"].is_number());
    assert!(overview_body["data"]["seed_lifecycle"]["last_scanned_at_unix_seconds"].is_number());
    assert!(overview_body["data"]["risk_history_narrative"]["narrative"]
        .as_str()
        .unwrap_or_default()
        .contains("Nos ultimos 30 dias"));
    assert!(overview_body["data"]["risk_history_narrative"]["observation_confidence"].is_number());
    assert!(
        overview_body["data"]["risk_history_narrative"]["provenance"]["canonical_event_ids"]
            .is_array()
    );
    assert!(
        overview_body["data"]["risk_history_narrative"]["provenance"]["bundle_hashes"].is_array()
    );
    assert!(
        overview_body["data"]["risk_history_narrative"]["provenance"]["manifest_hashes"].is_array()
    );
    assert!(
        overview_body["data"]["risk_history_narrative"]["provenance"]["top_drivers"].is_array()
    );
    assert!(
        overview_body["data"]["risk_history_narrative"]["provenance"]["site_overview_path"]
            .as_str()
            .unwrap_or_default()
            .contains("/v1/passive/sites/")
    );
    assert!(
        overview_body["data"]["risk_history_narrative"]["provenance"]["evidence_paths"].is_array()
    );
    assert!(
        overview_body["data"]["risk_history_narrative"]["provenance"]["replay_paths"].is_array()
    );

    let narrative_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/v1/passive/sites/{site_id}/narrative?days=30"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(narrative_response.status(), StatusCode::OK);
    let narrative_body = body_json(narrative_response).await;
    assert!(
        narrative_body["data"]["event_count"]
            .as_u64()
            .unwrap_or_default()
            >= 1
    );
    assert!(narrative_body["data"]["narrative"]
        .as_str()
        .unwrap_or_default()
        .contains("eventos de anomalia"));
    assert!(narrative_body["data"]["risk_direction"].is_string());
    assert!(narrative_body["data"]["provenance"]["canonical_event_ids"].is_array());
    assert!(narrative_body["data"]["provenance"]["top_drivers"].is_array());
    assert!(narrative_body["data"]["provenance"]["bundle_hashes"].is_array());
    assert!(narrative_body["data"]["provenance"]["manifest_hashes"].is_array());
    assert!(narrative_body["data"]["provenance"]["canonical_event_paths"].is_array());
    assert!(narrative_body["data"]["provenance"]["site_narrative_path"]
        .as_str()
        .unwrap_or_default()
        .contains("/narrative?days=30"));

    let seed_registry_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/seeds?limit=10")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(seed_registry_response.status(), StatusCode::OK);
    let seed_registry_body = body_json(seed_registry_response).await;
    let seed_key = seed_registry_body["data"][0]["seed_key"]
        .as_str()
        .expect("seed key");
    assert!(seed_registry_body["data"][0]["scan_priority"].is_number());

    let seed_detail_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/v1/passive/seeds/{seed_key}"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(seed_detail_response.status(), StatusCode::OK);
    let seed_detail_body = body_json(seed_detail_response).await;
    assert_eq!(seed_detail_body["data"]["seed_key"], seed_key);
    assert!(seed_detail_body["data"]["last_scanned_at_unix_seconds"].is_number());

    let recorded_requests = requests.lock().await.clone();
    assert!(recorded_requests
        .iter()
        .any(|entry| entry.starts_with("/v1/forecast?")));
    assert!(recorded_requests
        .iter()
        .any(|entry| entry.starts_with("/api/area/csv/test-firms-key/")));
    assert!(recorded_requests
        .iter()
        .any(|entry| entry.starts_with("/states/all?")));
}

#[tokio::test]
async fn passive_site_discovery_fetches_public_infra_seeds() {
    let bodies = Arc::new(Mutex::new(Vec::<String>::new()));
    let overpass_url = spawn_overpass_server(bodies.clone()).await;
    let app = build_router(
        AppState::demo()
            .expect("state")
            .with_overpass_api_base_url(overpass_url),
    );

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/passive/discover-sites")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "south": 37.30,
                        "west": -6.10,
                        "north": 37.50,
                        "east": -5.80,
                        "timezone": "Europe/Madrid",
                        "country_code": "ES",
                        "default_operator_name": "OSM Passive Intel",
                        "observation_radius_km": 20.0
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    let seeds = body["data"]["seeds"].as_array().expect("seeds");
    assert_eq!(seeds.len(), 3);
    assert!(seeds.iter().any(|seed| {
        seed["site_type"] == "SolarPlant" && seed["name"] == "Seville Solar South"
    }));
    assert!(seeds
        .iter()
        .any(|seed| { seed["site_type"] == "DataCenter" && seed["name"] == "Andalucia Data Hub" }));
    assert!(seeds
        .iter()
        .any(|seed| { seed["site_type"] == "Substation" && seed["name"] == "Seville Grid East" }));

    let seeds_registry_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/seeds?limit=10")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(seeds_registry_response.status(), StatusCode::OK);
    let seeds_registry_body = body_json(seeds_registry_response).await;
    assert_eq!(
        seeds_registry_body["data"].as_array().map_or(0, Vec::len),
        3
    );
    assert_eq!(
        seeds_registry_body["data"][0]["classification_status"],
        "Discovered"
    );
    assert_eq!(seeds_registry_body["data"][0]["seed_status"], "New");

    let recorded_body = bodies.lock().await[0].clone();
    assert!(recorded_body.contains("power"));
    assert!(recorded_body.contains("telecom"));
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn passive_discover_and_scan_builds_region_level_operational_output() {
    let requests = Arc::new(Mutex::new(Vec::<String>::new()));
    let bodies = Arc::new(Mutex::new(Vec::<String>::new()));
    let base_url = spawn_passive_discovery_and_feeds_server(requests.clone(), bodies.clone()).await;
    let app = build_router(
        AppState::demo()
            .expect("state")
            .with_overpass_api_base_url(format!("{base_url}/api/interpreter"))
            .with_open_meteo_base_url(base_url.clone())
            .with_firms_api_base_url(base_url.clone())
            .with_firms_map_key("test-firms-key")
            .with_opensky_api_base_url(base_url)
            .with_opensky_bearer_token("test-opensky-token"),
    );

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/passive/discover-and-scan")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "discovery": {
                            "south": 37.30,
                            "west": -6.10,
                            "north": 37.50,
                            "east": -5.80,
                            "timezone": "Europe/Madrid",
                            "country_code": "ES",
                            "default_operator_name": "OSM Passive Intel",
                            "observation_radius_km": 20.0
                        },
                        "window_hours": 24,
                        "include_adsb": true,
                        "include_weather": true,
                        "include_fire_smoke": true
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(
        body["data"]["discovery"]["seeds"]
            .as_array()
            .map_or(0, Vec::len),
        3
    );
    assert_eq!(
        body["data"]["live_scan"]["scan"]["sites"]
            .as_array()
            .map_or(0, Vec::len),
        3
    );
    assert!(body["data"]["execution_summary"]["highest_scan_priority"].is_number());
    assert!(
        body["data"]["execution_summary"]["scanned_seed_count"]
            .as_u64()
            .unwrap_or_default()
            >= 1
    );
    assert!(!body["data"]["site_overviews"]
        .as_array()
        .expect("site_overviews")
        .is_empty());
    assert!(body["data"]["site_overviews"][0]["seed_lifecycle"]["scan_priority"].is_number());
    assert!(
        body["data"]["site_overviews"][0]["risk_history_narrative"]["narrative"]
            .as_str()
            .unwrap_or_default()
            .contains("Nos ultimos 30 dias")
    );
    assert!(
        body["data"]["site_overviews"][0]["risk_history_narrative"]["narrative"]
            .as_str()
            .unwrap_or_default()
            .contains("confianca observacional")
    );

    let seed_key = body["data"]["discovery"]["seeds"][0]["source_reference"]
        .as_str()
        .expect("seed key");
    let seed_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/v1/passive/seeds/{seed_key}"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(seed_response.status(), StatusCode::OK);
    let seed_body = body_json(seed_response).await;
    assert!(seed_body["data"]["last_scanned_at_unix_seconds"].is_number());
    assert_ne!(seed_body["data"]["classification_status"], "Discovered");

    let recorded_requests = requests.lock().await.clone();
    assert!(recorded_requests
        .iter()
        .any(|entry| entry.starts_with("/v1/forecast?")));
    assert!(recorded_requests
        .iter()
        .any(|entry| entry.starts_with("/states/all?")));
    assert!(recorded_requests
        .iter()
        .any(|entry| entry.starts_with("/api/area/csv/test-firms-key/")));
    let recorded_body = bodies.lock().await[0].clone();
    assert!(recorded_body.contains("data="));
    assert!(recorded_body.contains("power"));
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn passive_scheduler_scans_due_seeds_and_skips_fresh_ones() {
    let requests = Arc::new(Mutex::new(Vec::<String>::new()));
    let bodies = Arc::new(Mutex::new(Vec::<String>::new()));
    let base_url = spawn_passive_discovery_and_feeds_server(requests.clone(), bodies.clone()).await;
    let app = build_router(
        AppState::demo()
            .expect("state")
            .with_overpass_api_base_url(format!("{base_url}/api/interpreter"))
            .with_open_meteo_base_url(base_url.clone())
            .with_firms_api_base_url(base_url.clone())
            .with_firms_map_key("test-firms-key")
            .with_opensky_api_base_url(base_url)
            .with_opensky_bearer_token("test-opensky-token"),
    );

    let discovery_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/passive/discover-sites")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "south": 37.30,
                        "west": -6.10,
                        "north": 37.50,
                        "east": -5.80,
                        "timezone": "Europe/Madrid",
                        "country_code": "ES",
                        "default_operator_name": "OSM Passive Intel",
                        "observation_radius_km": 20.0
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(discovery_response.status(), StatusCode::OK);

    let first_scheduler_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/passive/scheduler/run")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "limit": 10,
                        "window_hours": 24,
                        "include_adsb": true,
                        "include_weather": true,
                        "include_fire_smoke": true
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(first_scheduler_response.status(), StatusCode::OK);
    let first_body = body_json(first_scheduler_response).await;
    assert_eq!(first_body["data"]["evaluated_seed_count"], 3);
    assert_eq!(first_body["data"]["selected_seed_count"], 3);
    assert_eq!(
        first_body["data"]["planned_seeds"]
            .as_array()
            .map_or(0, Vec::len),
        3
    );
    assert!(first_body["data"]["planned_seeds"][0]["cadence_seconds"].is_number());
    assert!(first_body["data"]["live_scan"]["scan"]["sites"].is_array());
    assert!(first_body["data"]["execution_summary"]["event_count"].is_number());

    let request_count_after_first_run = requests.lock().await.len();
    assert!(request_count_after_first_run > 0);

    let second_scheduler_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/passive/scheduler/run")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "limit": 10,
                        "window_hours": 24,
                        "include_adsb": true,
                        "include_weather": true,
                        "include_fire_smoke": true
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(second_scheduler_response.status(), StatusCode::OK);
    let second_body = body_json(second_scheduler_response).await;
    assert_eq!(second_body["data"]["evaluated_seed_count"], 3);
    assert_eq!(second_body["data"]["selected_seed_count"], 0);
    assert_eq!(second_body["data"]["skipped_not_due_count"], 3);
    assert!(second_body["data"]["live_scan"].is_null());

    assert_eq!(requests.lock().await.len(), request_count_after_first_run);
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn passive_regions_persist_and_orchestrate_discovery_then_scheduler() {
    let requests = Arc::new(Mutex::new(Vec::<String>::new()));
    let bodies = Arc::new(Mutex::new(Vec::<String>::new()));
    let base_url = spawn_passive_discovery_and_feeds_server(requests.clone(), bodies.clone()).await;
    let app = build_router(
        AppState::demo()
            .expect("state")
            .with_overpass_api_base_url(format!("{base_url}/api/interpreter"))
            .with_open_meteo_base_url(base_url.clone())
            .with_firms_api_base_url(base_url.clone())
            .with_firms_map_key("test-firms-key")
            .with_opensky_api_base_url(base_url)
            .with_opensky_bearer_token("test-opensky-token"),
    );

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/passive/regions")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "region_id": "andalusia-bootstrap",
                        "name": "Andalusia Bootstrap",
                        "south": 37.30,
                        "west": -6.10,
                        "north": 37.50,
                        "east": -5.80,
                        "site_types": ["SolarPlant", "DataCenter", "Substation"],
                        "timezone": "Europe/Madrid",
                        "country_code": "ES",
                        "default_operator_name": "OSM Passive Intel",
                        "default_criticality": "High",
                        "observation_radius_km": 20.0,
                        "discovery_cadence_seconds": 86400,
                        "scan_limit": 10,
                        "minimum_priority": 0.0
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(create_response.status(), StatusCode::OK);
    let create_body = body_json(create_response).await;
    assert_eq!(create_body["data"]["region_id"], "andalusia-bootstrap");
    assert_eq!(create_body["data"]["enabled"], true);

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/regions")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_body = body_json(list_response).await;
    assert_eq!(
        list_body["data"].as_array().map_or(0, Vec::len),
        1,
        "region target should be persisted"
    );

    let first_run_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/passive/regions/run")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "window_hours": 24,
                        "include_adsb": true,
                        "include_weather": true,
                        "include_fire_smoke": true
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(first_run_response.status(), StatusCode::OK);
    let first_run_body = body_json(first_run_response).await;
    assert_eq!(first_run_body["data"]["evaluated_region_count"], 1);
    assert_eq!(first_run_body["data"]["discovered_region_count"], 1);
    assert_eq!(first_run_body["data"]["discovered_seed_count"], 3);
    assert_eq!(
        first_run_body["data"]["scheduler"]["selected_seed_count"],
        3
    );
    assert!(
        first_run_body["data"]["regions"][0]["region"]["last_discovered_at_unix_seconds"]
            .is_number()
    );
    assert!(
        first_run_body["data"]["regions"][0]["region"]["last_scheduler_run_at_unix_seconds"]
            .is_number()
    );

    let overview_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/regions/andalusia-bootstrap/overview?site_limit=3&days=30")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(overview_response.status(), StatusCode::OK);
    let overview_body = body_json(overview_response).await;
    assert_eq!(
        overview_body["data"]["region"]["region_id"],
        "andalusia-bootstrap"
    );
    assert_eq!(overview_body["data"]["seed_count"], 3);
    assert_eq!(overview_body["data"]["observed_seed_count"], 3);
    assert!(overview_body["data"]["recent_event_count"].is_number());
    assert!(overview_body["data"]["highest_scan_priority"].is_number());
    assert!(!overview_body["data"]["top_sites"]
        .as_array()
        .expect("top sites")
        .is_empty());
    assert!(overview_body["data"]["narrative"]
        .as_str()
        .unwrap_or_default()
        .contains("Andalusia Bootstrap"));
    assert!(overview_body["data"]["provenance"]["region_overview_path"]
        .as_str()
        .unwrap_or_default()
        .contains("/v1/passive/regions/andalusia-bootstrap/overview"));
    assert!(
        overview_body["data"]["provenance"]["operational_timeline_path"]
            .as_str()
            .unwrap_or_default()
            .contains("include_empty_buckets=false")
    );
    assert!(overview_body["data"]["provenance"]["top_site_overview_paths"].is_array());

    let run_logs_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/regions/runs?region_id=andalusia-bootstrap")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(run_logs_response.status(), StatusCode::OK);
    let run_logs_body = body_json(run_logs_response).await;
    assert_eq!(
        run_logs_body["data"].as_array().map_or(0, Vec::len),
        1,
        "regional run should be logged"
    );
    assert_eq!(run_logs_body["data"][0]["status"], "Completed");
    assert_eq!(run_logs_body["data"][0]["evaluated_region_count"], 1);
    assert_eq!(run_logs_body["data"][0]["discovered_seed_count"], 3);
    assert_eq!(run_logs_body["data"][0]["selected_seed_count"], 3);

    let request_count_after_first_run = requests.lock().await.len();
    assert!(request_count_after_first_run > 0);

    let second_run_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/passive/regions/run")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "window_hours": 24,
                        "include_adsb": true,
                        "include_weather": true,
                        "include_fire_smoke": true
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(second_run_response.status(), StatusCode::OK);
    let second_run_body = body_json(second_run_response).await;
    assert_eq!(second_run_body["data"]["evaluated_region_count"], 1);
    assert_eq!(second_run_body["data"]["discovered_region_count"], 0);
    assert!(second_run_body["data"]["scheduler"].is_null());
    assert_eq!(requests.lock().await.len(), request_count_after_first_run);

    let run_logs_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/regions/runs?region_id=andalusia-bootstrap")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(run_logs_response.status(), StatusCode::OK);
    let run_logs_body = body_json(run_logs_response).await;
    assert_eq!(run_logs_body["data"].as_array().map_or(0, Vec::len), 2);
    assert_eq!(
        run_logs_body["data"][0]["status"],
        "CompletedWithNoDueRegions"
    );
    let run_id = run_logs_body["data"][0]["run_id"].as_str().expect("run id");
    let run_log_detail_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/v1/passive/regions/runs/{run_id}"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(run_log_detail_response.status(), StatusCode::OK);
    let run_log_detail_body = body_json(run_log_detail_response).await;
    assert_eq!(run_log_detail_body["data"]["run_id"], run_id);

    let operations_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/operations/status")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(operations_response.status(), StatusCode::OK);
    let operations_body = body_json(operations_response).await;
    assert_eq!(operations_body["data"]["total_region_count"], 1);
    assert_eq!(operations_body["data"]["enabled_region_count"], 1);
    assert!(operations_body["data"]["active_region_lease_count"].is_number());
    assert!(operations_body["data"]["worker_heartbeat_count"].is_number());
    assert!(operations_body["data"]["active_region_leases"].is_array());
    assert!(operations_body["data"]["worker_heartbeats"].is_array());
    assert_eq!(operations_body["data"]["seed_count"], 3);
    assert_eq!(operations_body["data"]["observed_seed_count"], 3);
    assert_eq!(
        operations_body["data"]["latest_region_run"]["status"],
        "CompletedWithNoDueRegions"
    );
    assert!(!operations_body["data"]["recommendation"]
        .as_str()
        .unwrap_or_default()
        .is_empty());
    let leases_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/regions/leases?limit=10")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(leases_response.status(), StatusCode::OK);
    let leases_body = body_json(leases_response).await;
    assert!(leases_body["data"].is_array());
    if let Some(first) = leases_body["data"]
        .as_array()
        .and_then(|items| items.first())
    {
        assert!(first["region_id"].is_string());
        assert!(first["worker_id"].is_string());
        assert!(first["run_id"].is_string());
        assert!(first["acquired_at_unix_seconds"].is_number());
        assert!(first["heartbeat_at_unix_seconds"].is_number());
        assert!(first["expires_at_unix_seconds"].is_number());
    }
    let heartbeats_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/worker/heartbeats?limit=10")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(heartbeats_response.status(), StatusCode::OK);
    let heartbeats_body = body_json(heartbeats_response).await;
    assert!(heartbeats_body["data"].is_array());
    if let Some(first) = heartbeats_body["data"]
        .as_array()
        .and_then(|items| items.first())
    {
        assert!(first["worker_id"].is_string());
        assert!(first["started_at_unix_seconds"].is_number());
        assert!(first["last_heartbeat_unix_seconds"].is_number());
        assert!(first["status"].is_string());
        assert!(first["current_phase"].is_string());
        assert!(first["version"].is_string());
    }

    let map_regions_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/map/regions")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(map_regions_response.status(), StatusCode::OK);
    let map_regions_body = body_json(map_regions_response).await;
    assert_eq!(
        map_regions_body["data"]["regions"]
            .as_array()
            .map_or(0, Vec::len),
        1
    );
    assert!(map_regions_body["data"]["regions"][0]["dominant_status"].is_string());
    assert!(map_regions_body["data"]["regions"][0]["escalating_event_count"].is_number());
    assert!(map_regions_body["data"]["regions"][0]["active_lease_count"].is_number());
    assert!(map_regions_body["data"]["regions"][0]["stale_lease_count"].is_number());
    assert!(map_regions_body["data"]["regions"][0]["active_worker_count"].is_number());
    assert!(map_regions_body["data"]["regions"][0]["stale_worker_count"].is_number());
    assert!(map_regions_body["data"]["regions"][0]["recent_failed_run_count"].is_number());
    assert!(map_regions_body["data"]["regions"][0]["recent_partial_run_count"].is_number());
    assert!(map_regions_body["data"]["regions"][0]["recent_lease_loss_count"].is_number());
    assert!(map_regions_body["data"]["regions"][0]["operational_pressure_priority"].is_string());
    assert!(map_regions_body["data"]["regions"][0]["operational_summary"].is_string());
    assert_eq!(
        map_regions_body["data"]["regions"][0]["region_id"],
        "andalusia-bootstrap"
    );
    assert!(map_regions_body["data"]["regions"][0]["bbox"]["south"].is_number());

    let map_sites_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/map/sites?region_id=andalusia-bootstrap")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(map_sites_response.status(), StatusCode::OK);
    let map_sites_body = body_json(map_sites_response).await;
    assert_eq!(
        map_sites_body["data"]["sites"]
            .as_array()
            .map_or(0, Vec::len),
        3
    );
    assert_eq!(
        map_sites_body["data"]["sites"][0]["region_id"],
        "andalusia-bootstrap"
    );
    assert!(map_sites_body["data"]["sites"][0]["risk_trend"].is_string());
    assert!(map_sites_body["data"]["sites"][0]["top_canonical_status"].is_string());
    assert!(map_sites_body["data"]["sites"][0]["risk_delta_explanation"].is_string());
    assert!(map_sites_body["data"]["sites"][0]["coordinates"]["lat"].is_number());

    let region_timeline_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/regions/andalusia-bootstrap/semantic-timeline?limit=5&window_hours=720")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(region_timeline_response.status(), StatusCode::OK);
    let region_timeline_body = body_json(region_timeline_response).await;
    assert_eq!(region_timeline_body["data"]["scope"], "region");
    assert!(region_timeline_body["data"]["entries"].is_array());
    assert!(region_timeline_body["data"]["entries"][0]["status"].is_string());
    assert!(region_timeline_body["data"]["entries"][0]["temporal_phase"].is_string());
    assert!(region_timeline_body["data"]["entries"][0]["operational_readout"].is_string());
    assert!(region_timeline_body["data"]["entries"][0]["risk_delta"]["classification"].is_string());

    let site_id = map_sites_body["data"]["sites"][0]["site_id"]
        .as_str()
        .expect("site id");
    let site_timeline_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/v1/passive/sites/{site_id}/semantic-timeline?limit=5&window_hours=720"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(site_timeline_response.status(), StatusCode::OK);
    let site_timeline_body = body_json(site_timeline_response).await;
    assert_eq!(site_timeline_body["data"]["scope"], "site");
    assert!(site_timeline_body["data"]["entries"].is_array());
    assert!(site_timeline_body["data"]["entries"][0]["status_summary"].is_string());
    assert!(site_timeline_body["data"]["entries"][0]["temporal_phase"].is_string());
    assert!(site_timeline_body["data"]["entries"][0]["operational_readout"].is_string());

    let map_events_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/map/events?region_id=andalusia-bootstrap&limit=10")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(map_events_response.status(), StatusCode::OK);
    let map_events_body = body_json(map_events_response).await;
    assert!(map_events_body["data"]["events"].is_array());
    assert_eq!(
        map_events_body["data"]["events"][0]["region_id"],
        "andalusia-bootstrap"
    );

    let canonical_events_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/canonical-events?region_id=andalusia-bootstrap&limit=10")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(canonical_events_response.status(), StatusCode::OK);
    let canonical_events_body = body_json(canonical_events_response).await;
    assert!(canonical_events_body["data"]["events"].is_array());
    assert_eq!(
        canonical_events_body["data"]["events"][0]["region_id"],
        "andalusia-bootstrap"
    );
    assert!(canonical_events_body["data"]["compression_ratio"].is_number());
    assert!(canonical_events_body["data"]["raw_event_count"].is_number());
    assert!(canonical_events_body["data"]["events"][0]["bundle_hashes"].is_array());
    assert!(canonical_events_body["data"]["events"][0]["manifest_hashes"].is_array());
    assert!(canonical_events_body["data"]["events"][0]["status"].is_string());
    assert!(canonical_events_body["data"]["events"][0]["status_summary"].is_string());
    assert!(canonical_events_body["data"]["events"][0]["temporal_phase"].is_string());
    assert!(canonical_events_body["data"]["events"][0]["operational_readout"].is_string());
    assert!(canonical_events_body["data"]["events"][0]["risk_delta"]["classification"].is_string());
    assert!(canonical_events_body["data"]["events"][0]["risk_delta"]["explanation"].is_string());
    let canonical_event_id = canonical_events_body["data"]["events"][0]["canonical_event_id"]
        .as_str()
        .expect("canonical event id");
    let canonical_event_detail_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/v1/passive/canonical-events/{canonical_event_id}"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(canonical_event_detail_response.status(), StatusCode::OK);
    let canonical_event_detail_body = body_json(canonical_event_detail_response).await;
    assert_eq!(
        canonical_event_detail_body["data"]["canonical_id"],
        canonical_event_id
    );
    assert!(canonical_event_detail_body["data"]["related_event_ids"].is_array());
    assert!(canonical_event_detail_body["data"]["evidence_bundle_hashes"].is_array());
    assert!(canonical_event_detail_body["data"]["replay_manifest_hashes"].is_array());

    let canonical_map_events_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/map/canonical-events?region_id=andalusia-bootstrap&limit=10")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(canonical_map_events_response.status(), StatusCode::OK);
    let canonical_map_events_body = body_json(canonical_map_events_response).await;
    assert!(canonical_map_events_body["data"]["events"].is_array());

    let dashboard_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/dashboard/summary?limit=3")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(dashboard_response.status(), StatusCode::OK);
    let dashboard_body = body_json(dashboard_response).await;
    assert_eq!(
        dashboard_body["data"]["operations"]["enabled_region_count"],
        1
    );
    assert_eq!(dashboard_body["data"]["worker_health"]["status"], "healthy");
    assert!(dashboard_body["data"]["source_health"].is_array());
    assert!(dashboard_body["data"]["source_health"][0]["source"].is_string());
    assert!(dashboard_body["data"]["source_health"][0]["region_id"].is_null());
    assert!(dashboard_body["data"]["source_health"][0]["reliability_score"].is_number());
    assert!(dashboard_body["data"]["worker_health"]["active_region_lease_count"].is_number());
    assert!(dashboard_body["data"]["worker_health"]["worker_heartbeat_count"].is_number());
    assert!(dashboard_body["data"]["worker_health"]["active_worker_count"].is_number());
    assert!(dashboard_body["data"]["worker_health"]["running_region_ids"].is_array());
    assert!(dashboard_body["data"]["operations"]["active_region_leases"].is_array());
    assert!(dashboard_body["data"]["operations"]["worker_heartbeats"].is_array());
    assert!(dashboard_body["data"]["source_health"]
        .as_array()
        .expect("source health")
        .iter()
        .any(|entry| entry["source"].is_string()
            && entry["success_count"].is_number()
            && entry["health_status"].is_string()
            && entry["reliability_score"].is_number()
            && entry["consecutive_failure_count"].is_number()
            && entry["recovery_hint"].is_string()));

    let regional_dashboard_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/dashboard/summary?limit=3&region_id=andalusia-bootstrap")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(regional_dashboard_response.status(), StatusCode::OK);
    let regional_dashboard_body = body_json(regional_dashboard_response).await;
    assert_eq!(
        regional_dashboard_body["data"]["region_id"],
        "andalusia-bootstrap"
    );
    assert!(regional_dashboard_body["data"]["source_health"]
        .as_array()
        .expect("regional source health")
        .iter()
        .all(|entry| entry["region_id"] == "andalusia-bootstrap"));
    assert!(dashboard_body["data"]["top_regions"].is_array());
    assert!(dashboard_body["data"]["top_regions"][0]["operational_pressure_priority"].is_string());
    assert!(dashboard_body["data"]["top_regions"][0]["recent_failed_run_count"].is_number());
    assert!(dashboard_body["data"]["top_regions"][0]["recent_lease_loss_count"].is_number());
    assert!(dashboard_body["data"]["top_sites"].is_array());
    assert!(dashboard_body["data"]["top_events"].is_array());
    assert!(dashboard_body["data"]["top_canonical_events"].is_array());
    assert!(dashboard_body["data"]["semantic_timeline"].is_array());
    assert!(dashboard_body["data"]["semantic_timeline"][0]["status"].is_string());
    assert!(dashboard_body["data"]["semantic_timeline"][0]["temporal_phase"].is_string());
    assert!(dashboard_body["data"]["semantic_timeline"][0]["operational_readout"].is_string());
    assert!(
        dashboard_body["data"]["semantic_timeline"][0]["risk_delta"]["classification"].is_string()
    );
    assert!(dashboard_body["data"]["top_sites"][0]["risk_trend"].is_string());
    assert!(dashboard_body["data"]["top_sites"][0]["top_canonical_status"].is_string());
    assert!(!dashboard_body["data"]["narrative"]
        .as_str()
        .unwrap_or_default()
        .is_empty());

    let operational_visibility_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/operational-visibility?limit=10")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(operational_visibility_response.status(), StatusCode::OK);
    let operational_visibility_body = body_json(operational_visibility_response).await;
    assert!(operational_visibility_body["data"]["overall_state"].is_string());
    assert!(operational_visibility_body["data"]["regions"].is_array());
    assert!(operational_visibility_body["data"]["regions"][0]["state"].is_string());
    assert!(operational_visibility_body["data"]["regions"][0]["drivers"]
        ["recent_failed_run_count"]
        .is_number());
    assert!(
        operational_visibility_body["data"]["regions"][0]["drivers"]["active_lease_count"]
            .is_number()
    );
    assert!(
        operational_visibility_body["data"]["regions"][0]["drivers"]["active_worker_count"]
            .is_number()
    );
    assert!(operational_visibility_body["data"]["worker_heartbeats"].is_array());
    assert!(operational_visibility_body["data"]["active_region_leases"].is_array());
    assert!(operational_visibility_body["data"]["stale_region_leases"].is_array());
    assert!(operational_visibility_body["data"]["panel_paths"].is_array());
    assert!(operational_visibility_body["data"]["map_overlay_paths"].is_array());

    let command_center_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/command-center/summary?limit=3")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(command_center_response.status(), StatusCode::OK);
    let command_center_body = body_json(command_center_response).await;
    assert!(command_center_body["data"]["summary"].is_string());
    assert!(command_center_body["data"]["highlights"]["top_region"]["region_id"].is_string());
    assert!(command_center_body["data"]["highlights"]["top_site"]["site_id"].is_string());
    assert!(
        command_center_body["data"]["highlights"]["top_event"]["canonical_event_id"].is_string()
    );
    assert!(command_center_body["data"]["highlights"]["top_event"]["temporal_phase"].is_string());
    assert!(
        command_center_body["data"]["highlights"]["top_event"]["operational_readout"].is_string()
    );
    assert!(command_center_body["data"]["operator_paths"].is_array());
    assert!(command_center_body["data"]["focus_paths"].is_array());
    let attention_queue = command_center_body["data"]["attention_queue"]
        .as_array()
        .expect("attention queue");
    assert!(
        !attention_queue.is_empty(),
        "command center should expose operational attention items"
    );
    for item in attention_queue {
        assert_attention_item_shape(item);
        if matches!(item["kind"].as_str(), Some("canonical_event" | "site")) {
            assert!(
                !item["region_id"].as_str().unwrap_or_default().is_empty(),
                "mappable attention items should include region_id: {item:?}"
            );
        }
    }
    assert!(command_center_body["data"]["dashboard"]["top_regions"].is_array());
    assert!(command_center_body["data"]["maintenance"]["suggested_actions"].is_array());
    assert!(
        command_center_body["data"]["maintenance"]["source_health_prune_candidate_count"]
            .is_number()
    );
    assert!(command_center_body["data"]["operator_paths"]
        .as_array()
        .expect("operator paths")
        .iter()
        .any(|path| path
            .as_str()
            .unwrap_or_default()
            .contains("/v1/passive/operational-visibility")));
    assert!(command_center_body["data"]["operator_paths"]
        .as_array()
        .expect("operator paths")
        .iter()
        .any(|path| path
            .as_str()
            .unwrap_or_default()
            .contains("/v1/passive/maintenance/summary")));

    let filtered_attention_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/command-center/summary?limit=3&attention_kind=canonical_event")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(filtered_attention_response.status(), StatusCode::OK);
    let filtered_attention_body = body_json(filtered_attention_response).await;
    let filtered_attention_queue = filtered_attention_body["data"]["attention_queue"]
        .as_array()
        .expect("filtered attention queue");
    assert!(
        !filtered_attention_queue.is_empty(),
        "canonical event attention filter should retain fixture events"
    );
    assert!(filtered_attention_queue
        .iter()
        .all(|item| item["kind"] == "canonical_event"));
    for item in filtered_attention_queue {
        assert_attention_item_shape(item);
    }

    let priority_attention_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/passive/command-center/summary?limit=10&min_attention_priority=medium")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(priority_attention_response.status(), StatusCode::OK);
    let priority_attention_body = body_json(priority_attention_response).await;
    let priority_attention_queue = priority_attention_body["data"]["attention_queue"]
        .as_array()
        .expect("priority attention queue");
    assert!(
        !priority_attention_queue.is_empty(),
        "fixture should include medium-or-higher attention items"
    );
    assert!(priority_attention_queue
        .iter()
        .all(|item| attention_priority_rank(item["priority"].as_str().expect("priority")) >= 1));

    let pressure_command_center_response = app
        .oneshot(
            Request::builder()
                .uri("/v1/passive/command-center/summary?limit=3&min_pressure_priority=low")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(pressure_command_center_response.status(), StatusCode::OK);
    let pressure_command_center_body = body_json(pressure_command_center_response).await;
    assert!(pressure_command_center_body["data"]["dashboard"]["top_regions"].is_array());
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn operator_console_is_served() {
    let response = build_router(AppState::demo().expect("state"))
        .oneshot(
            Request::builder()
                .uri("/console")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    let html = String::from_utf8(bytes.to_vec()).expect("utf8");
    assert!(html.contains("SSS Operator Console"));
    assert!(html.contains("/v1/events/timeline"));
    assert!(html.contains("/v1/events/dispatch"));
    assert!(html.contains("/v1/events?limit=8"));
    assert!(html.contains("/v1/briefing/apod"));
    assert!(html.contains("/v1/briefing/neows"));
    assert!(html.contains("Show NEO context"));
    assert!(html.contains("Show close approaches"));
    assert!(html.contains("toggle-neo-layer"));
    assert!(html.contains("toggle-close-approach-layer"));
    assert!(html.contains("NEO Risk Briefing"));
    assert!(html.contains("neows-briefing"));
    assert!(html.contains("refreshNeoWsBriefing"));
    assert!(html.contains("state.neoBriefing = data;"));
    assert!(html.contains("item.priority_score"));
    assert!(html.contains("item.close_approach_date ?? \"date n/a\""));
    assert!(html.contains("item.miss_distance_km"));
    assert!(html.contains("item.relative_velocity_km_s"));
    assert!(html.contains("item.briefing_summary"));
    assert!(html.contains("Dispatch Event"));
    assert!(html.contains("Event Queue"));
    assert!(html.contains("Passive Operations"));
    assert!(html.contains("Source Health"));
    assert!(html.contains("health_status"));
    assert!(html.contains("staleness_seconds"));
    assert!(html.contains("consecutive_failure_count"));
    assert!(html.contains("recovery_hint"));
    assert!(html.contains("Top Regions"));
    assert!(html.contains("source_health"));
    assert!(html.contains("Active Leases"));
    assert!(html.contains("Worker Heartbeats"));
    assert!(html.contains("/v1/passive/command-center/summary"));
    assert!(html.contains("/v1/passive/map/canonical-events"));
    assert!(html.contains("Passive Focus"));
    assert!(html.contains("passive-focus"));
    assert!(html.contains("support_count"));
    assert!(html.contains("status_summary"));
    assert!(html.contains("risk_delta.explanation"));
    assert!(html.contains("top_canonical_status"));
    assert!(html.contains("dominant_status"));
    assert!(html.contains("operational_summary"));
    assert!(html.contains("passivePressureFilter"));
    assert!(html.contains("passive-pressure-filter"));
    assert!(html.contains("min_pressure_priority"));
    assert!(
        html.contains("/v1/passive/regions/${encodeURIComponent(payload.region_id)}/remediation")
    );
    assert!(html.contains("suggested_read_paths"));
    assert!(html.contains("passive-open-region-runs"));
    assert!(html.contains("passive-open-region-run"));
    assert!(html.contains("passive-open-region-overview"));
    assert!(html.contains("data-region-id"));
    assert!(html.contains("region_id=${encodeURIComponent(regionId)}"));
    assert!(html.contains(
        "/operational-timeline?window_hours=72&bucket_hours=6&include_empty_buckets=false"
    ));
    assert!(html.contains("Operational Timeline"));
    assert!(html.contains("selectedPassiveOperationalTimeline"));
    assert!(html.contains("ops trend"));
    assert!(html.contains("/v1/passive/source-health/samples/prune"));
    assert!(html.contains("Maintenance"));
    assert!(html.contains("Command Context"));
    assert!(html.contains("renderPassiveCommandContext"));
    assert!(html.contains("passiveMapSummary"));
    assert!(html.contains("passiveCommandCenterSummary"));
    assert!(html.contains("top_region"));
    assert!(html.contains("top_site"));
    assert!(html.contains("top_event"));
    assert!(html.contains("operational_readout"));
    assert!(html.contains("operator_paths"));
    assert!(html.contains("focus_paths"));
    assert!(html.contains("Operator Paths"));
    assert!(html.contains("Focus Paths"));
    assert!(html.contains("Attention Queue"));
    assert!(html.contains("renderPassiveAttentionQueue"));
    assert!(html.contains("attention_queue"));
    assert!(html.contains("passiveAttentionKindFilter"));
    assert!(html.contains("passiveAttentionPriorityFilter"));
    assert!(html.contains("attention_kind"));
    assert!(html.contains("min_attention_priority"));
    assert!(html.contains("passive-attention-kind-filter"));
    assert!(html.contains("passive-attention-priority-filter"));
    assert!(html.contains("passive-reset-attention-kind-filter"));
    assert!(html.contains("passive-reset-attention-priority-filter"));
    assert!(html.contains("Show All Attention"));
    assert!(html.contains("Show All Priorities"));
    assert!(html.contains("primary_action_label"));
    assert!(html.contains("passiveAttentionMapMode"));
    assert!(html.contains("passiveSelectedAttention"));
    assert!(html.contains("passive-toggle-attention-map"));
    assert!(html.contains("passiveAttentionSets"));
    assert!(html.contains("attentionModeAppliesToSite"));
    assert!(html.contains("attentionModeAppliesToEvent"));
    assert!(html.contains("focusPassiveSiteFromAttention"));
    assert!(html.contains("focusPassiveCanonicalFromAttention"));
    assert!(html.contains("focusPassiveAttentionItem"));
    assert!(html.contains("setPassiveSelectedAttentionFromDataset"));
    assert!(html.contains("selectedPassiveAttentionItem"));
    assert!(html.contains("renderPassiveAttentionPrimaryAction"));
    assert!(html.contains("clearPassiveSelectedAttention"));
    assert!(html.contains("handlePassiveRefocusCanonicalButton"));
    assert!(html.contains("handlePassiveRefocusSiteButton"));
    assert!(html.contains("handlePassiveOpenPathButton"));
    assert!(html.contains("handlePassiveOpenEvidenceButton"));
    assert!(html.contains("handlePassiveRunReplayButton"));
    assert!(html.contains("handlePassiveOpenRegionOverviewButton"));
    assert!(html.contains("renderPassiveSelectedAttentionContext"));
    assert!(html.contains("Selected Attention"));
    assert!(html.contains("Clear Selection"));
    assert!(html.contains("Focus Event"));
    assert!(html.contains("Focus Site"));
    assert!(html.contains("Open Region"));
    assert!(html.contains("Region Overview"));
    assert!(html.contains("Diagnostics"));
    assert!(html.contains("Stale Heartbeats"));
    assert!(html.contains("passive-open-worker-diagnostics"));
    assert!(html.contains("passive-open-heartbeat-list"));
    assert!(html.contains("passive-open-stale-heartbeats"));
    assert!(html.contains("passive-selected-attention-actions"));
    assert!(html.contains("passive-selected-attention-read-actions"));
    assert!(html.contains("passive-selected-attention-context"));
    assert!(html.contains("passive-clear-selected-attention"));
    assert!(html.contains("openPassiveWorkerDiagnostics"));
    assert!(html.contains("openPassiveHeartbeatList"));
    assert!(html.contains("passive-attention-item-selected"));
    assert!(html.contains("readPathButtons"));
    assert!(html.contains("readPathLabel"));
    assert!(html.contains("openPassivePath"));
    assert!(html.contains("passive-open-path"));
    assert!(html.contains("passive-attention-item-card"));
    assert!(html.contains("data-region-id=\"${escapeHtml(item.region_id ?? \"\")}\""));
    assert!(html.contains("Attention Map ${attentionMapActive}"));
    assert!(html.contains("attention map on"));
    assert!(html.contains("attention map off"));
    assert!(html.contains("Attention map:"));
    assert!(html.contains("No operational attention items."));
    assert!(html.contains("openPassiveRegionOverview"));
    assert!(html.contains("passive-reset-semantic-filter"));
    assert!(html.contains("passive-reset-pressure-filter"));
    assert!(html.contains("passive-refocus-canonical"));
    assert!(html.contains("passive-refocus-site"));
    assert!(html.contains("Show All Semantics"));
    assert!(html.contains("Show All Pressure"));
    assert!(html.contains("Focus Selected Event"));
    assert!(html.contains("Focus Selected Site"));
    assert!(html.contains("Focus Top Event"));
    assert!(html.contains("Focus Top Site"));
    assert!(html.contains("Open Top Region"));
    assert!(html.contains("source_health_prune_candidate_count"));
    assert!(html.contains("confirmation_read_paths"));
    assert!(html.contains("Preview Global Source Prune"));
    assert!(html.contains("passive-prune-source-health"));
    assert!(html.contains("passive-preview-source-prune"));
    assert!(html.contains("Source health prune preview"));
    assert!(html.contains("Narrative Provenance"));
    assert!(html.contains("renderNarrativeProvenance"));
    assert!(html.contains("Region Provenance"));
    assert!(html.contains("renderRegionProvenance"));
    assert!(html.contains("/semantic-timeline?limit=6&window_hours=720"));
    assert!(html.contains("Semantic Timeline"));
    assert!(html.contains("Semantic Strip"));
    assert!(html.contains("passive-semantic-filter"));
    assert!(html.contains("passive-strip-entry"));
    assert!(html.contains("passive-map-legend"));
    assert!(html.contains("clearPassiveMapEntities"));
    assert!(html.contains("highlighted sites"));
    assert!(html.contains("applyPassiveSiteFocus"));
    assert!(html.contains("focusPassiveCanonicalEvent"));
    assert!(html.contains("focusPassiveSite"));
    assert!(html.contains("openPassiveEvidence"));
    assert!(html.contains("runPassiveReplay"));
    assert!(html.contains("passiveSiteEntityIndex"));
    assert!(html.contains("passive-focus-link-"));
    assert!(html.contains("selectedPassiveCanonicalEventId"));
    assert!(html.contains("passive-timeline-entry"));
    assert!(html.contains("Timeline focus"));
    assert!(html.contains("Focused Chain Actions"));
    assert!(html.contains("passive-go-map"));
    assert!(html.contains("passive-open-evidence"));
    assert!(html.contains("passive-run-replay"));
    assert!(html.contains("state.passiveEntityIndex"));
    assert!(html.contains("Focused canonical event"));
    assert!(html.contains("cesium-globe"));
    assert!(html.contains("Cesium.js"));
    // FRENTE 3 — RiskDelta + canonical events panel
    assert!(html.contains("riskDeltaClass"));
    assert!(html.contains("renderRiskDeltaPill"));
    assert!(html.contains("renderPassiveCanonicalEvents"));
    assert!(html.contains("refreshPassiveCanonicalEvents"));
    assert!(html.contains("canonical-events"));
    assert!(html.contains("refresh-canonical-events"));
    assert!(html.contains("Canonical Events"));
}

#[tokio::test]
async fn passive_canonical_events_filter_by_region_is_consistent() {
    let app = build_router(AppState::demo().expect("state"));

    // Create a region target so we have a valid region_id to filter on.
    let region_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/passive/regions")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Iberian Peninsula",
                        "south": 35.0,
                        "west": -10.0,
                        "north": 44.0,
                        "east": 5.0,
                        "country_code": "ES",
                        "timezone": "Europe/Madrid"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(region_response.status(), StatusCode::OK);
    let region_body = body_json(region_response).await;
    let region_id = region_body["data"]["region_id"]
        .as_str()
        .expect("region_id in creation response");

    // Query canonical events filtered by this region_id.
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/v1/passive/canonical-events?region_id={region_id}&limit=25"
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    let events = body["data"]["events"].as_array().expect("events array");
    // Every returned event must match the requested region_id (trivially true when
    // the demo DB has no canonical events yet, but will catch regressions once data exists).
    assert!(
        events
            .iter()
            .all(|event| event["region_id"].as_str().unwrap_or_default() == region_id),
        "every canonical event should match region filter"
    );
}

#[tokio::test]
async fn operator_console_wires_canonical_focus_and_region_runs_actions() {
    let response = build_router(AppState::demo().expect("state"))
        .oneshot(
            Request::builder()
                .uri("/console")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    let html = String::from_utf8(bytes.to_vec()).expect("utf8");

    assert!(
        html.contains("$(\"canonical-events\").addEventListener(\"click\"")
    );
    assert!(html.contains("handlePassiveRefocusCanonicalButton"));
    assert!(html.contains("openPassiveRegionRuns(regionRunsButton.dataset.regionId"));
    assert!(html.contains("data-canonical-id"));
    assert!(html.contains("data-region-id"));
}

#[tokio::test]
async fn operator_console_guards_risk_delta_explanation_and_refresh_sync() {
    let response = build_router(AppState::demo().expect("state"))
        .oneshot(
            Request::builder()
                .uri("/console")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    let html = String::from_utf8(bytes.to_vec()).expect("utf8");

    // Empty/partial explanation guard for canonical cards.
    assert!(html.contains("event.risk_delta?.explanation ?"));
    // Timeline readability under volume is intentionally capped.
    assert!(html.contains("entries.slice(0, 10)"));
    // refreshAll keeps canonical panel in the same refresh batch.
    assert!(html.contains("refreshPassiveDashboardSummary(),"));
    assert!(html.contains("refreshPassiveOperationalVisibility(),"));
    assert!(html.contains("refreshPassiveCanonicalEvents(),"));
}

#[tokio::test]
async fn operator_can_fetch_native_close_approaches() {
    let app = build_router(close_approach_demo_state());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/objects/SAT-PRIMARY/close-approaches?horizon=24&threshold_km=250&limit=5")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["data"]["object_id"], "SAT-PRIMARY");
    assert_eq!(
        body["data"]["close_approaches"][0]["counterpart_object_id"],
        "SAT-NEAR"
    );
    assert!(
        body["data"]["close_approaches"][0]["miss_distance_km"]
            .as_f64()
            .unwrap_or(999.0)
            <= 250.0
    );
}

#[tokio::test]
async fn space_overview_persists_native_close_approach_events() {
    let app = build_router(close_approach_demo_state());
    let overview_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/space-overview")
                .header("content-type", "application/json")
                .body(Body::from(json!({"window_hours": 24.0}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(overview_response.status(), StatusCode::OK);
    let overview_body = body_json(overview_response).await;
    assert!(overview_body["data"]["top_events"]
        .as_array()
        .expect("top_events")
        .iter()
        .any(|event| {
            event["event_type"] == "PredictedCloseApproach"
                && event["event_id"]
                    .as_str()
                    .unwrap_or_default()
                    .contains("native_close_approach")
        }));

    let events_response = app
        .oneshot(
            Request::builder()
                .uri("/v1/events?limit=10&event_type=PredictedCloseApproach")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(events_response.status(), StatusCode::OK);
    let events_body = body_json(events_response).await;
    assert!(events_body["data"]
        .as_array()
        .expect("events")
        .iter()
        .any(|event| {
            event["event_id"]
                .as_str()
                .unwrap_or_default()
                .contains("native_close_approach")
        }));
}

#[tokio::test]
async fn analyze_object_exposes_close_approach_notification_policy() {
    let app = build_router(close_approach_demo_state());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/analyze-object")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"object_id": "SAT-PRIMARY", "timestamp_unix_seconds": 1_800_000_060})
                        .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["data"]["prediction"]["event"], "CloseApproach");
    assert!(
        body["data"]["decision"]["recipient_policy"]["escalation_rules"]
            .as_array()
            .expect("rules")
            .iter()
            .any(|rule| rule["rule_id"] == "predicted_close_approach_to_coordination")
    );
    assert!(body["data"]["decision"]["notifications"]
        .as_array()
        .expect("notifications")
        .iter()
        .any(|notification| {
            notification["recipient"] == "SpaceTrafficCoordination"
                && notification["notification_reason"]
                    .as_str()
                    .unwrap_or_default()
                    .contains("projected within")
        }));
}

#[tokio::test]
async fn ranked_event_can_be_dispatched_without_manual_analyze_request() {
    let received = Arc::new(Mutex::new(Vec::<Value>::new()));
    let webhook_url = spawn_webhook_server(received.clone(), false).await;
    let app = build_router(
        close_approach_demo_state()
            .with_webhook_endpoint(webhook_url)
            .with_webhook_delivery_policy(WebhookDeliveryPolicy {
                max_attempts: 2,
                retry_delay: std::time::Duration::from_millis(5),
            }),
    );

    let overview_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/space-overview")
                .header("content-type", "application/json")
                .body(Body::from(json!({"window_hours": 24.0}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(overview_response.status(), StatusCode::OK);
    let overview_body = body_json(overview_response).await;
    let event_id = overview_body["data"]["top_events"]
        .as_array()
        .expect("top events")
        .iter()
        .find_map(|event| {
            (event["event_type"] == "PredictedCloseApproach"
                && event["event_id"]
                    .as_str()
                    .unwrap_or_default()
                    .contains("native_close_approach"))
            .then(|| event["event_id"].as_str().unwrap_or_default().to_string())
        })
        .expect("native close approach event id");

    let dispatch_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events/dispatch")
                .header("content-type", "application/json")
                .header("x-request-id", "event-dispatch-test")
                .body(Body::from(json!({ "event_id": event_id }).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(dispatch_response.status(), StatusCode::OK);
    let dispatch_body = body_json(dispatch_response).await;
    assert_eq!(
        dispatch_body["data"]["dispatch"]["analysis"]["prediction"]["event"],
        "CloseApproach"
    );
    assert_eq!(
        dispatch_body["data"]["dispatch"]["deliveries"][0]["request_id"],
        "event-dispatch-test"
    );
    assert!(received.lock().await.len() >= 3);
}

#[tokio::test]
async fn operator_can_fetch_apod_briefing() {
    let requests = Arc::new(Mutex::new(Vec::<String>::new()));
    let base_url = spawn_nasa_apod_server(requests.clone()).await;
    let response = build_router(
        AppState::demo()
            .expect("state")
            .with_nasa_api_base_url(base_url)
            .with_nasa_api_key("test-nasa-key"),
    )
    .oneshot(
        Request::builder()
            .uri("/v1/briefing/apod?date=2026-04-18")
            .body(Body::empty())
            .expect("request"),
    )
    .await
    .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["data"]["title"], "Astronomy Picture Test");
    assert_eq!(body["data"]["date"], "2026-04-18");
    assert_eq!(body["data"]["media_type"], "image");
    assert_eq!(
        requests.lock().await[0],
        "/planetary/apod?api_key=test-nasa-key&date=2026-04-18"
    );
}

#[tokio::test]
async fn operator_can_fetch_neows_briefing() {
    let requests = Arc::new(Mutex::new(Vec::<String>::new()));
    let base_url = spawn_nasa_neows_server(requests.clone()).await;
    let response = build_router(
        AppState::demo()
            .expect("state")
            .with_nasa_api_base_url(base_url)
            .with_nasa_api_key("test-nasa-key"),
    )
    .oneshot(
        Request::builder()
            .uri("/v1/briefing/neows?start_date=2026-04-18&end_date=2026-04-18")
            .body(Body::empty())
            .expect("request"),
    )
    .await
    .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["data"]["start_date"], "2026-04-18");
    assert_eq!(body["data"]["end_date"], "2026-04-18");
    assert_eq!(body["data"]["total_objects"], 2);
    assert_eq!(body["data"]["hazardous_objects"], 1);
    let highest_priority = body["data"]["highest_priority"]
        .as_array()
        .expect("highest priority");
    assert_eq!(highest_priority.len(), 2);
    assert_eq!(highest_priority[0]["neo_reference_id"].as_str(), Some("1"));
    assert_eq!(highest_priority[0]["name"].as_str(), Some("(2026 AB)"));
    assert_eq!(
        highest_priority[0]["nasa_jpl_url"].as_str(),
        Some("https://ssd.jpl.nasa.gov/tools/sbdb_lookup.html#/?sstr=1")
    );
    assert_eq!(highest_priority[0]["hazardous"].as_bool(), Some(true));
    assert_eq!(
        highest_priority[0]["close_approach_date"].as_str(),
        Some("2026-04-18")
    );
    assert_eq!(
        highest_priority[0]["miss_distance_km"].as_f64(),
        Some(450_000.0)
    );
    assert_eq!(
        highest_priority[0]["relative_velocity_km_s"].as_f64(),
        Some(12.5)
    );
    assert_eq!(
        highest_priority[0]["estimated_diameter_min_m"].as_f64(),
        Some(120.0)
    );
    assert_eq!(
        highest_priority[0]["estimated_diameter_max_m"].as_f64(),
        Some(260.0)
    );
    assert!(highest_priority[0]["priority_score"].is_number());
    assert_eq!(
        highest_priority[0]["briefing_summary"].as_str(),
        Some("Potentially hazardous NEO with close approach inside selected window on 2026-04-18.")
    );
    assert_eq!(highest_priority[1]["neo_reference_id"].as_str(), Some("2"));
    assert_eq!(highest_priority[1]["name"].as_str(), Some("(2026 CD)"));
    assert!(highest_priority[1]["nasa_jpl_url"].is_null());
    assert_eq!(highest_priority[1]["hazardous"].as_bool(), Some(false));
    assert_eq!(
        highest_priority[1]["close_approach_date"].as_str(),
        Some("2026-04-18")
    );
    assert_eq!(
        highest_priority[1]["miss_distance_km"].as_f64(),
        Some(1_800_000.0)
    );
    assert_eq!(
        highest_priority[1]["relative_velocity_km_s"].as_f64(),
        Some(8.2)
    );
    assert_eq!(
        highest_priority[1]["estimated_diameter_min_m"].as_f64(),
        Some(40.0)
    );
    assert_eq!(
        highest_priority[1]["estimated_diameter_max_m"].as_f64(),
        Some(90.0)
    );
    assert!(highest_priority[1]["priority_score"].is_number());
    assert_eq!(
        highest_priority[1]["briefing_summary"].as_str(),
        Some("Non-hazardous NEO with relevant close approach on 2026-04-18.")
    );
    assert_eq!(
        requests.lock().await[0],
        "/neo/rest/v1/feed?api_key=test-nasa-key&start_date=2026-04-18&end_date=2026-04-18"
    );
}

#[tokio::test]
async fn operator_console_can_fetch_default_neows_briefing_window() {
    let requests = Arc::new(Mutex::new(Vec::<String>::new()));
    let base_url = spawn_nasa_neows_server(requests.clone()).await;
    let response = build_router(
        AppState::demo()
            .expect("state")
            .with_nasa_api_base_url(base_url)
            .with_nasa_api_key("test-nasa-key"),
    )
    .oneshot(
        Request::builder()
            .uri("/v1/briefing/neows")
            .body(Body::empty())
            .expect("request"),
    )
    .await
    .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["data"]["start_date"], "2026-04-18");
    assert_eq!(body["data"]["end_date"], "2026-04-18");
    assert_eq!(body["data"]["total_objects"], 2);
    assert_eq!(body["data"]["hazardous_objects"], 1);
    let highest_priority = body["data"]["highest_priority"]
        .as_array()
        .expect("highest priority");
    assert_eq!(highest_priority.len(), 2);
    assert_eq!(highest_priority[0]["neo_reference_id"].as_str(), Some("1"));
    assert_eq!(
        highest_priority[0]["briefing_summary"].as_str(),
        Some("Potentially hazardous NEO with close approach inside selected window on 2026-04-18.")
    );
    assert_eq!(
        requests.lock().await[0],
        "/neo/rest/v1/feed?api_key=test-nasa-key&start_date=2026-04-18&end_date=2026-04-18"
    );
}

#[tokio::test]
async fn neows_invalid_date_range_returns_invalid_request() {
    let response = build_router(AppState::demo().expect("state"))
        .oneshot(
            Request::builder()
                .uri("/v1/briefing/neows?start_date=2026-04-19&end_date=2026-04-18")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = body_json(response).await;
    assert_eq!(body["error"]["code"], "invalid_request");
}

#[tokio::test]
async fn neows_ranking_prefers_large_hazardous_object() {
    let requests = Arc::new(Mutex::new(Vec::<String>::new()));
    let base_url = spawn_nasa_neows_server(requests).await;
    let response = build_router(
        AppState::demo()
            .expect("state")
            .with_nasa_api_base_url(base_url)
            .with_nasa_api_key("test-nasa-key"),
    )
    .oneshot(
        Request::builder()
            .uri("/v1/briefing/neows/feed?start_date=2026-04-18&end_date=2026-04-18")
            .body(Body::empty())
            .expect("request"),
    )
    .await
    .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["data"]["objects"][0]["name"], "(2026 AB)");
    assert!(
        body["data"]["objects"][0]["priority_score"]
            .as_f64()
            .unwrap_or_default()
            > body["data"]["objects"][1]["priority_score"]
                .as_f64()
                .unwrap_or_default()
    );
}

#[tokio::test]
async fn analyze_object_returns_auditable_bundle_and_replay_manifest_can_be_fetched() {
    let app = build_router(AppState::demo().expect("state"));
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/analyze-object")
                .header("content-type", "application/json")
                .header("x-request-id", "contract-test")
                .body(Body::from(
                    json!({"object_id": "SAT-001", "timestamp_unix_seconds": 1_800_000_060})
                        .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["request_id"], "contract-test");
    assert_eq!(body["data"]["status"], "AnomalyDetected");
    let bundle_hash = body["data"]["evidence_bundle"]["bundle_hash"]
        .as_str()
        .expect("bundle hash");
    assert!(!bundle_hash.is_empty());

    let bundle_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/v1/evidence/{bundle_hash}"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(bundle_response.status(), StatusCode::OK);
    let bundle_body = body_json(bundle_response).await;
    let manifest_hash = replay_manifest_hash(&bundle_body["data"]);

    let bundle_manifest_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/v1/evidence/{bundle_hash}/replay"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(bundle_manifest_response.status(), StatusCode::OK);
    let bundle_manifest_body = body_json(bundle_manifest_response).await;
    assert_eq!(bundle_manifest_body["data"]["manifest_hash"], manifest_hash);

    let replay_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/v1/replay/{manifest_hash}"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(replay_response.status(), StatusCode::OK);

    let replay_execution_response = app
        .oneshot(
            Request::builder()
                .uri(format!("/v1/replay/{manifest_hash}/execute"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(replay_execution_response.status(), StatusCode::OK);
    let replay_execution_body = body_json(replay_execution_response).await;
    assert_eq!(
        replay_execution_body["data"]["manifest"]["manifest_hash"],
        manifest_hash
    );
    assert_eq!(
        replay_execution_body["data"]["replayed_assessment"]["behavior"],
        replay_execution_body["data"]["original_assessment"]["behavior"]
    );
    assert!(replay_execution_body["data"]["replayed_decision"]["recommendation"].is_string());
    assert!(replay_execution_body["data"]["diff"]["assessment"]["confidence_delta"].is_number());
    assert!(replay_execution_body["data"]["diff"]["decision"]["risk_score_delta"].is_number());
}

#[tokio::test]
async fn tle_ingest_feeds_analyze_object_and_persisted_evidence() {
    let app = build_router(AppState::demo().expect("state"));
    let ingest_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest/tle")
                .header("content-type", "application/json")
                .header("x-request-id", "tle-ingest-test")
                .body(Body::from(
                    json!({"source": "celestrak", "payload": CELESTRAK_ISS_SNAPSHOT}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(ingest_response.status(), StatusCode::OK);
    let ingest_body = body_json(ingest_response).await;
    assert_eq!(ingest_body["request_id"], "tle-ingest-test");
    assert_eq!(ingest_body["data"]["source"], "celestrak");
    assert!(ingest_body["data"]["payload_bytes"].as_u64().unwrap_or(0) > 0);
    assert_eq!(ingest_body["data"]["skipped_duplicate"], false);
    assert_eq!(ingest_body["data"]["freshness_seconds"], 0);
    assert_eq!(ingest_body["data"]["records_received"], 1);
    assert_eq!(ingest_body["data"]["observations_created"], 1);
    assert_eq!(ingest_body["data"]["object_ids"][0], "25544");

    let analyze_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/analyze-object")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"object_id": "25544", "timestamp_unix_seconds": null}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(analyze_response.status(), StatusCode::OK);
    let analyze_body = body_json(analyze_response).await;
    assert_eq!(analyze_body["data"]["object_id"], "25544");
    assert_eq!(
        analyze_body["data"]["evidence_bundle"]["sources"][0],
        "Catalog"
    );
    let bundle_hash = analyze_body["data"]["evidence_bundle"]["bundle_hash"]
        .as_str()
        .expect("bundle hash");

    let evidence_response = app
        .oneshot(
            Request::builder()
                .uri(format!("/v1/evidence/{bundle_hash}"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(evidence_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn operator_can_fetch_ranked_events_and_assessment_history() {
    let app = build_router(AppState::demo().expect("state"));
    let analyze_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/analyze-object")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"object_id": "SAT-001", "timestamp_unix_seconds": 1_800_000_060})
                        .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(analyze_response.status(), StatusCode::OK);

    let overview_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/space-overview")
                .header("content-type", "application/json")
                .body(Body::from(json!({"window_hours": 24.0}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(overview_response.status(), StatusCode::OK);

    let events_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/events?limit=1")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(events_response.status(), StatusCode::OK);
    let events_body = body_json(events_response).await;
    assert_eq!(events_body["data"][0]["object_id"], "SAT-001");
    assert!(events_body["data"][0]["target_epoch_unix_seconds"].is_number());

    let assessments_response = app
        .oneshot(
            Request::builder()
                .uri("/v1/objects/SAT-001/assessments")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(assessments_response.status(), StatusCode::OK);
    let assessments_body = body_json(assessments_response).await;
    assert_eq!(assessments_body["data"][0]["status"], "AnomalyDetected");
}

#[tokio::test]
async fn operator_can_filter_events_for_object_type_and_future_window() {
    let app = build_router(AppState::demo().expect("state"));
    let analyze_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/analyze-object")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"object_id": "SAT-001", "timestamp_unix_seconds": 1_800_000_060})
                        .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(analyze_response.status(), StatusCode::OK);

    let overview_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/space-overview")
                .header("content-type", "application/json")
                .body(Body::from(json!({"window_hours": 24.0}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(overview_response.status(), StatusCode::OK);

    let filtered_events_response = app
        .oneshot(
            Request::builder()
                .uri("/v1/events?limit=5&object_id=SAT-001&event_type=BehaviorShiftDetected&future_only=true&after_unix_seconds=1800000000")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(filtered_events_response.status(), StatusCode::OK);

    let body = body_json(filtered_events_response).await;
    assert_eq!(body["data"].as_array().map_or(0, Vec::len), 2);
    assert_eq!(body["data"][0]["object_id"], "SAT-001");
    assert_eq!(body["data"][0]["event_type"], "BehaviorShiftDetected");
    assert!(
        body["data"][0]["target_epoch_unix_seconds"]
            .as_i64()
            .unwrap_or_default()
            >= 1_800_000_000
    );
    assert_eq!(body["data"][1]["object_id"], "SAT-001");
}

#[tokio::test]
async fn operator_can_fetch_future_events_timeline() {
    let app = build_router(AppState::demo().expect("state"));
    let analyze_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/analyze-object")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"object_id": "SAT-001", "timestamp_unix_seconds": 1_800_000_060})
                        .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(analyze_response.status(), StatusCode::OK);

    let overview_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/space-overview")
                .header("content-type", "application/json")
                .body(Body::from(json!({"window_hours": 24.0}).to_string()))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(overview_response.status(), StatusCode::OK);

    let timeline_response = app
        .oneshot(
            Request::builder()
                .uri("/v1/events/timeline?horizon=72&limit_per_bucket=5&object_id=SAT-001&reference_unix_seconds=1800000000")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(timeline_response.status(), StatusCode::OK);

    let body = body_json(timeline_response).await;
    assert_eq!(body["data"]["horizon_hours"], 72.0);
    assert!(body["data"]["generated_at_unix_seconds"].is_number());
    assert_eq!(body["data"]["total_events"], 2);
    assert_eq!(body["data"]["buckets"][0]["label"], "24h+");
    assert_eq!(
        body["data"]["buckets"][0]["events"][0]["object_id"],
        "SAT-001"
    );
    assert!(body["data"]["buckets"][0]["events"][0]["target_epoch_unix_seconds"].is_number());
}

#[tokio::test]
async fn operator_can_fetch_object_timeline() {
    let app = build_router(
        AppState::from_storage(SqliteStore::in_memory().expect("store")).expect("state"),
    );
    let ingest_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest/tle")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"source": "celestrak", "payload": CELESTRAK_ISS_SNAPSHOT}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(ingest_response.status(), StatusCode::OK);

    let timeline_response = app
        .oneshot(
            Request::builder()
                .uri("/v1/objects/25544/timeline?horizon=72")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(timeline_response.status(), StatusCode::OK);
    let body = body_json(timeline_response).await;
    assert_eq!(body["data"]["object_id"], "25544");
    assert_eq!(body["data"]["checkpoints"][0]["offset_hours"], 1.0);
    assert_eq!(body["data"]["checkpoints"][1]["offset_hours"], 6.0);
    assert_eq!(body["data"]["checkpoints"][2]["offset_hours"], 24.0);
    assert_eq!(body["data"]["checkpoints"][3]["offset_hours"], 72.0);
    assert_eq!(body["data"]["checkpoints"][0]["propagation_model"], "sgp4");
    assert!(body["data"]["checkpoints"][0]["predicted_state"].is_object());
}

#[tokio::test]
async fn operator_can_fetch_prediction_snapshots_for_object() {
    let app = build_router(AppState::demo().expect("state"));

    let analyze_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/analyze-object")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"object_id": "SAT-001", "timestamp_unix_seconds": 1_800_000_060})
                        .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(analyze_response.status(), StatusCode::OK);

    let predict_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/predict")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"object_id": "SAT-001", "horizon_hours": 72.0}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(predict_response.status(), StatusCode::OK);

    let snapshots_response = app
        .oneshot(
            Request::builder()
                .uri("/v1/objects/SAT-001/predictions?limit=5")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(snapshots_response.status(), StatusCode::OK);

    let body = body_json(snapshots_response).await;
    let snapshots = body["data"].as_array().expect("snapshots");
    assert_eq!(snapshots.len(), 2);
    assert_eq!(snapshots[0]["object_id"], "SAT-001");
    assert!(snapshots[0]["generated_at_unix_seconds"].is_number());
    assert!(snapshots[0]["predictions"]
        .as_array()
        .is_some_and(|items| !items.is_empty()));
    assert_eq!(snapshots[1]["endpoint"], "/v1/analyze-object");
    assert!(snapshots[1]["evidence_bundle_hash"].is_string());
}

#[tokio::test]
async fn operator_can_fetch_recent_ingest_batches() {
    let app = build_router(AppState::demo().expect("state"));
    let ingest_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest/tle")
                .header("content-type", "application/json")
                .header("x-request-id", "ingest-history-test")
                .body(Body::from(
                    json!({"source": "celestrak", "payload": CELESTRAK_ISS_SNAPSHOT}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(ingest_response.status(), StatusCode::OK);

    let history_response = app
        .oneshot(
            Request::builder()
                .uri("/v1/ingest/batches?limit=1")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(history_response.status(), StatusCode::OK);
    let body = body_json(history_response).await;
    assert_eq!(body["data"][0]["request_id"], "ingest-history-test");
    assert_eq!(body["data"][0]["source"], "celestrak");
    assert_eq!(body["data"][0]["records_received"], 1);
    assert_eq!(body["data"][0]["object_ids"][0], "25544");
}

#[tokio::test]
async fn operator_can_fetch_ingest_source_status() {
    let app = build_router(AppState::demo().expect("state"));
    let ingest_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest/tle")
                .header("content-type", "application/json")
                .header("x-request-id", "status-test")
                .body(Body::from(
                    json!({"source": "celestrak", "payload": CELESTRAK_ISS_SNAPSHOT}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(ingest_response.status(), StatusCode::OK);

    let status_response = app
        .oneshot(
            Request::builder()
                .uri("/v1/ingest/status?source=celestrak")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(status_response.status(), StatusCode::OK);
    let body = body_json(status_response).await;
    assert_eq!(body["data"]["source"], "celestrak");
    assert_eq!(body["data"]["latest_request_id"], "status-test");
    assert_eq!(body["data"]["records_received"], 1);
    assert_eq!(body["data"]["object_count"], 1);
}

#[tokio::test]
async fn duplicate_tle_ingest_is_reported_without_new_processing() {
    let app = build_router(AppState::demo().expect("state"));
    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest/tle")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"source": "celestrak", "payload": CELESTRAK_ISS_SNAPSHOT}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(first.status(), StatusCode::OK);

    let second = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest/tle")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"source": "celestrak", "payload": CELESTRAK_ISS_SNAPSHOT}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(second.status(), StatusCode::OK);
    let body = body_json(second).await;
    assert_eq!(body["data"]["skipped_duplicate"], true);
    assert_eq!(body["data"]["records_received"], 1);
}

#[tokio::test]
async fn missing_evidence_returns_explicit_error_code() {
    let response = build_router(AppState::demo().expect("state"))
        .oneshot(
            Request::builder()
                .uri("/v1/evidence/missing")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = body_json(response).await;
    assert_eq!(body["error"]["code"], "evidence_bundle_not_found");
}

#[tokio::test]
async fn evidence_survives_reopening_sqlite_storage() {
    let path = persistent_test_db_path("evidence_survives_reopening_sqlite_storage");
    let app = build_router(
        AppState::demo_with_storage(SqliteStore::open(&path).expect("store")).expect("state"),
    );
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/analyze-object")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"object_id": "SAT-001", "timestamp_unix_seconds": 1_800_000_060})
                        .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    let body = body_json(response).await;
    let bundle_hash = body["data"]["evidence_bundle"]["bundle_hash"]
        .as_str()
        .expect("bundle hash")
        .to_string();

    let reopened = build_router(
        AppState::demo_with_storage(SqliteStore::open(&path).expect("store")).expect("state"),
    );
    let evidence_response = reopened
        .oneshot(
            Request::builder()
                .uri(format!("/v1/evidence/{bundle_hash}"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(evidence_response.status(), StatusCode::OK);
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn ingested_object_survives_reopening_sqlite_storage_for_analysis() {
    let path = persistent_test_db_path("ingested_object_survives_reopening");
    let app = build_router(
        AppState::from_storage(SqliteStore::open(&path).expect("store")).expect("state"),
    );
    let ingest_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/ingest/tle")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"source": "celestrak", "payload": CELESTRAK_ISS_SNAPSHOT}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(ingest_response.status(), StatusCode::OK);

    let reopened = build_router(
        AppState::from_storage(SqliteStore::open(&path).expect("store")).expect("state"),
    );
    let analyze_response = reopened
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/analyze-object")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"object_id": "25544", "timestamp_unix_seconds": null}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(analyze_response.status(), StatusCode::OK);
    let body = body_json(analyze_response).await;
    assert_eq!(body["data"]["object_id"], "25544");
    assert_eq!(body["data"]["evidence_bundle"]["sources"][0], "Catalog");
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn operator_can_dispatch_notifications_to_webhook_and_read_audit_log() {
    let received = Arc::new(Mutex::new(Vec::<Value>::new()));
    let webhook_url = spawn_webhook_server(received.clone(), false).await;
    let app = build_router(
        AppState::demo()
            .expect("state")
            .with_webhook_endpoint(webhook_url),
    );

    let dispatch_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/notifications/dispatch")
                .header("content-type", "application/json")
                .header("x-request-id", "dispatch-test")
                .body(Body::from(
                    json!({"object_id": "SAT-001", "timestamp_unix_seconds": 1_800_000_060})
                        .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(dispatch_response.status(), StatusCode::OK);
    let dispatch_body = body_json(dispatch_response).await;
    assert_eq!(dispatch_body["data"]["analysis"]["object_id"], "SAT-001");
    assert_eq!(dispatch_body["data"]["deliveries"][0]["channel"], "webhook");
    assert_eq!(
        dispatch_body["data"]["deliveries"][0]["status"],
        "delivered"
    );
    assert_eq!(dispatch_body["data"]["deliveries"][0]["attempt_number"], 1);
    assert_eq!(dispatch_body["data"]["deliveries"][0]["max_attempts"], 3);
    let delivery_count = dispatch_body["data"]["deliveries"]
        .as_array()
        .map_or(0, Vec::len);

    let payloads = received.lock().await;
    assert_eq!(payloads.len(), delivery_count);
    assert!(payloads.len() >= 3);
    assert_eq!(payloads[0]["object_id"], "SAT-001");
    assert!(payloads[0]["notification"]["notification_reason"].is_string());
    drop(payloads);

    let deliveries_response = app
        .oneshot(
            Request::builder()
                .uri("/v1/notifications?limit=5&object_id=SAT-001")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(deliveries_response.status(), StatusCode::OK);
    let deliveries_body = body_json(deliveries_response).await;
    assert_eq!(deliveries_body["data"][0]["object_id"], "SAT-001");
    assert_eq!(deliveries_body["data"][0]["request_id"], "dispatch-test");
    assert_eq!(deliveries_body["data"][0]["attempt_number"], 1);
    assert_eq!(deliveries_body["data"][0]["max_attempts"], 3);
}

#[tokio::test]
async fn notification_dispatch_retries_and_audits_each_attempt() {
    let received = Arc::new(Mutex::new(Vec::<Value>::new()));
    let webhook_url = spawn_webhook_server(received.clone(), true).await;
    let app = build_router(
        AppState::demo()
            .expect("state")
            .with_webhook_endpoint(webhook_url)
            .with_webhook_delivery_policy(WebhookDeliveryPolicy {
                max_attempts: 2,
                retry_delay: std::time::Duration::from_millis(5),
            }),
    );

    let dispatch_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/notifications/dispatch")
                .header("content-type", "application/json")
                .header("x-request-id", "retry-test")
                .body(Body::from(
                    json!({"object_id": "SAT-001", "timestamp_unix_seconds": 1_800_000_060})
                        .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(dispatch_response.status(), StatusCode::OK);
    let dispatch_body = body_json(dispatch_response).await;
    let deliveries = dispatch_body["data"]["deliveries"]
        .as_array()
        .expect("deliveries");
    assert!(deliveries.len() >= 4);
    assert_eq!(deliveries[0]["status"], "retrying");
    assert_eq!(deliveries[0]["attempt_number"], 1);
    assert_eq!(deliveries[1]["status"], "delivered");
    assert_eq!(deliveries[1]["attempt_number"], 2);

    let deliveries_response = app
        .oneshot(
            Request::builder()
                .uri("/v1/notifications?limit=10&object_id=SAT-001")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(deliveries_response.status(), StatusCode::OK);
    let deliveries_body = body_json(deliveries_response).await;
    let persisted = deliveries_body["data"]
        .as_array()
        .expect("persisted deliveries");
    assert!(persisted
        .iter()
        .any(|item| { item["attempt_number"] == 2 && item["status"] == "delivered" }));
    assert!(persisted
        .iter()
        .any(|item| { item["attempt_number"] == 1 && item["status"] == "retrying" }));
    assert_eq!(received.lock().await.len(), deliveries.len());
}

#[allow(clippy::too_many_lines)]
fn close_approach_demo_state() -> AppState {
    let primary = SpaceObject {
        id: "SAT-PRIMARY".to_string(),
        name: "Primary Watch Object".to_string(),
        regime: OrbitRegime::Leo,
        state: OrbitalState {
            epoch_unix_seconds: 1_800_000_000,
            position_km: Vector3 {
                x: 6_950.0,
                y: 0.0,
                z: 0.0,
            },
            velocity_km_s: Vector3 {
                x: 0.0,
                y: 7.6,
                z: 0.0,
            },
        },
        tle: None,
        behavior: BehaviorSignature {
            nominal_rf_mhz: Some(2_240.0),
            historical_maneuvers_per_week: 0.2,
            station_keeping_expected: false,
        },
        risk: RiskProfile {
            operator: Some("alpha".to_string()),
            mission_class: MissionClass::Commercial,
            criticality: 0.7,
        },
    };
    let near = SpaceObject {
        id: "SAT-NEAR".to_string(),
        name: "Close Neighbor".to_string(),
        regime: OrbitRegime::Leo,
        state: OrbitalState {
            epoch_unix_seconds: 1_800_000_000,
            position_km: Vector3 {
                x: 6_950.0,
                y: 180.0,
                z: 0.0,
            },
            velocity_km_s: Vector3 {
                x: 0.0,
                y: 7.55,
                z: 0.0,
            },
        },
        tle: None,
        behavior: BehaviorSignature {
            nominal_rf_mhz: None,
            historical_maneuvers_per_week: 0.1,
            station_keeping_expected: false,
        },
        risk: RiskProfile {
            operator: Some("beta".to_string()),
            mission_class: MissionClass::Commercial,
            criticality: 0.6,
        },
    };
    let far = SpaceObject {
        id: "SAT-FAR".to_string(),
        name: "Far Object".to_string(),
        regime: OrbitRegime::Leo,
        state: OrbitalState {
            epoch_unix_seconds: 1_800_000_000,
            position_km: Vector3 {
                x: 12_000.0,
                y: 12_000.0,
                z: 0.0,
            },
            velocity_km_s: Vector3 {
                x: -1.0,
                y: 1.0,
                z: 0.0,
            },
        },
        tle: None,
        behavior: BehaviorSignature {
            nominal_rf_mhz: None,
            historical_maneuvers_per_week: 0.0,
            station_keeping_expected: false,
        },
        risk: RiskProfile {
            operator: Some("gamma".to_string()),
            mission_class: MissionClass::Unknown,
            criticality: 0.2,
        },
    };
    let mut intelligence = sss_core::IntelligenceLayer::new(
        DigitalTwin::new(vec![primary.clone(), near.clone(), far]),
        AnticipationEngine::default(),
    );
    intelligence.record_observation(Observation {
        object_id: primary.id.clone(),
        observed_state: primary.state.clone(),
        rf_mhz: Some(2_240.2),
        maneuver_detected: false,
        source: ObservationSource::Synthetic,
    });
    intelligence.record_observation(Observation {
        object_id: near.id.clone(),
        observed_state: near.state.clone(),
        rf_mhz: None,
        maneuver_detected: false,
        source: ObservationSource::Synthetic,
    });
    AppState::new(
        intelligence,
        SqliteStore::in_memory().expect("memory store"),
    )
}

fn sample_passive_scan_request() -> Value {
    json!({
        "window_start_unix_seconds": 1_776_384_000_i64,
        "window_end_unix_seconds": 1_776_470_400_i64,
        "infra_sites": [
            {
                "name": "Seville Solar South",
                "site_type": "SolarPlant",
                "latitude": 37.3891,
                "longitude": -5.9845,
                "elevation_m": 12.0,
                "timezone": "Europe/Madrid",
                "country_code": "ES",
                "criticality": "High",
                "operator_name": "Open Corpus Energy",
                "observation_radius_km": 18.0,
                "source_reference": "openinframap:substation:sev-001"
            }
        ],
        "adsb_observations": [
            {
                "flight_id": "ADSBX-ES001",
                "observed_at_unix_seconds": 1_776_390_000_i64,
                "latitude": 37.40,
                "longitude": -5.99,
                "altitude_m": 620.0,
                "speed_kts": 92.0,
                "vertical_rate_m_s": -1.0
            },
            {
                "flight_id": "ADSBX-ES002",
                "observed_at_unix_seconds": 1_776_391_800_i64,
                "latitude": 37.395,
                "longitude": -5.986,
                "altitude_m": 540.0,
                "speed_kts": 88.0,
                "vertical_rate_m_s": -0.4
            }
        ],
        "weather_observations": [
            {
                "observed_at_unix_seconds": 1_776_392_000_i64,
                "latitude": 37.42,
                "longitude": -5.97,
                "wind_kph": 58.0,
                "hail_probability": 0.62,
                "irradiance_drop_ratio": 0.41
            }
        ],
        "fire_smoke_observations": [
            {
                "observed_at_unix_seconds": 1_776_394_000_i64,
                "latitude": 37.44,
                "longitude": -6.00,
                "fire_radiative_power": 78.0,
                "smoke_density_index": 61.0
            }
        ],
        "orbital_observations": [],
        "satellite_observations": [
            {
                "scene_id": "sentinel-2a-t32",
                "observed_at_unix_seconds": 1_776_398_000_i64,
                "latitude": 37.39,
                "longitude": -5.985,
                "change_score": 0.73,
                "cloud_cover_ratio": 0.12
            }
        ],
        "notam_observations": []
    })
}

fn sample_passive_live_scan_request() -> Value {
    json!({
        "window_hours": 24,
        "infra_sites": [
            {
                "name": "Seville Solar South",
                "site_type": "SolarPlant",
                "latitude": 37.3891,
                "longitude": -5.9845,
                "elevation_m": 12.0,
                "timezone": "Europe/Madrid",
                "country_code": "ES",
                "criticality": "High",
                "operator_name": "Open Corpus Energy",
                "observation_radius_km": 18.0,
                "source_reference": "openinframap:substation:sev-001"
            }
        ],
        "include_adsb": true,
        "include_weather": true,
        "include_fire_smoke": true
    })
}

async fn body_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    serde_json::from_slice(&bytes).expect("json")
}

fn assert_attention_item_shape(item: &Value) {
    for field in [
        "item_id",
        "kind",
        "priority",
        "title",
        "reason",
        "primary_action_label",
    ] {
        let value = item[field].as_str().unwrap_or_default();
        assert!(
            !value.is_empty(),
            "attention item should include non-empty {field}: {item:?}"
        );
    }
    assert!(
        ["region_id", "site_id", "canonical_event_id", "action_path"]
            .iter()
            .any(|field| !item[*field].as_str().unwrap_or_default().is_empty()),
        "attention item should include a mappable target or action path: {item:?}"
    );
}

fn attention_priority_rank(priority: &str) -> u8 {
    match priority {
        "low" => 0,
        "medium" => 1,
        "high" => 2,
        "critical" => 3,
        other => panic!("unknown attention priority: {other}"),
    }
}

fn replay_manifest_hash(bundle: &Value) -> String {
    let events = bundle["events"].as_array().expect("events");
    let event_ids = events
        .iter()
        .map(|event| event["event_id"].as_str().expect("event_id").to_string())
        .collect::<Vec<_>>();
    assert!(!event_ids.is_empty());

    // The manifest itself is stored by the analyze request. This test only needs
    // the hash from the API state; fetch it via a second analyze response field
    // once public contracts expose it. Until then, call the deterministic core
    // helper through the typed bundle from the response.
    let typed: sss_core::EvidenceBundle =
        serde_json::from_value(bundle.clone()).expect("typed bundle");
    sss_core::replay_manifest_for(&typed).manifest_hash
}

fn persistent_test_db_path(name: &str) -> std::path::PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir().join(format!("sss-api-{name}-{unique}.sqlite"))
}

async fn spawn_webhook_server(received: Arc<Mutex<Vec<Value>>>, fail_first: bool) -> String {
    async fn webhook_handler(
        State(state): State<WebhookTestState>,
        Json(payload): Json<Value>,
    ) -> (StatusCode, Json<Value>) {
        let attempt = state.attempts.fetch_add(1, Ordering::SeqCst) + 1;
        state.received.lock().await.push(payload);
        if state.fail_first && attempt == 1 {
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({"status": "retry", "attempt": attempt})),
            )
        } else {
            (
                StatusCode::OK,
                Json(json!({"status": "ok", "attempt": attempt})),
            )
        }
    }

    let state = WebhookTestState {
        received,
        attempts: Arc::new(AtomicUsize::new(0)),
        fail_first,
    };

    let app = Router::new()
        .route("/hook", post(webhook_handler))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind webhook");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve webhook");
    });
    format!("http://{addr}/hook")
}

async fn spawn_nasa_apod_server(requests: Arc<Mutex<Vec<String>>>) -> String {
    async fn apod_handler(State(state): State<NasaApodTestState>, uri: OriginalUri) -> Json<Value> {
        state.requests.lock().await.push(uri.to_string());
        Json(json!({
            "title": "Astronomy Picture Test",
            "date": "2026-04-18",
            "explanation": "Synthetic APOD payload for contract verification.",
            "media_type": "image",
            "url": "https://example.test/apod.jpg",
            "hdurl": "https://example.test/apod-hd.jpg",
            "service_version": "v1"
        }))
    }

    let app = Router::new()
        .route("/planetary/apod", get(apod_handler))
        .with_state(NasaApodTestState { requests });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind nasa apod");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve nasa apod");
    });
    format!("http://{addr}")
}

async fn spawn_nasa_neows_server(requests: Arc<Mutex<Vec<String>>>) -> String {
    async fn neows_handler(
        State(state): State<NasaNeoWsTestState>,
        uri: OriginalUri,
    ) -> Json<Value> {
        state.requests.lock().await.push(uri.to_string());
        Json(json!({
            "element_count": 2,
            "near_earth_objects": {
                "2026-04-18": [
                    {
                        "id": "1",
                        "name": "(2026 AB)",
                        "nasa_jpl_url": "https://ssd.jpl.nasa.gov/tools/sbdb_lookup.html#/?sstr=1",
                        "is_potentially_hazardous_asteroid": true,
                        "estimated_diameter": {
                            "meters": {
                                "estimated_diameter_min": 120.0,
                                "estimated_diameter_max": 260.0
                            }
                        },
                        "close_approach_data": [
                            {
                                "close_approach_date": "2026-04-18",
                                "relative_velocity": { "kilometers_per_second": "12.5" },
                                "miss_distance": { "kilometers": "450000.0" }
                            }
                        ]
                    },
                    {
                        "id": "2",
                        "name": "(2026 CD)",
                        "nasa_jpl_url": null,
                        "is_potentially_hazardous_asteroid": false,
                        "estimated_diameter": {
                            "meters": {
                                "estimated_diameter_min": 40.0,
                                "estimated_diameter_max": 90.0
                            }
                        },
                        "close_approach_data": [
                            {
                                "close_approach_date": "2026-04-18",
                                "relative_velocity": { "kilometers_per_second": "8.2" },
                                "miss_distance": { "kilometers": "1800000.0" }
                            }
                        ]
                    }
                ]
            }
        }))
    }

    let app = Router::new()
        .route("/neo/rest/v1/feed", get(neows_handler))
        .with_state(NasaNeoWsTestState { requests });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind nasa neows");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve nasa neows");
    });
    format!("http://{addr}")
}

async fn spawn_passive_feeds_server(requests: Arc<Mutex<Vec<String>>>) -> String {
    async fn open_meteo_handler(
        State(state): State<PassiveFeedsTestState>,
        uri: OriginalUri,
    ) -> Json<Value> {
        state.requests.lock().await.push(uri.to_string());
        Json(json!({
            "current": {
                "time": 1_776_470_000_i64,
                "wind_speed_10m": 42.0,
                "precipitation_probability": 68.0,
                "shortwave_radiation": 310.0,
                "cloud_cover": 72.0
            }
        }))
    }

    async fn opensky_handler(
        State(state): State<PassiveFeedsTestState>,
        uri: OriginalUri,
    ) -> Json<Value> {
        state.requests.lock().await.push(uri.to_string());
        Json(json!({
            "time": 1_776_470_100_i64,
            "states": [
                [
                    "abc123",
                    "IB1234 ",
                    "Spain",
                    1_776_470_090_i64,
                    1_776_470_100_i64,
                    -5.988,
                    37.392,
                    640.0,
                    false,
                    47.0,
                    180.0,
                    -0.6,
                    null,
                    650.0,
                    null,
                    false,
                    0
                ]
            ]
        }))
    }

    async fn firms_handler(
        State(state): State<PassiveFeedsTestState>,
        uri: OriginalUri,
    ) -> (StatusCode, String) {
        state.requests.lock().await.push(uri.to_string());
        (
            StatusCode::OK,
            "latitude,longitude,frp,confidence\n37.401,-5.971,55.5,78\n".to_string(),
        )
    }

    let state = PassiveFeedsTestState { requests };
    let app = Router::new()
        .route("/v1/forecast", get(open_meteo_handler))
        .route("/states/all", get(opensky_handler))
        .route(
            "/api/area/csv/:map_key/:source/:area/:days",
            get(firms_handler),
        )
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind passive feeds");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("serve passive feeds");
    });
    format!("http://{addr}")
}

async fn spawn_overpass_server(bodies: Arc<Mutex<Vec<String>>>) -> String {
    async fn overpass_handler(State(state): State<OverpassTestState>, body: String) -> Json<Value> {
        state.bodies.lock().await.push(body);
        Json(json!({
            "elements": [
                {
                    "type": "way",
                    "id": 101,
                    "center": { "lat": 37.3891, "lon": -5.9845 },
                    "tags": {
                        "power": "plant",
                        "plant:source": "solar",
                        "name": "Seville Solar South",
                        "operator": "Open Corpus Energy",
                        "plant:output:electricity": "50 MW"
                    }
                },
                {
                    "type": "way",
                    "id": 202,
                    "center": { "lat": 37.4011, "lon": -5.9650 },
                    "tags": {
                        "telecom": "data_center",
                        "building": "data_center",
                        "name": "Andalucia Data Hub",
                        "operator": "Cloud Iberia"
                    }
                },
                {
                    "type": "node",
                    "id": 303,
                    "lat": 37.4120,
                    "lon": -5.9550,
                    "tags": {
                        "power": "substation",
                        "name": "Seville Grid East",
                        "voltage": "220000"
                    }
                },
                {
                    "type": "way",
                    "id": 404,
                    "center": { "lat": 37.4100, "lon": -5.9500 },
                    "tags": {
                        "power": "generator",
                        "generator:source": "solar",
                        "generator:method": "photovoltaic",
                        "location": "roof",
                        "name": "Roof Array Tiny"
                    }
                }
            ]
        }))
    }

    let app = Router::new()
        .route("/api/interpreter", post(overpass_handler))
        .with_state(OverpassTestState { bodies });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind overpass");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve overpass");
    });
    format!("http://{addr}/api/interpreter")
}

#[allow(clippy::too_many_lines)]
async fn spawn_passive_discovery_and_feeds_server(
    requests: Arc<Mutex<Vec<String>>>,
    bodies: Arc<Mutex<Vec<String>>>,
) -> String {
    async fn open_meteo_handler(
        State(state): State<PassiveDiscoveryAndFeedsState>,
        uri: OriginalUri,
    ) -> Json<Value> {
        state.requests.lock().await.push(uri.to_string());
        Json(json!({
            "current": {
                "time": 1_776_470_000_i64,
                "wind_speed_10m": 42.0,
                "precipitation_probability": 68.0,
                "shortwave_radiation": 310.0,
                "cloud_cover": 72.0
            }
        }))
    }

    async fn opensky_handler(
        State(state): State<PassiveDiscoveryAndFeedsState>,
        uri: OriginalUri,
    ) -> Json<Value> {
        state.requests.lock().await.push(uri.to_string());
        Json(json!({
            "time": 1_776_470_100_i64,
            "states": [
                [
                    "abc123",
                    "IB1234 ",
                    "Spain",
                    1_776_470_090_i64,
                    1_776_470_100_i64,
                    -5.988,
                    37.392,
                    640.0,
                    false,
                    47.0,
                    180.0,
                    -0.6,
                    null,
                    650.0,
                    null,
                    false,
                    0
                ]
            ]
        }))
    }

    async fn firms_handler(
        State(state): State<PassiveDiscoveryAndFeedsState>,
        uri: OriginalUri,
    ) -> (StatusCode, String) {
        state.requests.lock().await.push(uri.to_string());
        (
            StatusCode::OK,
            "latitude,longitude,frp,confidence\n37.401,-5.971,55.5,78\n".to_string(),
        )
    }

    async fn overpass_handler(
        State(state): State<PassiveDiscoveryAndFeedsState>,
        body: String,
    ) -> Json<Value> {
        state.bodies.lock().await.push(body);
        Json(json!({
            "elements": [
                {
                    "type": "way",
                    "id": 101,
                    "center": { "lat": 37.3891, "lon": -5.9845 },
                    "tags": {
                        "power": "plant",
                        "plant:source": "solar",
                        "name": "Seville Solar South",
                        "operator": "Open Corpus Energy",
                        "plant:output:electricity": "50 MW"
                    }
                },
                {
                    "type": "way",
                    "id": 202,
                    "center": { "lat": 37.4011, "lon": -5.9650 },
                    "tags": {
                        "telecom": "data_center",
                        "building": "data_center",
                        "name": "Andalucia Data Hub",
                        "operator": "Cloud Iberia"
                    }
                },
                {
                    "type": "node",
                    "id": 303,
                    "lat": 37.4120,
                    "lon": -5.9550,
                    "tags": {
                        "power": "substation",
                        "name": "Seville Grid East",
                        "voltage": "220000"
                    }
                }
            ]
        }))
    }

    let app = Router::new()
        .route("/v1/forecast", get(open_meteo_handler))
        .route("/states/all", get(opensky_handler))
        .route(
            "/api/area/csv/:map_key/:source/:area/:days",
            get(firms_handler),
        )
        .route("/api/interpreter", post(overpass_handler))
        .with_state(PassiveDiscoveryAndFeedsState { requests, bodies });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind passive discovery and feeds");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("serve passive discovery and feeds");
    });
    format!("http://{addr}")
}
