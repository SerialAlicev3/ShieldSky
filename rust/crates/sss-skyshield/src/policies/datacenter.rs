use crate::{ResponseAction, ResponsePolicy, Severity, ThreatType};
use uuid::Uuid;

#[must_use]
pub fn default_datacenter_policy(site_id: Uuid, threat_type: ThreatType) -> ResponsePolicy {
    ResponsePolicy {
        policy_id: Uuid::new_v4(),
        site_id,
        threat_type,
        minimum_confidence: 0.75,
        minimum_severity: Severity::High,
        allowed_actions: vec![
            ResponseAction::ObserveOnly,
            ResponseAction::DispatchDronePatrol,
            ResponseAction::DispatchDroneIntercept,
            ResponseAction::EscalateToSoc,
        ],
        requires_human_authorization: true,
        requires_external_authority: matches!(
            threat_type,
            ThreatType::SuspectedPayloadRisk | ThreatType::MultiDronePattern
        ),
    }
}
