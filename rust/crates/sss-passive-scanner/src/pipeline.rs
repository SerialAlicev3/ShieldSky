use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use sss_core::{
    replay_manifest_for, AlertSeverity, AnalysisStatus, AnomalySignal, ConfidenceContributor,
    DerivedFeatures, EscalationRule, EvidenceBundle, EvidenceStrength, IntelligenceAssessment,
    IntelligenceEvent, IntelligenceEventType, IntelligenceRecipient, NotificationDecision,
    Observation, ObservationFinding, ObservationSource, OperationalDecision, OrbitalState,
    PredictedEvent, Prediction, RankedEvent, RankedEventType, RecipientPolicy, RuleHit, Vector3,
};
use sss_site_registry::{Criticality, SiteType};

use crate::correlator::{
    event_matcher::match_events, pattern_extractor::extract_patterns,
    risk_accumulator::build_risk_history, site_builder::build_sites,
};
use crate::domain::{
    PassiveCoreEnvelope, PassiveEvent, PassiveObservation, PassiveScanInput, PassiveScanOutput,
    PassiveSiteProfile, PassiveSourceKind, PassiveThreatType,
};
use crate::sources::{
    adsb_ingest::normalize_adsb, fire_smoke_ingest::normalize_fire_smoke,
    notam_ingest::normalize_notams, orbital_ingest::normalize_orbital,
    satellite_ingest::normalize_satellite, weather_ingest::normalize_weather,
};

#[derive(Debug, Default, Clone)]
pub struct PassiveScanner;

impl PassiveScanner {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn scan(&self, input: &PassiveScanInput) -> PassiveScanOutput {
        let sites = build_sites(&input.infra_sites);
        let observations = normalized_observations(input);
        let events = match_events(&sites, &observations);
        let risk_history = build_risk_history(
            &sites,
            &events,
            input.window_start_unix_seconds,
            input.window_end_unix_seconds,
        );
        let patterns = extract_patterns(&events);
        let core_envelopes = events
            .iter()
            .map(|event| build_core_envelope(event, &sites))
            .collect();

        PassiveScanOutput {
            sites,
            observations,
            events,
            risk_history,
            patterns,
            core_envelopes,
        }
    }
}

fn normalized_observations(input: &PassiveScanInput) -> Vec<PassiveObservation> {
    let mut observations = Vec::new();
    observations.extend(input.adsb_observations.iter().map(normalize_adsb));
    observations.extend(input.weather_observations.iter().map(normalize_weather));
    observations.extend(
        input
            .fire_smoke_observations
            .iter()
            .map(normalize_fire_smoke),
    );
    observations.extend(input.orbital_observations.iter().map(normalize_orbital));
    observations.extend(input.satellite_observations.iter().map(normalize_satellite));
    observations.extend(input.notam_observations.iter().map(normalize_notams));
    observations.sort_by_key(|observation| observation.observed_at_unix_seconds);
    observations
}

fn build_core_envelope(event: &PassiveEvent, sites: &[PassiveSiteProfile]) -> PassiveCoreEnvelope {
    let site = sites
        .iter()
        .find(|site| site.site.site_id == event.site_id)
        .expect("matched event always has a site");
    let object_id = format!("passive:{}:{}", event.site_id, event.event_id);
    let signal = signal_for_threat(event.threat_type);
    let observation = observation_for(event, site, &object_id);
    let derived_features = derived_features_for(event, &observation);
    let mut evidence_bundle = evidence_bundle_for(
        event,
        site,
        &object_id,
        &observation,
        &derived_features,
        &signal,
    );
    evidence_bundle.bundle_hash = evidence_bundle_hash(&evidence_bundle);
    let finding = finding_for(event, &object_id, &derived_features, &signal);
    let prediction = prediction_for(event, site, &object_id, &observation, &derived_features);
    let assessment = assessment_for(event, site, &prediction);
    let decision = decision_for(event, site);

    let ranked_event = RankedEvent {
        event_id: event.event_id.to_string(),
        object_id,
        object_name: site.site.name.clone(),
        risk: event.risk_score,
        target_epoch_unix_seconds: Some(event.observed_at_unix_seconds),
        event_type: ranked_event_type(event.threat_type),
        prediction: Some(prediction),
        summary: event.summary.clone(),
    };

    let replay_manifest = replay_manifest_for(&evidence_bundle);

    PassiveCoreEnvelope {
        finding,
        assessment,
        decision,
        evidence_bundle,
        replay_manifest,
        ranked_event,
    }
}

