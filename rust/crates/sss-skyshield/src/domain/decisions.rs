use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::threats::{Severity, ThreatType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RecipientRole {
    SiteOperator,
    SecurityOperationsCenter,
    FacilityManager,
    DroneDefenseCoordinator,
    GridOperator,
    LocalAuthority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscalationPolicy {
    MonitorOnly,
    NotifyAndTrack,
    HumanAuthorizationRequired,
    ExternalAuthorityRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseOption {
    ObserveOnly,
    DispatchDroneInspection,
    DispatchDronePatrol,
    DispatchDroneIntercept,
    EscalateToSiteSecurity,
    EscalateToSoc,
    ActivatePerimeterHardening,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseAction {
    ObserveOnly,
    DispatchDroneInspection,
    DispatchDronePatrol,
    DispatchDroneIntercept,
    EscalateToSiteSecurity,
    EscalateToSoc,
    ActivatePerimeterHardening,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SiteOperationalDecision {
    pub decision_id: Uuid,
    pub site_id: Uuid,
    pub severity: Severity,
    pub priority: u8,
    pub recommendation: String,
    pub recipients: Vec<RecipientRole>,
    pub escalation_policy: EscalationPolicy,
    pub authorized_response_options: Vec<ResponseOption>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponsePolicy {
    pub policy_id: Uuid,
    pub site_id: Uuid,
    pub threat_type: ThreatType,
    pub minimum_confidence: f32,
    pub minimum_severity: Severity,
    pub allowed_actions: Vec<ResponseAction>,
    pub requires_human_authorization: bool,
    pub requires_external_authority: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthorizationGate {
    pub gate_id: Uuid,
    pub incident_id: Uuid,
    pub site_id: Uuid,
    pub action_type: ResponseAction,
    pub approved_by: String,
    pub approved_at_unix_seconds: i64,
    pub approval_basis: String,
    pub status: GateStatus,
}
