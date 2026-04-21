use serde::{Deserialize, Serialize};

use sss_skyshield::{ResponsePolicy, SiteIncident, SkyThreatAssessment};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicyEvaluation {
    pub allowed: bool,
    pub requires_human_authorization: bool,
    pub requires_external_authority: bool,
    pub reason: String,
}

#[must_use]
pub fn policy_allows(
    policy: &ResponsePolicy,
    threat: &SkyThreatAssessment,
    incident: &SiteIncident,
) -> PolicyEvaluation {
    let severity_ok = incident.severity >= policy.minimum_severity;
    let confidence_ok = threat.confidence >= policy.minimum_confidence;
    PolicyEvaluation {
        allowed: severity_ok && confidence_ok,
        requires_human_authorization: policy.requires_human_authorization,
        requires_external_authority: policy.requires_external_authority,
        reason: if severity_ok && confidence_ok {
            "policy thresholds satisfied".to_string()
        } else {
            format!(
                "policy thresholds not met: severity={:?}, confidence={:.2}",
                incident.severity, threat.confidence
            )
        },
    }
}