fn observation_for(
    event: &PassiveEvent,
    site: &PassiveSiteProfile,
    object_id: &str,
) -> Observation {
    Observation {
        object_id: object_id.to_string(),
        observed_state: OrbitalState {
            epoch_unix_seconds: event.observed_at_unix_seconds,
            position_km: geo_to_vector(
                event.payload["latitude"]
                    .as_f64()
                    .unwrap_or(site.site.latitude),
                event.payload["longitude"]
                    .as_f64()
                    .unwrap_or(site.site.longitude),
                event.payload["altitude_m"]
                    .as_f64()
                    .unwrap_or(f64::from(site.site.elevation_m)),
            ),
            velocity_km_s: Vector3 {
                x: event.risk_score,
                y: event.distance_to_site_km / 10.0,
                z: 0.0,
            },
        },
        rf_mhz: Some(source_rf_hint(event.source_kind)),
        maneuver_detected: matches!(
            event.threat_type,
            PassiveThreatType::LowAltitudeOverflight
                | PassiveThreatType::RestrictedAirspaceActivity
                | PassiveThreatType::OrbitalReentryContext
        ),
        source: source_to_observation_source(event.source_kind),
    }
}

fn derived_features_for(event: &PassiveEvent, observation: &Observation) -> DerivedFeatures {
    DerivedFeatures {
        position_delta_km: event.distance_to_site_km,
        velocity_delta_km_s: event.risk_score * 4.0,
        rf_delta_mhz: Some((1.0 - event.risk_score) * 12.5),
        maneuver_detected: observation.maneuver_detected,
    }
}

fn evidence_bundle_for(
    event: &PassiveEvent,
    site: &PassiveSiteProfile,
    object_id: &str,
    observation: &Observation,
    derived_features: &DerivedFeatures,
    signal: &AnomalySignal,
) -> EvidenceBundle {
    let source = source_to_observation_source(event.source_kind);
    let evidence_events = vec![
        intelligence_event(
            object_id,
            IntelligenceEventType::Observation,
            EvidenceStrength::ObservedFact,
            &source,
            event.observed_at_unix_seconds,
            event.summary.clone(),
            &[],
        ),
        intelligence_event(
            object_id,
            IntelligenceEventType::Assessment,
            EvidenceStrength::InferredContext,
            &ObservationSource::Api,
            event.observed_at_unix_seconds,
            format!(
                "passive correlation raised {:?} for {}",
                event.threat_type, site.site.name
            ),
            &[format!("obs:{}", event.event_id)],
        ),
    ];

    EvidenceBundle {
        object_id: object_id.to_string(),
        window_start_unix_seconds: Some(event.observed_at_unix_seconds),
        window_end_unix_seconds: Some(event.observed_at_unix_seconds),
        observations: vec![observation.clone()],
        sources: vec![source],
        events: evidence_events,
        anomaly_score: event.risk_score,
        signals: vec![signal.clone()],
        derived_features: derived_features.clone(),
        confidence_contributors: confidence_contributors(event, site),
        bundle_hash: String::new(),
    }
}

