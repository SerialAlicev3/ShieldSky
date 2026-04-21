use crate::{EscalationPolicy, RecipientRole, ResponseOption, Severity, ThreatType};

#[must_use]
pub fn recipients_for(threat_type: ThreatType, severity: Severity) -> Vec<RecipientRole> {
    let mut recipients = vec![
        RecipientRole::SiteOperator,
        RecipientRole::SecurityOperationsCenter,
    ];
    match threat_type {
        ThreatType::UnauthorizedOverflight
        | ThreatType::PerimeterAirspaceViolation
        | ThreatType::HoverOverCriticalZone
        | ThreatType::RepeatPassPattern
        | ThreatType::RfLinkAnomaly
        | ThreatType::SuspectedPayloadRisk
        | ThreatType::MultiDronePattern => {
            recipients.push(RecipientRole::DroneDefenseCoordinator);
        }
        ThreatType::IrradianceDropExpected
        | ThreatType::StormCellApproach
        | ThreatType::HailRisk
        | ThreatType::SmokeIntrusionRisk
        | ThreatType::DustHazeDetected
        | ThreatType::CoolingAirQualityRisk
        | ThreatType::WindLoadRisk
        | ThreatType::LightningRisk => {
            recipients.push(RecipientRole::FacilityManager);
        }
        ThreatType::OrbitalReentryContext
        | ThreatType::SpaceWeatherContext
        | ThreatType::OverheadRiskContext => recipients.push(RecipientRole::FacilityManager),
    }
    if matches!(severity, Severity::High | Severity::Critical) {
        recipients.push(RecipientRole::LocalAuthority);
    }
    recipients.sort();
    recipients.dedup();
    recipients
}

#[must_use]
pub fn escalation_policy_for(threat_type: ThreatType, severity: Severity) -> EscalationPolicy {
    match (threat_type, severity) {
        (ThreatType::MultiDronePattern | ThreatType::SuspectedPayloadRisk, _)
        | (_, Severity::Critical) => EscalationPolicy::ExternalAuthorityRequired,
        (
            ThreatType::UnauthorizedOverflight
            | ThreatType::PerimeterAirspaceViolation
            | ThreatType::HoverOverCriticalZone
            | ThreatType::RepeatPassPattern,
            Severity::High,
        ) => EscalationPolicy::HumanAuthorizationRequired,
        (_, Severity::Medium | Severity::High) => EscalationPolicy::NotifyAndTrack,
        _ => EscalationPolicy::MonitorOnly,
    }
}

#[must_use]
pub fn response_options_for(threat_type: ThreatType, severity: Severity) -> Vec<ResponseOption> {
    match threat_type {
        ThreatType::UnauthorizedOverflight
        | ThreatType::PerimeterAirspaceViolation
        | ThreatType::HoverOverCriticalZone
        | ThreatType::RepeatPassPattern
        | ThreatType::RfLinkAnomaly
        | ThreatType::SuspectedPayloadRisk
        | ThreatType::MultiDronePattern => vec![
            ResponseOption::ObserveOnly,
            ResponseOption::DispatchDroneInspection,
            ResponseOption::DispatchDronePatrol,
            if matches!(severity, Severity::High | Severity::Critical) {
                ResponseOption::DispatchDroneIntercept
            } else {
                ResponseOption::EscalateToSiteSecurity
            },
        ],
        _ => vec![
            ResponseOption::ObserveOnly,
            ResponseOption::EscalateToSoc,
            ResponseOption::ActivatePerimeterHardening,
        ],
    }
}
