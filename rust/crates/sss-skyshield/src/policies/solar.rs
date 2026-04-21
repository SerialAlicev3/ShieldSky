use crate::{ResponseAction, ResponsePolicy, Severity, ThreatType};
use uuid::Uuid;

#[must_use]
pub fn default_solar_policy(site_id: Uuid, threat_type: ThreatType) -> ResponsePolicy {
    ResponsePolicy {
        policy_id: Uuid::new_v4(),
        site_id,
        threat_type,
        minimum_confidence: 0.65,
        minimum_severity: Severity::Medium,
        allowed_actions: vec![
            ResponseAction::ObserveOnly,
            ResponseAction::DispatchDroneInspection,
            ResponseAction::EscalateToSiteSecurity,
        ],
        requires_human_authorization: true,
        requires_external_authority: false,
    }
}