fn confidence_contributors(
    event: &PassiveEvent,
    site: &PassiveSiteProfile,
) -> Vec<ConfidenceContributor> {
    vec![
        ConfidenceContributor {
            name: "open_source_correlation".to_string(),
            weight: 0.35,
            reason: format!(
                "{:?} signal correlated against {} passive site profile",
                event.source_kind, site.site.name
            ),
        },
        ConfidenceContributor {
            name: "site_proximity".to_string(),
            weight: 0.25,
            reason: format!(
                "matched within {:.2} km of {}",
                event.distance_to_site_km, site.site.name
            ),
        },
        ConfidenceContributor {
            name: "site_criticality".to_string(),
            weight: criticality_weight(site.site.criticality),
            reason: format!("site criticality is {:?}", site.site.criticality),
        },
    ]
}

fn finding_for(
    event: &PassiveEvent,
    object_id: &str,
    derived_features: &DerivedFeatures,
    signal: &AnomalySignal,
) -> ObservationFinding {
    ObservationFinding {
        object_id: object_id.to_string(),
        observed_at_unix_seconds: Some(event.observed_at_unix_seconds),
        source: Some(source_to_observation_source(event.source_kind)),
        anomaly_score: event.risk_score,
        signals: vec![signal.clone()],
        derived_features: derived_features.clone(),
        summary: event.summary.clone(),
    }
}

fn prediction_for(
    event: &PassiveEvent,
    site: &PassiveSiteProfile,
    object_id: &str,
    observation: &Observation,
    derived_features: &DerivedFeatures,
) -> Prediction {
    Prediction {
        object_id: object_id.to_string(),
        horizon_hours: prediction_horizon_hours(event.threat_type),
        event: predicted_event_for(event.threat_type),
        probability: event.risk_score,
        explanation: format!(
            "passive observation suggests {:?} remains relevant for {}",
            event.threat_type, site.site.name
        ),
        propagation_model: "sss-passive-heuristic-v1".to_string(),
        based_on_observation_epoch: Some(event.observed_at_unix_seconds),
        predicted_state: Some(observation.observed_state.clone()),
        position_delta_km: Some(event.distance_to_site_km),
        velocity_delta_km_s: Some(derived_features.velocity_delta_km_s),
    }
}

fn assessment_for(
    event: &PassiveEvent,
    site: &PassiveSiteProfile,
    prediction: &Prediction,
) -> IntelligenceAssessment {
    IntelligenceAssessment {
        status: status_for_risk(event.risk_score),
        behavior: behavior_label(event.threat_type).to_string(),
        confidence: (0.45 + (event.risk_score * 0.45)).min(0.96),
        prediction: prediction.clone(),
        explanation: format!(
            "Passive scanner correlated {:?} against site {} using open-source context.",
            event.threat_type, site.site.name
        ),
        decision_version: "sss-passive-decision-v1".to_string(),
        model_version: "sss-passive-model-v1".to_string(),
        rule_hits: vec![
            RuleHit {
                rule_id: "passive_site_match".to_string(),
                description: "observation intersects the passive observation radius".to_string(),
            },
            RuleHit {
                rule_id: format!("threat_{:?}", event.threat_type).to_lowercase(),
                description: event.summary.clone(),
            },
        ],
        signal_summary: vec![
            format!("{:?} source", event.source_kind),
            format!("{:?}", event.threat_type),
        ],
    }
}

fn decision_for(event: &PassiveEvent, site: &PassiveSiteProfile) -> OperationalDecision {
    OperationalDecision {
        severity: severity_for_risk(event.risk_score),
        risk_score: event.risk_score,
        recommendation: recommendation_for(event, site.site.site_type),
        recipient_policy: RecipientPolicy {
            policy_id: "sss-passive-recipients-v1".to_string(),
            escalation_rules: escalation_rules(site.site.criticality, event.risk_score),
        },
        notifications: recipients(site.site.criticality, event.risk_score, event.threat_type),
    }
}

