use uuid::Uuid;

use crate::action_router::{route_for, route_for_action, ResponseRoute};
use crate::authorization::{authorize_gate, AuthorizationOutcome};
use crate::policy_eval::policy_allows;
use sss_skyshield::{
    AuthorizationGate, GateStatus, IncidentStatus, IncidentType, ResponseAction, ResponsePolicy,
    Severity, SiteIncident, SkyThreatAssessment, ThreatType,
};

#[test]
fn policy_allows_when_thresholds_are_met() {
    let policy = sample_policy(vec![ResponseAction::DispatchDroneIntercept]);
    let evaluation = policy_allows(
        &policy,
        &sample_threat(0.92),
        &sample_incident(Severity::High),
    );
    assert!(evaluation.allowed);
    assert_eq!(evaluation.reason, "policy thresholds satisfied");
}

#[test]
fn policy_blocks_when_confidence_is_low() {
    let policy = sample_policy(vec![ResponseAction::DispatchDroneIntercept]);
    let evaluation = policy_allows(
        &policy,
        &sample_threat(0.40),
        &sample_incident(Severity::High),
    );
    assert!(!evaluation.allowed);
    assert!(evaluation.reason.contains("policy thresholds not met"));
}

#[test]
fn authorization_gate_maps_status() {
    assert_eq!(
        authorize_gate(&sample_gate(GateStatus::Approved)),
        AuthorizationOutcome::Approved
    );
    assert_eq!(
        authorize_gate(&sample_gate(GateStatus::Pending)),
        AuthorizationOutcome::Pending
    );
    assert_eq!(
        authorize_gate(&sample_gate(GateStatus::Expired)),
        AuthorizationOutcome::Pending
    );
    assert_eq!(
        authorize_gate(&sample_gate(GateStatus::Rejected)),
        AuthorizationOutcome::Rejected
    );
}

#[test]
fn route_for_maps_drone_actions_to_anti_drone() {
    for action in [
        ResponseAction::DispatchDroneIntercept,
        ResponseAction::DispatchDroneInspection,
        ResponseAction::DispatchDronePatrol,
    ] {
        let policy = sample_policy(vec![action]);
        let dispatch =
            route_for(&policy, &sample_incident(Severity::Critical)).expect("dispatch route");
        assert_eq!(dispatch.route, ResponseRoute::AntiDrone);
    }
}

#[test]
fn route_for_maps_security_actions_to_site_security() {
    for action in [
        ResponseAction::EscalateToSiteSecurity,
        ResponseAction::EscalateToSoc,
        ResponseAction::ActivatePerimeterHardening,
    ] {
        let policy = sample_policy(vec![action]);
        let dispatch =
            route_for(&policy, &sample_incident(Severity::Critical)).expect("dispatch route");
        assert_eq!(dispatch.route, ResponseRoute::SiteSecurity);
    }
}

#[test]
fn route_for_maps_observe_only_to_operator_only() {
    let policy = sample_policy(vec![ResponseAction::ObserveOnly]);
    let dispatch = route_for(&policy, &sample_incident(Severity::Medium)).expect("dispatch route");
    assert_eq!(dispatch.route, ResponseRoute::OperatorOnly);
}

#[test]
fn route_for_returns_none_without_allowed_actions() {
    let policy = sample_policy(Vec::new());
    assert!(route_for(&policy, &sample_incident(Severity::Medium)).is_none());
}

#[test]
fn route_for_action_honors_explicit_allowed_action() {
    let policy = sample_policy(vec![
        ResponseAction::ObserveOnly,
        ResponseAction::DispatchDroneInspection,
    ]);
    let dispatch = route_for_action(
        &policy,
        &sample_incident(Severity::High),
        ResponseAction::DispatchDroneInspection,
    )
    .expect("dispatch route");
    assert_eq!(
        dispatch.selected_action,
        ResponseAction::DispatchDroneInspection
    );
    assert_eq!(dispatch.route, ResponseRoute::AntiDrone);
}

#[test]
fn route_for_action_rejects_action_outside_policy() {
    let policy = sample_policy(vec![ResponseAction::ObserveOnly]);
    assert!(route_for_action(
        &policy,
        &sample_incident(Severity::High),
        ResponseAction::DispatchDroneInspection,
    )
    .is_none());
}

fn sample_policy(allowed_actions: Vec<ResponseAction>) -> ResponsePolicy {
    ResponsePolicy {
        policy_id: Uuid::new_v4(),
        site_id: Uuid::new_v4(),
        threat_type: ThreatType::UnauthorizedOverflight,
        minimum_confidence: 0.65,
        minimum_severity: Severity::Medium,
        allowed_actions,
        requires_human_authorization: true,
        requires_external_authority: false,
    }
}

fn sample_threat(confidence: f32) -> SkyThreatAssessment {
    SkyThreatAssessment {
        assessment_id: Uuid::new_v4(),
        site_id: Uuid::new_v4(),
        threat_type: ThreatType::UnauthorizedOverflight,
        confidence,
        impact_probability: 0.81,
        impact_window_seconds: 600,
        affected_zones: vec![Uuid::new_v4()],
        affected_assets: vec![Uuid::new_v4()],
        explanation: "threat".to_string(),
        predicted_path_summary: "path".to_string(),
    }
}

fn sample_incident(severity: Severity) -> SiteIncident {
    SiteIncident {
        incident_id: Uuid::new_v4(),
        site_id: Uuid::new_v4(),
        incident_type: IncidentType::DroneDefense,
        status: IncidentStatus::Confirmed,
        created_at_unix_seconds: 1_800_000_000,
        updated_at_unix_seconds: 1_800_000_000,
        linked_track_ids: vec![Uuid::new_v4()],
        linked_event_ids: Vec::new(),
        linked_bundle_hashes: Vec::new(),
        severity,
        current_owner: Some("operator".to_string()),
    }
}

fn sample_gate(status: GateStatus) -> AuthorizationGate {
    AuthorizationGate {
        gate_id: Uuid::new_v4(),
        incident_id: Uuid::new_v4(),
        site_id: Uuid::new_v4(),
        action_type: ResponseAction::DispatchDroneIntercept,
        approved_by: "operator".to_string(),
        approved_at_unix_seconds: 1_800_000_000,
        approval_basis: "approved".to_string(),
        status,
    }
}
