use std::time::Duration;

use uuid::Uuid;

use sss_core::{
    AlertSeverity, AnalysisStatus, AnomalySignal, ConfidenceContributor, DerivedFeatures,
    EscalationRule, EvidenceBundle, EvidenceStrength, IntelligenceAssessment, IntelligenceEvent,
    IntelligenceEventType, IntelligenceRecipient, NotificationDecision, Observation,
    ObservationFinding, ObservationSource, OperationalDecision, Prediction, RankedEvent,
    RankedEventType, RecipientPolicy, ReplayManifest, RuleHit, Vector3,
};

use crate::application::decide_recipients::{
    escalation_policy_for, recipients_for, response_options_for,
};
use crate::application::evaluate_impact::{risk_score, zone_exposure_sensitivity};
use crate::engines::behavior::behavior_summary;
use crate::engines::classification::classify_track;
use crate::engines::impact::impact_summary;
use crate::engines::prediction::predicted_path_summary;
use crate::engines::signal_fusion::fusion_summary;
use crate::engines::track_engine::track_observation;
use crate::{
    IncidentStatus, IncidentType, RecipientRole, Severity, SiteContext, SiteIncident,
    SiteOperationalDecision, SkySignal, SkyThreatAssessment, ThreatType,
};

#[derive(Debug, Clone, PartialEq)]
pub struct AssessSiteSkyInput {
    pub site: SiteContext,
    pub signals: Vec<SkySignal>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkyshieldCoreEnvelope {
    pub finding: ObservationFinding,
    pub assessment: IntelligenceAssessment,
    pub decision: OperationalDecision,
    pub evidence_bundle: EvidenceBundle,
    pub replay_manifest: ReplayManifest,
    pub ranked_event: RankedEvent,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssessSiteSkyOutput {
    pub threat_assessment: SkyThreatAssessment,
    pub operational_decision: SiteOperationalDecision,
    pub incident: SiteIncident,
    pub core: SkyshieldCoreEnvelope,
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn assess_site_sky(input: &AssessSiteSkyInput) -> Option<AssessSiteSkyOutput> {
    let track = track_observation(&input.site.site, &input.signals)?;
    let affected_zone = input.site.zones.first()?;
    let affected_asset = input.site.assets.first()?;
    let threat_type = classify_track(&track, &input.site.site.site_type);
    let impact_window_seconds = 720;
    let exposure_sensitivity =
        zone_exposure_sensitivity(&input.site.exposure_profile, affected_zone, affected_asset);
    let score = risk_score(
        0.88,
        0.82,
        affected_zone.criticality,
        affected_asset.criticality,
        exposure_sensitivity,
        impact_window_seconds,
    )
    .clamp(0.0, 1.0);
    let severity = severity_for(score);
    let threat_assessment = SkyThreatAssessment {
        assessment_id: Uuid::new_v4(),
        site_id: input.site.site.site_id,
        threat_type,
        confidence: 0.88,
        impact_probability: score,
        impact_window_seconds,
        affected_zones: vec![affected_zone.zone_id],
        affected_assets: vec![affected_asset.asset_id],
        explanation: impact_summary(
            &input.site.site.name,
            threat_type,
            affected_zone,
            affected_asset,
        ),
        predicted_path_summary: predicted_path_summary(&track),
    };
    let recipients = recipients_for(threat_type, severity);
    let site_decision = SiteOperationalDecision {
        decision_id: Uuid::new_v4(),
        site_id: input.site.site.site_id,
        severity,
        priority: priority_for(score),
        recommendation: format!(
            "Defensive response only for {:?} near {}",
            threat_type, affected_zone.name
        ),
        recipients: recipients.clone(),
        escalation_policy: escalation_policy_for(threat_type, severity),
        authorized_response_options: response_options_for(threat_type, severity),
    };
    let observation = observation_from_track(&track);
    let finding = ObservationFinding {
        object_id: track.track_id.to_string(),
        observed_at_unix_seconds: Some(track.last_seen_unix_seconds),
        source: Some(ObservationSource::Api),
        anomaly_score: score.into(),
        signals: vec![AnomalySignal::PositionDrift],
        derived_features: DerivedFeatures {
            position_delta_km: 0.0,
            velocity_delta_km_s: velocity_magnitude_km_s(&track.velocity_vector),
            rf_delta_mhz: None,
            maneuver_detected: false,
        },
        summary: fusion_summary(&input.signals),
    };
    let prediction = Prediction {
        object_id: track.track_id.to_string(),
        horizon_hours: Duration::from_secs(impact_window_seconds.unsigned_abs()).as_secs_f64()
            / 3600.0,
        event: sss_core::PredictedEvent::ContinuedAnomalousBehavior,
        probability: score.into(),
        explanation: threat_assessment.explanation.clone(),
        propagation_model: "site-context".to_string(),
        based_on_observation_epoch: Some(track.last_seen_unix_seconds),
        predicted_state: None,
        position_delta_km: Some(0.0),
        velocity_delta_km_s: Some(velocity_magnitude_km_s(&track.velocity_vector)),
    };
    let assessment = IntelligenceAssessment {
        status: analysis_status_for(score),
        behavior: behavior_summary(&track),
        confidence: threat_assessment.confidence.into(),
        prediction: prediction.clone(),
        explanation: threat_assessment.explanation.clone(),
        decision_version: "skyshield-v1".to_string(),
        model_version: "skyshield-site-risk-v1".to_string(),
        rule_hits: vec![RuleHit {
            rule_id: "site_critical_infrastructure_mapping".to_string(),
            description: "site-specific sky threat semantics mapped onto SSS core".to_string(),
        }],
        signal_summary: input
            .signals
            .iter()
            .map(|signal| format!("{:?}:{:?}", signal.source, signal.signal_type))
            .collect(),
    };
    let decision = OperationalDecision {
        severity: alert_severity_for(severity),
        risk_score: score.into(),
        recommendation: site_decision.recommendation.clone(),
        recipient_policy: RecipientPolicy {
            policy_id: "sss-skyshield-recipient-policy-v1".to_string(),
            escalation_rules: escalation_rules_for(&site_decision),
        },
        notifications: site_decision
            .recipients
            .iter()
            .map(|recipient| NotificationDecision {
                recipient: intelligence_recipient_for(*recipient),
                notification_reason: format!(
                    "site={} severity={:?} threat={:?}",
                    input.site.site.name, severity, threat_type
                ),
            })
            .collect(),
    };
    let evidence_bundle = EvidenceBundle {
        object_id: track.track_id.to_string(),
        window_start_unix_seconds: Some(track.first_seen_unix_seconds),
        window_end_unix_seconds: Some(track.last_seen_unix_seconds),
        observations: vec![observation.clone()],
        sources: vec![ObservationSource::Api],
        events: vec![IntelligenceEvent {
            event_id: Uuid::new_v4().to_string(),
            object_id: input.site.site.site_id.to_string(),
            event_type: IntelligenceEventType::Assessment,
            strength: EvidenceStrength::ValidatedFact,
            source_id: track.track_id.to_string(),
            timestamp_unix_seconds: Some(
                track.last_seen_unix_seconds + threat_assessment.impact_window_seconds,
            ),
            schema_version: "sss-skyshield-event-v1".to_string(),
            payload_summary: threat_assessment.explanation.clone(),
            causal_parent_ids: vec![track.track_id.to_string()],
            canonical_hash: Uuid::new_v4().to_string(),
        }],
        anomaly_score: score.into(),
        signals: vec![AnomalySignal::PositionDrift],
        derived_features: finding.derived_features.clone(),
        confidence_contributors: vec![ConfidenceContributor {
            name: "site_exposure".to_string(),
            weight: exposure_sensitivity.into(),
            reason: format!(
                "zone criticality={:?}, asset criticality={:?}",
                affected_zone.criticality, affected_asset.criticality
            ),
        }],
        bundle_hash: Uuid::new_v4().to_string(),
    };
    let ranked_event = RankedEvent {
        event_id: Uuid::new_v4().to_string(),
        object_id: input.site.site.site_id.to_string(),
        object_name: input.site.site.name.clone(),
        risk: score.into(),
        target_epoch_unix_seconds: Some(
            track.last_seen_unix_seconds + threat_assessment.impact_window_seconds,
        ),
        event_type: ranked_event_type_for(threat_type),
        prediction: Some(prediction.clone()),
        summary: threat_assessment.explanation.clone(),
    };
    let replay_manifest = ReplayManifest {
        object_id: input.site.site.site_id.to_string(),
        created_at_unix_seconds: Some(track.last_seen_unix_seconds),
        event_ids: vec![ranked_event.event_id.clone()],
        event_fingerprint: format!("{}:{:?}", input.site.site.site_id, threat_type),
        versions: std::collections::BTreeMap::from([
            ("domain".to_string(), "sss-skyshield-v1".to_string()),
            (
                "policy".to_string(),
                "skyshield-response-policy-v1".to_string(),
            ),
        ]),
        manifest_hash: Uuid::new_v4().to_string(),
    };
    let incident = SiteIncident {
        incident_id: Uuid::new_v4(),
        site_id: input.site.site.site_id,
        incident_type: incident_type_for(threat_type),
        status: IncidentStatus::Confirmed,
        created_at_unix_seconds: track.first_seen_unix_seconds,
        updated_at_unix_seconds: track.last_seen_unix_seconds,
        linked_track_ids: vec![track.track_id],
        linked_event_ids: vec![Uuid::new_v4()],
        linked_bundle_hashes: vec![evidence_bundle.bundle_hash.clone()],
        severity,
        current_owner: Some("skyshield-operator".to_string()),
    };
    Some(AssessSiteSkyOutput {
        threat_assessment,
        operational_decision: site_decision,
        incident,
        core: SkyshieldCoreEnvelope {
            finding,
            assessment,
            decision,
            evidence_bundle,
            replay_manifest,
            ranked_event,
        },
    })
}

fn severity_for(score: f32) -> Severity {
    if score >= 0.85 {
        Severity::Critical
    } else if score >= 0.65 {
        Severity::High
    } else if score >= 0.4 {
        Severity::Medium
    } else {
        Severity::Low
    }
}

fn priority_for(score: f32) -> u8 {
    if score >= 0.995 {
        100
    } else if score <= 0.01 {
        1
    } else {
        let normalized = score.clamp(0.0, 1.0);
        let scaled = normalized.mul_add(99.0, 1.0);
        let rounded = scaled.round();
        rounded
            .to_string()
            .parse::<u8>()
            .unwrap_or(100)
            .clamp(1, 100)
    }
}

fn alert_severity_for(severity: Severity) -> AlertSeverity {
    match severity {
        Severity::Low => AlertSeverity::Info,
        Severity::Medium => AlertSeverity::Watch,
        Severity::High => AlertSeverity::Warning,
        Severity::Critical => AlertSeverity::Critical,
    }
}

fn analysis_status_for(score: f32) -> AnalysisStatus {
    if score >= 0.65 {
        AnalysisStatus::AnomalyDetected
    } else if score >= 0.35 {
        AnalysisStatus::Watch
    } else {
        AnalysisStatus::Nominal
    }
}

fn escalation_rules_for(decision: &SiteOperationalDecision) -> Vec<EscalationRule> {
    match decision.escalation_policy {
        crate::EscalationPolicy::MonitorOnly => Vec::new(),
        crate::EscalationPolicy::NotifyAndTrack => vec![EscalationRule {
            rule_id: "notify_and_track".to_string(),
            reason: "site requires defensive monitoring escalation".to_string(),
        }],
        crate::EscalationPolicy::HumanAuthorizationRequired => vec![EscalationRule {
            rule_id: "human_authorization_required".to_string(),
            reason: "defensive response requires operator authorization".to_string(),
        }],
        crate::EscalationPolicy::ExternalAuthorityRequired => vec![EscalationRule {
            rule_id: "external_authority_required".to_string(),
            reason: "external authority visibility is required before defensive dispatch"
                .to_string(),
        }],
    }
}

fn intelligence_recipient_for(recipient: RecipientRole) -> IntelligenceRecipient {
    match recipient {
        RecipientRole::SiteOperator | RecipientRole::FacilityManager => {
            IntelligenceRecipient::Operator
        }
        RecipientRole::SecurityOperationsCenter | RecipientRole::DroneDefenseCoordinator => {
            IntelligenceRecipient::SpaceTrafficCoordination
        }
        RecipientRole::GridOperator => IntelligenceRecipient::SatelliteOwner,
        RecipientRole::LocalAuthority => IntelligenceRecipient::NationalAuthority,
    }
}

fn ranked_event_type_for(threat_type: ThreatType) -> RankedEventType {
    match threat_type {
        ThreatType::UnauthorizedOverflight
        | ThreatType::PerimeterAirspaceViolation
        | ThreatType::HoverOverCriticalZone
        | ThreatType::RepeatPassPattern
        | ThreatType::RfLinkAnomaly
        | ThreatType::SuspectedPayloadRisk
        | ThreatType::MultiDronePattern => RankedEventType::PredictedCloseApproach,
        ThreatType::CoolingAirQualityRisk
        | ThreatType::SmokeIntrusionRisk
        | ThreatType::StormCellApproach
        | ThreatType::HailRisk
        | ThreatType::DustHazeDetected
        | ThreatType::WindLoadRisk
        | ThreatType::LightningRisk
        | ThreatType::IrradianceDropExpected => RankedEventType::UnstableTrack,
        ThreatType::OrbitalReentryContext
        | ThreatType::SpaceWeatherContext
        | ThreatType::OverheadRiskContext => RankedEventType::CoordinationPatternSuspected,
    }
}

fn incident_type_for(threat_type: ThreatType) -> IncidentType {
    match threat_type {
        ThreatType::UnauthorizedOverflight
        | ThreatType::PerimeterAirspaceViolation
        | ThreatType::HoverOverCriticalZone
        | ThreatType::RepeatPassPattern
        | ThreatType::RfLinkAnomaly
        | ThreatType::SuspectedPayloadRisk
        | ThreatType::MultiDronePattern => IncidentType::DroneDefense,
        ThreatType::OrbitalReentryContext
        | ThreatType::SpaceWeatherContext
        | ThreatType::OverheadRiskContext => IncidentType::OrbitalContext,
        _ => IncidentType::Environmental,
    }
}

fn observation_from_track(track: &crate::AerialTrack) -> Observation {
    Observation {
        object_id: track.track_id.to_string(),
        observed_state: track_observation_state(track),
        rf_mhz: None,
        maneuver_detected: false,
        source: ObservationSource::Api,
    }
}

fn track_observation_state(track: &crate::AerialTrack) -> sss_core::OrbitalState {
    sss_core::OrbitalState {
        epoch_unix_seconds: track.last_seen_unix_seconds,
        position_km: Vector3 {
            x: track.current_position.latitude,
            y: track.current_position.longitude,
            z: f64::from(track.altitude_m) / 1000.0,
        },
        velocity_km_s: Vector3 {
            x: f64::from(track.velocity_vector.x),
            y: f64::from(track.velocity_vector.y),
            z: f64::from(track.velocity_vector.z),
        },
    }
}

fn velocity_magnitude_km_s(value: &crate::Vec3) -> f64 {
    f64::from(
        (value
            .x
            .mul_add(value.x, value.y.mul_add(value.y, value.z * value.z)))
        .sqrt(),
    )
}