fn geo_to_vector(latitude: f64, longitude: f64, altitude_m: f64) -> Vector3 {
    let earth_radius_km = 6_371.0 + (altitude_m / 1_000.0);
    let latitude_radians = latitude.to_radians();
    let longitude_radians = longitude.to_radians();
    let cos_latitude = latitude_radians.cos();

    Vector3 {
        x: earth_radius_km * cos_latitude * longitude_radians.cos(),
        y: earth_radius_km * cos_latitude * longitude_radians.sin(),
        z: earth_radius_km * latitude_radians.sin(),
    }
}

fn source_rf_hint(source_kind: PassiveSourceKind) -> f64 {
    match source_kind {
        PassiveSourceKind::Adsb => 1_090.0,
        PassiveSourceKind::Orbital => 437.0,
        PassiveSourceKind::Notam => 121.5,
        PassiveSourceKind::Weather => 5_600.0,
        PassiveSourceKind::FireSmoke => 1_400.0,
        PassiveSourceKind::Satellite => 8_200.0,
        PassiveSourceKind::InfraMap => 915.0,
    }
}

fn source_to_observation_source(source_kind: PassiveSourceKind) -> ObservationSource {
    match source_kind {
        PassiveSourceKind::Orbital | PassiveSourceKind::InfraMap => ObservationSource::Catalog,
        PassiveSourceKind::Adsb
        | PassiveSourceKind::Satellite
        | PassiveSourceKind::Weather
        | PassiveSourceKind::FireSmoke
        | PassiveSourceKind::Notam => ObservationSource::Api,
    }
}

fn signal_for_threat(threat_type: PassiveThreatType) -> AnomalySignal {
    match threat_type {
        PassiveThreatType::LowAltitudeOverflight => AnomalySignal::VelocityChange,
        PassiveThreatType::RestrictedAirspaceActivity => AnomalySignal::RadioFrequencyShift,
        PassiveThreatType::HailExposure
        | PassiveThreatType::WindLoadRisk
        | PassiveThreatType::IrradianceDropExpected
        | PassiveThreatType::SmokeExposure
        | PassiveThreatType::SurfaceChangeDetected => AnomalySignal::PositionDrift,
        PassiveThreatType::OrbitalReentryContext => AnomalySignal::UnexpectedManeuver,
    }
}

fn prediction_horizon_hours(threat_type: PassiveThreatType) -> f64 {
    match threat_type {
        PassiveThreatType::LowAltitudeOverflight => 1.0,
        PassiveThreatType::RestrictedAirspaceActivity => 4.0,
        PassiveThreatType::HailExposure | PassiveThreatType::WindLoadRisk => 6.0,
        PassiveThreatType::IrradianceDropExpected | PassiveThreatType::SmokeExposure => 8.0,
        PassiveThreatType::OrbitalReentryContext => 12.0,
        PassiveThreatType::SurfaceChangeDetected => 24.0,
    }
}

fn predicted_event_for(threat_type: PassiveThreatType) -> PredictedEvent {
    match threat_type {
        PassiveThreatType::LowAltitudeOverflight
        | PassiveThreatType::RestrictedAirspaceActivity
        | PassiveThreatType::OrbitalReentryContext => PredictedEvent::CloseApproach,
        PassiveThreatType::SurfaceChangeDetected => PredictedEvent::UnstableTrack,
        PassiveThreatType::HailExposure
        | PassiveThreatType::WindLoadRisk
        | PassiveThreatType::IrradianceDropExpected
        | PassiveThreatType::SmokeExposure => PredictedEvent::ContinuedAnomalousBehavior,
    }
}

fn status_for_risk(risk: f64) -> AnalysisStatus {
    if risk >= 0.65 {
        AnalysisStatus::AnomalyDetected
    } else if risk >= 0.4 {
        AnalysisStatus::Watch
    } else {
        AnalysisStatus::Nominal
    }
}

