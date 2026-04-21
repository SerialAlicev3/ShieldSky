use serde::{Deserialize, Serialize};

use sss_skyshield::{AuthorizationGate, GateStatus};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorizationOutcome {
    Approved,
    Pending,
    Rejected,
}

#[must_use]
pub fn authorize_gate(gate: &AuthorizationGate) -> AuthorizationOutcome {
    match gate.status {
        GateStatus::Approved => AuthorizationOutcome::Approved,
        GateStatus::Pending | GateStatus::Expired => AuthorizationOutcome::Pending,
        GateStatus::Rejected => AuthorizationOutcome::Rejected,
    }
}