fn severity_for_risk(risk: f64) -> AlertSeverity {
    if risk >= 0.85 {
        AlertSeverity::Critical
    } else if risk >= 0.65 {
        AlertSeverity::Warning
    } else if risk >= 0.4 {
        AlertSeverity::Watch
    } else {
        AlertSeverity::Info
    }
}

fn behavior_label(threat_type: PassiveThreatType) -> &'static str {
    match threat_type {
        PassiveThreatType::LowAltitudeOverflight => "persistent low-altitude aerial activity",
        PassiveThreatType::RestrictedAirspaceActivity => "restricted airspace activity",
        PassiveThreatType::HailExposure => "hail exposure build-up",
        PassiveThreatType::WindLoadRisk => "wind-load pressure increase",
        PassiveThreatType::IrradianceDropExpected => "irradiance degradation pattern",
        PassiveThreatType::SmokeExposure => "smoke plume exposure",
        PassiveThreatType::OrbitalReentryContext => "orbital reentry context",
        PassiveThreatType::SurfaceChangeDetected => "surface change pattern",
    }
}

fn recommendation_for(event: &PassiveEvent, site_type: SiteType) -> String {
    match event.threat_type {
        PassiveThreatType::LowAltitudeOverflight => format!(
            "Review aerial activity near the {site_type:?} site and keep operator console on watch."
        ),
        PassiveThreatType::RestrictedAirspaceActivity => {
            "Cross-check NOTAM and airspace restrictions for this site immediately.".to_string()
        }
        PassiveThreatType::HailExposure => {
            "Prepare weather hardening actions for exposed assets in the next window.".to_string()
        }
        PassiveThreatType::WindLoadRisk => {
            "Verify vulnerable rooftop or perimeter assets against wind thresholds.".to_string()
        }
        PassiveThreatType::IrradianceDropExpected => {
            "Brief operations that power output may drift below expected baseline.".to_string()
        }
        PassiveThreatType::SmokeExposure => {
            "Assess smoke path and readiness of cooling or panel cleaning workflows.".to_string()
        }
        PassiveThreatType::OrbitalReentryContext => {
            "Maintain contextual watch and elevate if corroborated by authoritative feeds."
                .to_string()
        }
        PassiveThreatType::SurfaceChangeDetected => {
            "Review latest site imagery for construction or degradation around protected assets."
                .to_string()
        }
    }
}

fn escalation_rules(criticality: Criticality, risk: f64) -> Vec<EscalationRule> {
    let mut rules = Vec::new();
    if risk >= 0.4 {
        rules.push(EscalationRule {
            rule_id: "passive_watch_threshold".to_string(),
            reason: "site moved above passive watch threshold".to_string(),
        });
    }
    if risk >= 0.7 || matches!(criticality, Criticality::Critical) {
        rules.push(EscalationRule {
            rule_id: "critical_infrastructure_attention".to_string(),
            reason: "site criticality or risk justifies operational review".to_string(),
        });
    }
    rules
}

fn recipients(
    criticality: Criticality,
    risk: f64,
    threat_type: PassiveThreatType,
) -> Vec<NotificationDecision> {
    let mut notifications = vec![NotificationDecision {
        recipient: IntelligenceRecipient::Operator,
        notification_reason: "site operator consumes passive intelligence by default".to_string(),
    }];

    if risk >= 0.65 {
        notifications.push(NotificationDecision {
            recipient: IntelligenceRecipient::InsuranceRiskDesk,
            notification_reason: "elevated passive risk contributes to underwriting context"
                .to_string(),
        });
    }

    if risk >= 0.8
        || matches!(criticality, Criticality::Critical)
        || matches!(threat_type, PassiveThreatType::RestrictedAirspaceActivity)
    {
        notifications.push(NotificationDecision {
            recipient: IntelligenceRecipient::NationalAuthority,
            notification_reason:
                "critical infrastructure or restricted-airspace context may require authority awareness"
                    .to_string(),
        });
    }

    notifications
}

fn ranked_event_type(threat_type: PassiveThreatType) -> RankedEventType {
    match threat_type {
        PassiveThreatType::LowAltitudeOverflight
        | PassiveThreatType::RestrictedAirspaceActivity
        | PassiveThreatType::OrbitalReentryContext => RankedEventType::PredictedCloseApproach,
        PassiveThreatType::SurfaceChangeDetected => RankedEventType::UnstableTrack,
        PassiveThreatType::HailExposure
        | PassiveThreatType::WindLoadRisk
        | PassiveThreatType::IrradianceDropExpected
        | PassiveThreatType::SmokeExposure => RankedEventType::BehaviorShiftDetected,
    }
}

fn criticality_weight(criticality: Criticality) -> f64 {
    match criticality {
        Criticality::Low => 0.25,
        Criticality::Medium => 0.5,
        Criticality::High => 0.75,
        Criticality::Critical => 1.0,
    }
}

fn intelligence_event(
    object_id: &str,
    event_type: IntelligenceEventType,
    strength: EvidenceStrength,
    source: &ObservationSource,
    timestamp_unix_seconds: i64,
    payload_summary: String,
    parents: &[String],
) -> IntelligenceEvent {
    let event_id = match event_type {
        IntelligenceEventType::Observation => format!("obs:{object_id}:{timestamp_unix_seconds}"),
        IntelligenceEventType::Assessment => format!("assess:{object_id}:{timestamp_unix_seconds}"),
        IntelligenceEventType::Validation => {
            format!("validate:{object_id}:{timestamp_unix_seconds}")
        }
        IntelligenceEventType::Policy => format!("policy:{object_id}:{timestamp_unix_seconds}"),
        IntelligenceEventType::Decision => format!("decision:{object_id}:{timestamp_unix_seconds}"),
        IntelligenceEventType::Notification => {
            format!("notify:{object_id}:{timestamp_unix_seconds}")
        }
        IntelligenceEventType::Outcome => format!("outcome:{object_id}:{timestamp_unix_seconds}"),
        IntelligenceEventType::Interpretation => {
            format!("interpret:{object_id}:{timestamp_unix_seconds}")
        }
    };
    let source_id = match source {
        ObservationSource::Radar => "radar",
        ObservationSource::Telescope => "telescope",
        ObservationSource::RadioFrequency => "rf",
        ObservationSource::Catalog => "catalog",
        ObservationSource::Api => "api",
        ObservationSource::Synthetic => "synthetic",
    }
    .to_string();
    let canonical_hash = stable_id(&format!(
        "{event_id}:{event_type:?}:{strength:?}:{source_id}:{timestamp_unix_seconds:?}:{payload_summary}"
    ));

    IntelligenceEvent {
        event_id,
        object_id: object_id.to_string(),
        event_type,
        strength,
        source_id,
        timestamp_unix_seconds: Some(timestamp_unix_seconds),
        schema_version: "sss-passive-event-v1".to_string(),
        payload_summary,
        causal_parent_ids: parents.to_vec(),
        canonical_hash,
    }
}

fn evidence_bundle_hash(bundle: &EvidenceBundle) -> String {
    let event_hashes = bundle
        .events
        .iter()
        .map(|event| event.canonical_hash.as_str())
        .collect::<Vec<_>>()
        .join("|");

    stable_id(&format!(
        "{}:{:?}:{:?}:{:?}:{:.6}:{:.6}:{:.6}:{:?}:{}",
        bundle.object_id,
        bundle.window_start_unix_seconds,
        bundle.window_end_unix_seconds,
        bundle.sources,
        bundle.anomaly_score,
        bundle.derived_features.position_delta_km,
        bundle.derived_features.velocity_delta_km_s,
        bundle.derived_features.rf_delta_mhz,
        event_hashes
    ))
}

fn stable_id(value: &str) -> String {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
