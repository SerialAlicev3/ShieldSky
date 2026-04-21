//! Serial Alice Sky core intelligence primitives.
//!
//! This crate is intentionally small: it gives the MVP a deterministic
//! behavioral twin, simple anomaly scoring, and first-pass anticipation.

use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};
use std::ops::{Add, Sub};
use std::time::Duration;

use serde::{Deserialize, Serialize};

const EARTH_RADIUS_KM: f64 = 6_371.0;
const CLOSE_APPROACH_KM: f64 = 25.0;
const OVERVIEW_CLOSE_APPROACH_KM: f64 = 250.0;
const DEFAULT_WINDOW_HOURS: f64 = 72.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrbitalState {
    pub epoch_unix_seconds: i64,
    pub position_km: Vector3,
    pub velocity_km_s: Vector3,
}

impl OrbitalState {
    #[must_use]
    pub fn propagate_linear_seconds(&self, horizon_seconds: i64) -> Self {
        Self {
            epoch_unix_seconds: self.epoch_unix_seconds + horizon_seconds,
            position_km: self.position_km
                + self.velocity_km_s.scale(seconds_to_f64(horizon_seconds)),
            velocity_km_s: self.velocity_km_s,
        }
    }

    #[must_use]
    pub fn altitude_km(&self) -> f64 {
        (self.position_km.magnitude() - EARTH_RADIUS_KM).max(0.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    #[must_use]
    pub fn magnitude(self) -> f64 {
        self.dot(self).sqrt()
    }

    #[must_use]
    pub fn distance_to(self, other: Self) -> f64 {
        (self - other).magnitude()
    }

    #[must_use]
    pub fn dot(self, other: Self) -> f64 {
        self.x
            .mul_add(other.x, self.y.mul_add(other.y, self.z * other.z))
    }

    #[must_use]
    pub fn scale(self, scalar: f64) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl Add for Vector3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Vector3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrbitRegime {
    Leo,
    Meo,
    Geo,
    DeepSpace,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BehaviorSignature {
    pub nominal_rf_mhz: Option<f64>,
    pub historical_maneuvers_per_week: f64,
    pub station_keeping_expected: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskProfile {
    pub operator: Option<String>,
    pub mission_class: MissionClass,
    pub criticality: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionClass {
    Civil,
    Commercial,
    Defense,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpaceObject {
    pub id: String,
    pub name: String,
    pub regime: OrbitRegime,
    pub state: OrbitalState,
    #[serde(default)]
    pub tle: Option<TleEphemeris>,
    pub behavior: BehaviorSignature,
    pub risk: RiskProfile,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TleEphemeris {
    pub line1: String,
    pub line2: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    pub object_id: String,
    pub observed_state: OrbitalState,
    pub rf_mhz: Option<f64>,
    pub maneuver_detected: bool,
    pub source: ObservationSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObservationSource {
    Radar,
    Telescope,
    RadioFrequency,
    Catalog,
    Api,
    Synthetic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceStrength {
    ObservedFact,
    ValidatedFact,
    InferredContext,
    PolicyContext,
    ProposedAction,
    ExecutedAction,
    MeasuredOutcome,
    EstimatedOutcome,
    WeakClaim,
}

impl EvidenceStrength {
    #[must_use]
    pub fn rank(&self) -> u8 {
        match self {
            Self::ObservedFact => 9,
            Self::ValidatedFact => 8,
            Self::ExecutedAction => 7,
            Self::MeasuredOutcome => 6,
            Self::InferredContext => 5,
            Self::PolicyContext => 4,
            Self::ProposedAction => 3,
            Self::EstimatedOutcome => 2,
            Self::WeakClaim => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntelligenceEventType {
    Observation,
    Validation,
    Interpretation,
    Policy,
    Assessment,
    Decision,
    Notification,
    Outcome,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntelligenceEvent {
    pub event_id: String,
    pub object_id: String,
    pub event_type: IntelligenceEventType,
    pub strength: EvidenceStrength,
    pub source_id: String,
    pub timestamp_unix_seconds: Option<i64>,
    pub schema_version: String,
    pub payload_summary: String,
    pub causal_parent_ids: Vec<String>,
    pub canonical_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DigitalTwin {
    objects: BTreeMap<String, SpaceObject>,
}

impl DigitalTwin {
    #[must_use]
    pub fn new(objects: Vec<SpaceObject>) -> Self {
        Self {
            objects: objects
                .into_iter()
                .map(|object| (object.id.clone(), object))
                .collect(),
        }
    }

    #[must_use]
    pub fn object(&self, object_id: &str) -> Option<&SpaceObject> {
        self.objects.get(object_id)
    }

    pub fn upsert_object(&mut self, object: SpaceObject) {
        self.objects.insert(object.id.clone(), object);
    }

    pub fn ingest_observation(&mut self, observation: &Observation) -> Option<SpaceObject> {
        let object = self.objects.get_mut(&observation.object_id)?;
        object.state = observation.observed_state.clone();
        Some(object.clone())
    }

    #[must_use]
    pub fn predicted_state(
        &self,
        object_id: &str,
        at_epoch_unix_seconds: i64,
    ) -> Option<OrbitalState> {
        let object = self.objects.get(object_id)?;
        propagate_object_state(object, at_epoch_unix_seconds).or_else(|| {
            let horizon = at_epoch_unix_seconds - object.state.epoch_unix_seconds;
            Some(object.state.propagate_linear_seconds(horizon))
        })
    }

    pub fn objects(&self) -> impl Iterator<Item = &SpaceObject> {
        self.objects.values()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnomalyReport {
    pub object_id: String,
    pub score: f64,
    pub position_delta_km: f64,
    pub velocity_delta_km_s: f64,
    pub rf_delta_mhz: Option<f64>,
    pub signals: Vec<AnomalySignal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalySignal {
    UnexpectedManeuver,
    PositionDrift,
    VelocityChange,
    RadioFrequencyShift,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnomalyEngine {
    pub position_threshold_km: f64,
    pub velocity_threshold_km_s: f64,
    pub rf_threshold_mhz: f64,
}

impl Default for AnomalyEngine {
    fn default() -> Self {
        Self {
            position_threshold_km: 10.0,
            velocity_threshold_km_s: 0.02,
            rf_threshold_mhz: 0.5,
        }
    }
}

impl AnomalyEngine {
    #[must_use]
    pub fn evaluate(&self, object: &SpaceObject, observation: &Observation) -> AnomalyReport {
        let seconds =
            observation.observed_state.epoch_unix_seconds - object.state.epoch_unix_seconds;
        let expected = object.state.propagate_linear_seconds(seconds);
        let position_delta_km = observation
            .observed_state
            .position_km
            .distance_to(expected.position_km);
        let velocity_delta_km_s = observation
            .observed_state
            .velocity_km_s
            .distance_to(expected.velocity_km_s);
        let rf_delta_mhz = object
            .behavior
            .nominal_rf_mhz
            .zip(observation.rf_mhz)
            .map(|(nominal, observed)| (observed - nominal).abs());

        let mut signals = Vec::new();
        let mut score: f64 = 0.0;

        if position_delta_km > self.position_threshold_km {
            signals.push(AnomalySignal::PositionDrift);
            score += (position_delta_km / self.position_threshold_km).min(3.0) * 0.25;
        }
        if velocity_delta_km_s > self.velocity_threshold_km_s {
            signals.push(AnomalySignal::VelocityChange);
            score += (velocity_delta_km_s / self.velocity_threshold_km_s).min(3.0) * 0.25;
        }
        if rf_delta_mhz.is_some_and(|delta| delta > self.rf_threshold_mhz) {
            signals.push(AnomalySignal::RadioFrequencyShift);
            score += 0.2;
        }
        if observation.maneuver_detected && object.behavior.historical_maneuvers_per_week < 1.0 {
            signals.push(AnomalySignal::UnexpectedManeuver);
            score += 0.3;
        }

        AnomalyReport {
            object_id: observation.object_id.clone(),
            score: score.min(1.0),
            position_delta_km,
            velocity_delta_km_s,
            rf_delta_mhz,
            signals,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prediction {
    pub object_id: String,
    pub horizon_hours: f64,
    pub event: PredictedEvent,
    pub probability: f64,
    pub explanation: String,
    pub propagation_model: String,
    pub based_on_observation_epoch: Option<i64>,
    pub predicted_state: Option<OrbitalState>,
    pub position_delta_km: Option<f64>,
    pub velocity_delta_km_s: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredictedEvent {
    ContinuedAnomalousBehavior,
    CloseApproach,
    UnstableTrack,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionAlert {
    pub object_id: String,
    pub severity: AlertSeverity,
    pub risk_score: f64,
    pub summary: String,
    pub recommendation: String,
    pub predictions: Vec<Prediction>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Watch,
    Warning,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalyzeObjectRequest {
    pub object_id: String,
    pub timestamp_unix_seconds: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalyzeObjectResponse {
    pub object_id: String,
    pub status: AnalysisStatus,
    pub anomaly_score: f64,
    pub behavior: String,
    pub confidence: f64,
    pub prediction: Prediction,
    pub explanation: String,
    pub risk_score: f64,
    pub severity: AlertSeverity,
    pub recipients: Vec<IntelligenceRecipient>,
    pub finding: ObservationFinding,
    pub assessment: IntelligenceAssessment,
    pub decision: OperationalDecision,
    pub evidence_bundle: EvidenceBundle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisStatus {
    Nominal,
    Watch,
    AnomalyDetected,
    UnknownObject,
    NoRecentObservation,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IntelligenceRecipient {
    Operator,
    SatelliteOwner,
    SpaceTrafficCoordination,
    NationalAuthority,
    InsuranceRiskDesk,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpaceOverviewRequest {
    pub window_hours: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpaceOverviewResponse {
    pub window_hours: f64,
    pub top_events: Vec<RankedEvent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RankedEvent {
    pub event_id: String,
    pub object_id: String,
    pub object_name: String,
    pub risk: f64,
    pub target_epoch_unix_seconds: Option<i64>,
    pub event_type: RankedEventType,
    pub prediction: Option<Prediction>,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RankedEventType {
    BehaviorShiftDetected,
    PredictedCloseApproach,
    CoordinationPatternSuspected,
    UnstableTrack,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictRequest {
    pub object_id: String,
    pub horizon_hours: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictResponse {
    pub object_id: String,
    pub predictions: Vec<Prediction>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineRequest {
    pub object_id: String,
    pub horizon_hours: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectTimelineResponse {
    pub object_id: String,
    pub base_epoch_unix_seconds: i64,
    pub checkpoints: Vec<TimelineCheckpoint>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineCheckpoint {
    pub offset_hours: f64,
    pub target_epoch_unix_seconds: i64,
    pub propagation_model: String,
    pub predicted_state: OrbitalState,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservationFinding {
    pub object_id: String,
    pub observed_at_unix_seconds: Option<i64>,
    pub source: Option<ObservationSource>,
    pub anomaly_score: f64,
    pub signals: Vec<AnomalySignal>,
    pub derived_features: DerivedFeatures,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DerivedFeatures {
    pub position_delta_km: f64,
    pub velocity_delta_km_s: f64,
    pub rf_delta_mhz: Option<f64>,
    pub maneuver_detected: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceBundle {
    pub object_id: String,
    pub window_start_unix_seconds: Option<i64>,
    pub window_end_unix_seconds: Option<i64>,
    pub observations: Vec<Observation>,
    pub sources: Vec<ObservationSource>,
    pub events: Vec<IntelligenceEvent>,
    pub anomaly_score: f64,
    pub signals: Vec<AnomalySignal>,
    pub derived_features: DerivedFeatures,
    pub confidence_contributors: Vec<ConfidenceContributor>,
    pub bundle_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfidenceContributor {
    pub name: String,
    pub weight: f64,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntelligenceAssessment {
    pub status: AnalysisStatus,
    pub behavior: String,
    pub confidence: f64,
    pub prediction: Prediction,
    pub explanation: String,
    pub decision_version: String,
    pub model_version: String,
    pub rule_hits: Vec<RuleHit>,
    pub signal_summary: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayManifest {
    pub object_id: String,
    pub created_at_unix_seconds: Option<i64>,
    pub event_ids: Vec<String>,
    pub event_fingerprint: String,
    pub versions: BTreeMap<String, String>,
    pub manifest_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayExecution {
    pub manifest: ReplayManifest,
    pub evidence_bundle: EvidenceBundle,
    pub original_assessment: Option<IntelligenceAssessment>,
    pub replayed_assessment: IntelligenceAssessment,
    pub original_decision: Option<OperationalDecision>,
    pub replayed_decision: OperationalDecision,
    pub diff: ReplayDiff,
    pub drift_detected: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayDiff {
    pub assessment: AssessmentDiff,
    pub decision: DecisionDiff,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssessmentDiff {
    pub changed_fields: Vec<String>,
    pub confidence_delta: f64,
    pub prediction_probability_delta: f64,
    pub added_rule_hits: Vec<String>,
    pub removed_rule_hits: Vec<String>,
    pub added_signals: Vec<String>,
    pub removed_signals: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionDiff {
    pub changed_fields: Vec<String>,
    pub risk_score_delta: Option<f64>,
    pub added_recipients: Vec<IntelligenceRecipient>,
    pub removed_recipients: Vec<IntelligenceRecipient>,
    pub added_escalation_rules: Vec<String>,
    pub removed_escalation_rules: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleHit {
    pub rule_id: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationalDecision {
    pub severity: AlertSeverity,
    pub risk_score: f64,
    pub recommendation: String,
    pub recipient_policy: RecipientPolicy,
    pub notifications: Vec<NotificationDecision>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipientPolicy {
    pub policy_id: String,
    pub escalation_rules: Vec<EscalationRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EscalationRule {
    pub rule_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationDecision {
    pub recipient: IntelligenceRecipient,
    pub notification_reason: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnticipationEngine {
    anomaly_engine: AnomalyEngine,
    horizon_hours: f64,
}

impl Default for AnticipationEngine {
    fn default() -> Self {
        Self {
            anomaly_engine: AnomalyEngine::default(),
            horizon_hours: DEFAULT_WINDOW_HOURS,
        }
    }
}

impl AnticipationEngine {
    #[must_use]
    pub fn new(anomaly_engine: AnomalyEngine, horizon_hours: f64) -> Self {
        Self {
            anomaly_engine,
            horizon_hours,
        }
    }

    #[must_use]
    pub fn evaluate_observation(
        &self,
        twin: &DigitalTwin,
        observation: &Observation,
    ) -> Option<DecisionAlert> {
        let object = twin.object(&observation.object_id)?;
        let anomaly = self.anomaly_engine.evaluate(object, observation);
        let mut predictions = vec![build_prediction(
            twin,
            object,
            Some(observation),
            Some(&anomaly),
            self.horizon_hours,
            PredictedEvent::ContinuedAnomalousBehavior,
            probability_from_anomaly(anomaly.score),
            explain_anomaly(&anomaly),
        )];

        if let Some(close_approach_prediction) =
            predicted_pairwise_close_approach(twin, object, observation, self.horizon_hours)
        {
            predictions.push(close_approach_prediction);
        } else if observation.observed_state.altitude_km() < CLOSE_APPROACH_KM {
            predictions.push(build_prediction(
                twin,
                object,
                Some(observation),
                Some(&anomaly),
                self.horizon_hours,
                PredictedEvent::CloseApproach,
                0.72,
                "track is near the atmospheric boundary used by the MVP safety model".to_string(),
            ));
        }

        if let Some(projected_state) =
            projected_state_for_prediction(twin, object, observation, self.horizon_hours)
        {
            if projected_state.altitude_km() < 200.0 {
                predictions.push(build_prediction(
                    twin,
                    object,
                    Some(observation),
                    Some(&anomaly),
                    self.horizon_hours,
                    PredictedEvent::UnstableTrack,
                    (probability_from_anomaly(anomaly.score) + 0.18).min(0.97),
                    format!(
                        "projected altitude falls to {:.1} km within {:.1} h",
                        projected_state.altitude_km(),
                        self.horizon_hours
                    ),
                ));
            }
        }

        let risk_score = (anomaly.score * 0.7 + object.risk.criticality * 0.3).min(1.0);
        let severity = severity_for_risk(risk_score);

        Some(DecisionAlert {
            object_id: object.id.clone(),
            severity,
            risk_score,
            summary: format!(
                "{} risk={:.2}, anomaly={:.2}, signals={}",
                object.name,
                risk_score,
                anomaly.score,
                anomaly.signals.len()
            ),
            recommendation: recommendation_for_risk(risk_score).to_string(),
            predictions,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntelligenceLayer {
    twin: DigitalTwin,
    anticipation_engine: AnticipationEngine,
    recent_observations: BTreeMap<String, Observation>,
}

impl IntelligenceLayer {
    #[must_use]
    pub fn new(twin: DigitalTwin, anticipation_engine: AnticipationEngine) -> Self {
        Self {
            twin,
            anticipation_engine,
            recent_observations: BTreeMap::new(),
        }
    }

    pub fn record_observation(&mut self, observation: Observation) {
        self.recent_observations
            .insert(observation.object_id.clone(), observation);
    }

    pub fn upsert_object(&mut self, object: SpaceObject) {
        self.twin.upsert_object(object);
    }

    #[must_use]
    pub fn object(&self, object_id: &str) -> Option<SpaceObject> {
        self.twin.object(object_id).cloned()
    }

    #[must_use]
    pub fn objects(&self) -> Vec<SpaceObject> {
        self.twin.objects().cloned().collect()
    }

    #[must_use]
    pub fn predicted_state(
        &self,
        object_id: &str,
        at_epoch_unix_seconds: i64,
    ) -> Option<OrbitalState> {
        self.twin.predicted_state(object_id, at_epoch_unix_seconds)
    }

    #[must_use]
    pub fn analyze_object(&self, request: AnalyzeObjectRequest) -> AnalyzeObjectResponse {
        let Some(object) = self.twin.object(&request.object_id) else {
            return unknown_object_response(request.object_id);
        };

        let Some(observation) = self.recent_observations.get(&request.object_id) else {
            return no_recent_observation_response(object, self.anticipation_engine.horizon_hours);
        };

        let anomaly = self
            .anticipation_engine
            .anomaly_engine
            .evaluate(object, observation);
        let alert = self
            .anticipation_engine
            .evaluate_observation(&self.twin, observation)
            .expect("object was already resolved");
        analyzed_object_response(
            object,
            observation,
            &anomaly,
            &alert,
            self.anticipation_engine.horizon_hours,
        )
    }

    #[must_use]
    pub fn predict(&self, request: PredictRequest) -> PredictResponse {
        let predictions = self
            .recent_observations
            .get(&request.object_id)
            .and_then(|observation| {
                let engine = AnticipationEngine::new(
                    self.anticipation_engine.anomaly_engine.clone(),
                    request.horizon_hours,
                );
                engine
                    .evaluate_observation(&self.twin, observation)
                    .map(|alert| alert.predictions)
            })
            .unwrap_or_else(|| {
                vec![baseline_prediction(
                    request.object_id.clone(),
                    request.horizon_hours,
                )]
            });

        PredictResponse {
            object_id: request.object_id,
            predictions,
        }
    }

    #[must_use]
    pub fn timeline(&self, request: TimelineRequest) -> Option<ObjectTimelineResponse> {
        let object = self.twin.object(&request.object_id)?;
        let base_epoch_unix_seconds = self
            .recent_observations
            .get(&request.object_id)
            .map_or(object.state.epoch_unix_seconds, |observation| {
                observation.observed_state.epoch_unix_seconds
            });
        let propagation_model = propagation_model_for(object).to_string();
        let checkpoints = timeline_offsets(request.horizon_hours)
            .into_iter()
            .filter_map(|offset_hours| {
                let target_epoch_unix_seconds =
                    base_epoch_unix_seconds + rounded_seconds_from_hours(offset_hours);
                let predicted_state = self
                    .twin
                    .predicted_state(&request.object_id, target_epoch_unix_seconds)?;
                Some(TimelineCheckpoint {
                    offset_hours,
                    target_epoch_unix_seconds,
                    summary: format!(
                        "{} projection to t+{offset_hours:.1}h at {:.1} km altitude",
                        propagation_model.to_uppercase(),
                        predicted_state.altitude_km()
                    ),
                    propagation_model: propagation_model.clone(),
                    predicted_state,
                })
            })
            .collect();

        Some(ObjectTimelineResponse {
            object_id: request.object_id,
            base_epoch_unix_seconds,
            checkpoints,
        })
    }

    #[must_use]
    pub fn space_overview(&self, request: SpaceOverviewRequest) -> SpaceOverviewResponse {
        let mut top_events = self
            .recent_observations
            .values()
            .filter_map(|observation| {
                let object = self.twin.object(&observation.object_id)?;
                let engine = AnticipationEngine::new(
                    self.anticipation_engine.anomaly_engine.clone(),
                    request.window_hours,
                );
                let alert = engine.evaluate_observation(&self.twin, observation)?;
                let anomaly = engine.anomaly_engine.evaluate(object, observation);
                Some(ranked_events_for(object, &alert, &anomaly))
            })
            .flatten()
            .collect::<Vec<_>>();
        top_events.extend(native_close_approach_events(
            &self.twin,
            &self.recent_observations,
            request.window_hours,
        ));

        top_events.sort_by(compare_ranked_events);
        top_events.truncate(10);

        SpaceOverviewResponse {
            window_hours: request.window_hours,
            top_events,
        }
    }
}

fn probability_from_anomaly(score: f64) -> f64 {
    (0.08 + score * 0.76).clamp(0.0, 0.92)
}

fn seconds_to_f64(seconds: i64) -> f64 {
    let magnitude = Duration::from_secs(seconds.unsigned_abs()).as_secs_f64();
    if seconds.is_negative() {
        -magnitude
    } else {
        magnitude
    }
}

fn propagate_object_state(
    object: &SpaceObject,
    at_epoch_unix_seconds: i64,
) -> Option<OrbitalState> {
    let tle = object.tle.as_ref()?;
    let elements = sgp4::Elements::from_tle(
        Some(object.name.clone()),
        tle.line1.as_bytes(),
        tle.line2.as_bytes(),
    )
    .ok()?;
    let epoch_unix_seconds = elements.datetime.and_utc().timestamp();
    let constants = sgp4::Constants::from_elements(&elements).ok()?;
    let minutes_since_epoch = seconds_to_f64(at_epoch_unix_seconds - epoch_unix_seconds) / 60.0;
    let prediction = constants
        .propagate(sgp4::MinutesSinceEpoch(minutes_since_epoch))
        .ok()?;

    Some(OrbitalState {
        epoch_unix_seconds: at_epoch_unix_seconds,
        position_km: vector3_from_sgp4(prediction.position),
        velocity_km_s: vector3_from_sgp4(prediction.velocity),
    })
}

fn vector3_from_sgp4(value: [f64; 3]) -> Vector3 {
    Vector3 {
        x: value[0],
        y: value[1],
        z: value[2],
    }
}

fn explain_anomaly(anomaly: &AnomalyReport) -> String {
    if anomaly.signals.is_empty() {
        return "current behavior remains inside the MVP baseline envelope".to_string();
    }

    let signals = anomaly
        .signals
        .iter()
        .map(|signal| match signal {
            AnomalySignal::UnexpectedManeuver => "unexpected maneuver",
            AnomalySignal::PositionDrift => "position drift",
            AnomalySignal::VelocityChange => "velocity change",
            AnomalySignal::RadioFrequencyShift => "RF shift",
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("detected {signals}")
}

fn severity_for_risk(score: f64) -> AlertSeverity {
    if score >= 0.85 {
        AlertSeverity::Critical
    } else if score >= 0.65 {
        AlertSeverity::Warning
    } else if score >= 0.35 {
        AlertSeverity::Watch
    } else {
        AlertSeverity::Info
    }
}

fn recommendation_for_risk(score: f64) -> &'static str {
    if score >= 0.85 {
        "escalate immediately and request multi-source confirmation"
    } else if score >= 0.65 {
        "increase tracking cadence and prepare operator notification"
    } else if score >= 0.35 {
        "keep on watchlist and compare next observation"
    } else {
        "continue normal monitoring"
    }
}

fn unknown_object_response(object_id: String) -> AnalyzeObjectResponse {
    let finding = empty_finding(object_id.clone(), "unknown object");
    let prediction = no_data_prediction();
    let assessment = IntelligenceAssessment {
        status: AnalysisStatus::UnknownObject,
        behavior: "unknown object".to_string(),
        confidence: 0.0,
        prediction: prediction.clone(),
        explanation: "object is not present in the active digital twin".to_string(),
        decision_version: decision_version().to_string(),
        model_version: model_version().to_string(),
        rule_hits: vec![RuleHit {
            rule_id: "object_not_in_twin".to_string(),
            description: "analysis cannot continue without a known digital twin entity".to_string(),
        }],
        signal_summary: Vec::new(),
    };
    let decision = empty_decision(AlertSeverity::Info, 0.0);
    let evidence_bundle = empty_evidence_bundle(object_id.clone());

    AnalyzeObjectResponse {
        object_id,
        status: AnalysisStatus::UnknownObject,
        anomaly_score: 0.0,
        behavior: "unknown object".to_string(),
        confidence: 0.0,
        prediction,
        explanation: "object is not present in the active digital twin".to_string(),
        risk_score: 0.0,
        severity: AlertSeverity::Info,
        recipients: Vec::new(),
        finding,
        assessment,
        decision,
        evidence_bundle,
    }
}

fn no_recent_observation_response(
    object: &SpaceObject,
    horizon_hours: f64,
) -> AnalyzeObjectResponse {
    let finding = empty_finding(object.id.clone(), "no recent observation");
    let prediction = baseline_prediction(object.id.clone(), horizon_hours);
    let assessment = IntelligenceAssessment {
        status: AnalysisStatus::NoRecentObservation,
        behavior: "no recent observation".to_string(),
        confidence: 0.2,
        prediction: prediction.clone(),
        explanation: "SSS needs a recent normalized observation to compare against behavior"
            .to_string(),
        decision_version: decision_version().to_string(),
        model_version: model_version().to_string(),
        rule_hits: vec![RuleHit {
            rule_id: "missing_recent_observation".to_string(),
            description: "no current evidence is available for behavioral comparison".to_string(),
        }],
        signal_summary: Vec::new(),
    };
    let risk_score = object.risk.criticality * 0.3;
    let decision = OperationalDecision {
        severity: AlertSeverity::Info,
        risk_score,
        recommendation: "continue normal monitoring".to_string(),
        recipient_policy: RecipientPolicy {
            policy_id: "sss-recipient-policy-v1".to_string(),
            escalation_rules: Vec::new(),
        },
        notifications: vec![NotificationDecision {
            recipient: IntelligenceRecipient::Operator,
            notification_reason: "operator owns watchlist triage for missing evidence".to_string(),
        }],
    };
    let evidence_bundle = empty_evidence_bundle(object.id.clone());

    AnalyzeObjectResponse {
        object_id: object.id.clone(),
        status: AnalysisStatus::NoRecentObservation,
        anomaly_score: 0.0,
        behavior: "no recent observation".to_string(),
        confidence: 0.2,
        prediction,
        explanation: "SSS needs a recent normalized observation to compare against behavior"
            .to_string(),
        risk_score,
        severity: AlertSeverity::Info,
        recipients: vec![IntelligenceRecipient::Operator],
        finding,
        assessment,
        decision,
        evidence_bundle,
    }
}

fn analyzed_object_response(
    object: &SpaceObject,
    observation: &Observation,
    anomaly: &AnomalyReport,
    alert: &DecisionAlert,
    horizon_hours: f64,
) -> AnalyzeObjectResponse {
    let prediction = primary_prediction_for_alert(alert)
        .cloned()
        .unwrap_or_else(|| baseline_prediction(object.id.clone(), horizon_hours));
    let finding = observation_finding(observation, anomaly);
    let evidence_bundle = evidence_bundle(observation, anomaly);
    let status = analysis_status_for_risk(alert.risk_score);
    let confidence = confidence_from_evidence(anomaly);
    let assessment = IntelligenceAssessment {
        status: status.clone(),
        behavior: behavior_label(anomaly).to_string(),
        confidence,
        prediction: prediction.clone(),
        explanation: explain_anomaly(anomaly),
        decision_version: decision_version().to_string(),
        model_version: model_version().to_string(),
        rule_hits: rule_hits_for(anomaly, alert, object),
        signal_summary: signal_summary(anomaly),
    };
    let decision = operational_decision(alert, object);
    let recipients = decision
        .notifications
        .iter()
        .map(|notification| notification.recipient.clone())
        .collect();

    AnalyzeObjectResponse {
        object_id: object.id.clone(),
        status,
        anomaly_score: anomaly.score,
        behavior: behavior_label(anomaly).to_string(),
        confidence,
        prediction,
        explanation: explain_anomaly(anomaly),
        risk_score: alert.risk_score,
        severity: alert.severity.clone(),
        recipients,
        finding,
        assessment,
        decision,
        evidence_bundle,
    }
}

fn decision_version() -> &'static str {
    "sss-decision-v1"
}

fn model_version() -> &'static str {
    "sss-behavioral-mvp-v1"
}

fn empty_finding(object_id: String, summary: &str) -> ObservationFinding {
    ObservationFinding {
        object_id,
        observed_at_unix_seconds: None,
        source: None,
        anomaly_score: 0.0,
        signals: Vec::new(),
        derived_features: DerivedFeatures {
            position_delta_km: 0.0,
            velocity_delta_km_s: 0.0,
            rf_delta_mhz: None,
            maneuver_detected: false,
        },
        summary: summary.to_string(),
    }
}

fn empty_evidence_bundle(object_id: String) -> EvidenceBundle {
    let mut bundle = EvidenceBundle {
        object_id,
        window_start_unix_seconds: None,
        window_end_unix_seconds: None,
        observations: Vec::new(),
        sources: Vec::new(),
        events: Vec::new(),
        anomaly_score: 0.0,
        signals: Vec::new(),
        derived_features: DerivedFeatures {
            position_delta_km: 0.0,
            velocity_delta_km_s: 0.0,
            rf_delta_mhz: None,
            maneuver_detected: false,
        },
        confidence_contributors: Vec::new(),
        bundle_hash: String::new(),
    };
    bundle.bundle_hash = evidence_bundle_hash(&bundle);
    bundle
}

fn observation_finding(observation: &Observation, anomaly: &AnomalyReport) -> ObservationFinding {
    ObservationFinding {
        object_id: observation.object_id.clone(),
        observed_at_unix_seconds: Some(observation.observed_state.epoch_unix_seconds),
        source: Some(observation.source.clone()),
        anomaly_score: anomaly.score,
        signals: anomaly.signals.clone(),
        derived_features: derived_features(observation, anomaly),
        summary: explain_anomaly(anomaly),
    }
}

fn evidence_bundle(observation: &Observation, anomaly: &AnomalyReport) -> EvidenceBundle {
    let observed_at = observation.observed_state.epoch_unix_seconds;
    let derived_features = derived_features(observation, anomaly);
    let observation_event = intelligence_event(
        &observation.object_id,
        IntelligenceEventType::Observation,
        EvidenceStrength::ObservedFact,
        source_id_for(&observation.source),
        Some(observed_at),
        format!(
            "normalized orbital observation from {:?}",
            observation.source
        ),
        Vec::new(),
    );
    let interpretation_event = intelligence_event(
        &observation.object_id,
        IntelligenceEventType::Interpretation,
        EvidenceStrength::InferredContext,
        "sss-core".to_string(),
        Some(observed_at),
        explain_anomaly(anomaly),
        vec![observation_event.event_id.clone()],
    );

    let mut bundle = EvidenceBundle {
        object_id: observation.object_id.clone(),
        window_start_unix_seconds: Some(observed_at),
        window_end_unix_seconds: Some(observed_at),
        observations: vec![observation.clone()],
        sources: vec![observation.source.clone()],
        events: vec![observation_event, interpretation_event],
        anomaly_score: anomaly.score,
        signals: anomaly.signals.clone(),
        derived_features,
        confidence_contributors: confidence_contributors(anomaly),
        bundle_hash: String::new(),
    };
    bundle.bundle_hash = evidence_bundle_hash(&bundle);
    bundle
}

fn source_id_for(source: &ObservationSource) -> String {
    match source {
        ObservationSource::Radar => "radar",
        ObservationSource::Telescope => "telescope",
        ObservationSource::RadioFrequency => "rf",
        ObservationSource::Catalog => "catalog",
        ObservationSource::Api => "api",
        ObservationSource::Synthetic => "synthetic",
    }
    .to_string()
}

fn intelligence_event(
    object_id: &str,
    event_type: IntelligenceEventType,
    strength: EvidenceStrength,
    source_id: String,
    timestamp_unix_seconds: Option<i64>,
    payload_summary: String,
    causal_parent_ids: Vec<String>,
) -> IntelligenceEvent {
    let event_id = stable_id(&format!(
        "{object_id}:{event_type:?}:{strength:?}:{source_id}:{timestamp_unix_seconds:?}:{payload_summary}:{causal_parent_ids:?}"
    ));
    let canonical_hash = stable_id(&format!(
        "{object_id}:{event_type:?}:{strength:?}:{source_id}:{timestamp_unix_seconds:?}:sss-intelligence-event-v1:{payload_summary}:{causal_parent_ids:?}"
    ));
    IntelligenceEvent {
        event_id,
        object_id: object_id.to_string(),
        event_type,
        strength,
        source_id,
        timestamp_unix_seconds,
        schema_version: "sss-intelligence-event-v1".to_string(),
        payload_summary,
        causal_parent_ids,
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

#[must_use]
pub fn replay_manifest_for(bundle: &EvidenceBundle) -> ReplayManifest {
    let event_ids = bundle
        .events
        .iter()
        .map(|event| event.event_id.clone())
        .collect::<Vec<_>>();
    let versions = BTreeMap::from([
        ("evidence_model".to_string(), "sss-evidence-v1".to_string()),
        ("behavior_model".to_string(), model_version().to_string()),
        (
            "decision_engine".to_string(),
            decision_version().to_string(),
        ),
        (
            "canonicalization".to_string(),
            "sss-canonical-v1".to_string(),
        ),
        ("hash_algorithm".to_string(), "siphash-1-3".to_string()),
    ]);
    let event_fingerprint = stable_id(&event_ids.join("|"));
    let manifest_hash = stable_id(&format!(
        "{}:{:?}:{event_fingerprint}:{versions:?}",
        bundle.object_id, bundle.window_end_unix_seconds
    ));
    ReplayManifest {
        object_id: bundle.object_id.clone(),
        created_at_unix_seconds: bundle.window_end_unix_seconds,
        event_ids,
        event_fingerprint,
        versions,
        manifest_hash,
    }
}

#[must_use]
pub fn replay_assessment_from_evidence_bundle(bundle: &EvidenceBundle) -> IntelligenceAssessment {
    let anomaly = AnomalyReport {
        object_id: bundle.object_id.clone(),
        score: bundle.anomaly_score,
        position_delta_km: bundle.derived_features.position_delta_km,
        velocity_delta_km_s: bundle.derived_features.velocity_delta_km_s,
        rf_delta_mhz: bundle.derived_features.rf_delta_mhz,
        signals: bundle.signals.clone(),
    };
    let prediction = baseline_prediction(bundle.object_id.clone(), DEFAULT_WINDOW_HOURS);

    IntelligenceAssessment {
        status: analysis_status_for_risk(anomaly.score),
        behavior: behavior_label(&anomaly).to_string(),
        confidence: confidence_from_evidence(&anomaly),
        prediction,
        explanation: explain_anomaly(&anomaly),
        decision_version: decision_version().to_string(),
        model_version: model_version().to_string(),
        rule_hits: replay_rule_hits_for(&anomaly),
        signal_summary: signal_summary(&anomaly),
    }
}

#[must_use]
pub fn replay_response_from_evidence_bundle(
    bundle: &EvidenceBundle,
    object: &SpaceObject,
) -> AnalyzeObjectResponse {
    let mut layer = IntelligenceLayer::new(
        DigitalTwin::new(vec![object.clone()]),
        AnticipationEngine::default(),
    );
    for observation in &bundle.observations {
        layer.record_observation(observation.clone());
    }
    layer.analyze_object(AnalyzeObjectRequest {
        object_id: object.id.clone(),
        timestamp_unix_seconds: bundle.window_end_unix_seconds,
    })
}

#[must_use]
pub fn replay_diff(
    original_assessment: Option<&IntelligenceAssessment>,
    replayed_assessment: &IntelligenceAssessment,
    original_decision: Option<&OperationalDecision>,
    replayed_decision: &OperationalDecision,
) -> ReplayDiff {
    ReplayDiff {
        assessment: assessment_diff(original_assessment, replayed_assessment),
        decision: decision_diff(original_decision, replayed_decision),
    }
}

fn stable_id(value: &str) -> String {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn derived_features(observation: &Observation, anomaly: &AnomalyReport) -> DerivedFeatures {
    DerivedFeatures {
        position_delta_km: anomaly.position_delta_km,
        velocity_delta_km_s: anomaly.velocity_delta_km_s,
        rf_delta_mhz: anomaly.rf_delta_mhz,
        maneuver_detected: observation.maneuver_detected,
    }
}

fn confidence_contributors(anomaly: &AnomalyReport) -> Vec<ConfidenceContributor> {
    let mut contributors = vec![ConfidenceContributor {
        name: "anomaly_score".to_string(),
        weight: anomaly.score,
        reason: "normalized deviation from the current behavioral twin".to_string(),
    }];

    if !anomaly.signals.is_empty() {
        contributors.push(ConfidenceContributor {
            name: "multi_signal_support".to_string(),
            weight: 0.24,
            reason: "independent behavioral signals support the same assessment".to_string(),
        });
    }

    if anomaly.signals.contains(&AnomalySignal::UnexpectedManeuver) {
        contributors.push(ConfidenceContributor {
            name: "unexpected_maneuver".to_string(),
            weight: 0.2,
            reason: "maneuver does not match the object's historical cadence".to_string(),
        });
    }

    contributors
}

fn rule_hits_for(
    anomaly: &AnomalyReport,
    alert: &DecisionAlert,
    object: &SpaceObject,
) -> Vec<RuleHit> {
    let mut hits = anomaly
        .signals
        .iter()
        .map(|signal| RuleHit {
            rule_id: match signal {
                AnomalySignal::UnexpectedManeuver => "unexpected_maneuver",
                AnomalySignal::PositionDrift => "position_drift",
                AnomalySignal::VelocityChange => "velocity_change",
                AnomalySignal::RadioFrequencyShift => "rf_shift",
            }
            .to_string(),
            description: match signal {
                AnomalySignal::UnexpectedManeuver => {
                    "maneuver detected below normal maneuver cadence"
                }
                AnomalySignal::PositionDrift => "observed position diverges from twin projection",
                AnomalySignal::VelocityChange => "observed velocity diverges from twin projection",
                AnomalySignal::RadioFrequencyShift => "RF differs from nominal signature",
            }
            .to_string(),
        })
        .collect::<Vec<_>>();

    if alert.risk_score >= 0.65 {
        hits.push(RuleHit {
            rule_id: "risk_escalation_threshold".to_string(),
            description: "combined anomaly and criticality exceeded warning threshold".to_string(),
        });
    }

    if alert
        .predictions
        .iter()
        .any(|prediction| prediction.event == PredictedEvent::CloseApproach)
    {
        hits.push(RuleHit {
            rule_id: "predicted_close_approach".to_string(),
            description: "pairwise orbital projection indicates a close approach inside horizon"
                .to_string(),
        });
    }

    if object.risk.mission_class == MissionClass::Defense {
        hits.push(RuleHit {
            rule_id: "institutional_visibility".to_string(),
            description: "defense-class object requires institutional visibility".to_string(),
        });
    }

    hits
}

fn replay_rule_hits_for(anomaly: &AnomalyReport) -> Vec<RuleHit> {
    let mut hits = anomaly
        .signals
        .iter()
        .map(|signal| RuleHit {
            rule_id: match signal {
                AnomalySignal::UnexpectedManeuver => "unexpected_maneuver",
                AnomalySignal::PositionDrift => "position_drift",
                AnomalySignal::VelocityChange => "velocity_change",
                AnomalySignal::RadioFrequencyShift => "rf_shift",
            }
            .to_string(),
            description: match signal {
                AnomalySignal::UnexpectedManeuver => {
                    "maneuver detected below normal maneuver cadence"
                }
                AnomalySignal::PositionDrift => "observed position diverges from twin projection",
                AnomalySignal::VelocityChange => "observed velocity diverges from twin projection",
                AnomalySignal::RadioFrequencyShift => "RF differs from nominal signature",
            }
            .to_string(),
        })
        .collect::<Vec<_>>();

    if anomaly.score >= 0.65 {
        hits.push(RuleHit {
            rule_id: "risk_escalation_threshold".to_string(),
            description: "evidence-only replay score exceeded warning threshold".to_string(),
        });
    }

    hits
}

fn signal_summary(anomaly: &AnomalyReport) -> Vec<String> {
    anomaly
        .signals
        .iter()
        .map(|signal| match signal {
            AnomalySignal::UnexpectedManeuver => "unexpected maneuver",
            AnomalySignal::PositionDrift => "position drift",
            AnomalySignal::VelocityChange => "velocity change",
            AnomalySignal::RadioFrequencyShift => "RF shift",
        })
        .map(str::to_string)
        .collect()
}

fn operational_decision(alert: &DecisionAlert, object: &SpaceObject) -> OperationalDecision {
    let escalation_rules = escalation_rules_for(alert, object);
    let notifications = recipients_for_alert(alert, object)
        .into_iter()
        .map(|recipient| NotificationDecision {
            notification_reason: notification_reason_for(&recipient, alert, object),
            recipient,
        })
        .collect();

    OperationalDecision {
        severity: alert.severity.clone(),
        risk_score: alert.risk_score,
        recommendation: alert.recommendation.clone(),
        recipient_policy: RecipientPolicy {
            policy_id: "sss-recipient-policy-v1".to_string(),
            escalation_rules,
        },
        notifications,
    }
}

fn empty_decision(severity: AlertSeverity, risk_score: f64) -> OperationalDecision {
    OperationalDecision {
        severity,
        risk_score,
        recommendation: "continue normal monitoring".to_string(),
        recipient_policy: RecipientPolicy {
            policy_id: "sss-recipient-policy-v1".to_string(),
            escalation_rules: Vec::new(),
        },
        notifications: Vec::new(),
    }
}

fn escalation_rules_for(alert: &DecisionAlert, object: &SpaceObject) -> Vec<EscalationRule> {
    let mut rules = Vec::new();
    let has_close_approach = alert
        .predictions
        .iter()
        .any(|prediction| prediction.event == PredictedEvent::CloseApproach);

    if matches!(
        alert.severity,
        AlertSeverity::Warning | AlertSeverity::Critical
    ) {
        rules.push(EscalationRule {
            rule_id: "warn_or_higher_to_coordination".to_string(),
            reason: "severity requires owner and space-traffic coordination visibility".to_string(),
        });
    }

    if has_close_approach {
        rules.push(EscalationRule {
            rule_id: "predicted_close_approach_to_coordination".to_string(),
            reason: "predicted close approach requires explicit orbital coordination visibility"
                .to_string(),
        });
    }

    if alert.severity == AlertSeverity::Critical {
        rules.push(EscalationRule {
            rule_id: "critical_to_national_authority".to_string(),
            reason: "critical event may require institutional escalation".to_string(),
        });
    }

    if object.risk.mission_class == MissionClass::Defense {
        rules.push(EscalationRule {
            rule_id: "defense_to_national_authority".to_string(),
            reason: "defense-class object requires national authority visibility".to_string(),
        });
    }

    if object.risk.criticality >= 0.7 {
        rules.push(EscalationRule {
            rule_id: "high_criticality_to_risk_desk".to_string(),
            reason: "high criticality object affects risk exposure".to_string(),
        });
    }

    rules
}

fn notification_reason_for(
    recipient: &IntelligenceRecipient,
    alert: &DecisionAlert,
    object: &SpaceObject,
) -> String {
    let close_approach_prediction = primary_prediction_for_alert(alert)
        .filter(|prediction| prediction.event == PredictedEvent::CloseApproach);
    match recipient {
        IntelligenceRecipient::Operator => "operator owns immediate triage".to_string(),
        IntelligenceRecipient::SatelliteOwner => {
            if let Some(prediction) = close_approach_prediction {
                format!(
                    "owner is notified because a close approach is projected within {:.1} h",
                    prediction.horizon_hours
                )
            } else {
                "owner is notified because severity reached warning or higher".to_string()
            }
        }
        IntelligenceRecipient::SpaceTrafficCoordination => {
            if let Some(prediction) = close_approach_prediction {
                format!(
                    "coordination desk is notified because {}",
                    prediction.explanation
                )
            } else {
                "coordination desk is notified because event may affect orbital traffic".to_string()
            }
        }
        IntelligenceRecipient::NationalAuthority => {
            if object.risk.mission_class == MissionClass::Defense {
                "national authority is notified because mission_class=defense".to_string()
            } else {
                format!(
                    "national authority is notified because severity={:?}",
                    alert.severity
                )
            }
        }
        IntelligenceRecipient::InsuranceRiskDesk => {
            "risk desk is notified because object criticality is high".to_string()
        }
    }
}

fn ranked_events_for(
    object: &SpaceObject,
    alert: &DecisionAlert,
    anomaly: &AnomalyReport,
) -> Vec<RankedEvent> {
    let mut events = Vec::new();

    if !anomaly.signals.is_empty() {
        events.push(RankedEvent {
            event_id: format!("{}:behavior_shift", object.id),
            object_id: object.id.clone(),
            object_name: object.name.clone(),
            risk: alert.risk_score,
            target_epoch_unix_seconds: alert
                .predictions
                .first()
                .and_then(prediction_target_epoch_unix_seconds),
            event_type: RankedEventType::BehaviorShiftDetected,
            prediction: alert.predictions.first().cloned(),
            summary: format!("{}: {}", object.name, explain_anomaly(anomaly)),
        });
    }

    for prediction in &alert.predictions {
        let event_type = match prediction.event {
            PredictedEvent::CloseApproach => RankedEventType::PredictedCloseApproach,
            PredictedEvent::UnstableTrack => RankedEventType::UnstableTrack,
            PredictedEvent::ContinuedAnomalousBehavior => RankedEventType::BehaviorShiftDetected,
        };
        events.push(RankedEvent {
            event_id: format!("{}:{:?}", object.id, event_type),
            object_id: object.id.clone(),
            object_name: object.name.clone(),
            risk: (alert.risk_score * 0.65 + prediction.probability * 0.35).min(1.0),
            target_epoch_unix_seconds: prediction_target_epoch_unix_seconds(prediction),
            event_type,
            prediction: Some(prediction.clone()),
            summary: prediction.explanation.clone(),
        });
    }

    events
}

fn native_close_approach_events(
    twin: &DigitalTwin,
    recent_observations: &BTreeMap<String, Observation>,
    horizon_hours: f64,
) -> Vec<RankedEvent> {
    let objects = recent_observations
        .keys()
        .filter_map(|object_id| twin.object(object_id).cloned())
        .collect::<Vec<_>>();
    let mut emitted_pairs = BTreeSet::new();
    let mut events = Vec::new();

    for (index, left) in objects.iter().enumerate() {
        for right in objects.iter().skip(index + 1) {
            let pair_key = if left.id <= right.id {
                format!("{}:{}", left.id, right.id)
            } else {
                format!("{}:{}", right.id, left.id)
            };
            if !emitted_pairs.insert(pair_key.clone()) {
                continue;
            }
            let base_epoch_unix_seconds = recent_observations
                .get(&left.id)
                .zip(recent_observations.get(&right.id))
                .map_or(
                    left.state
                        .epoch_unix_seconds
                        .max(right.state.epoch_unix_seconds),
                    |(left_observation, right_observation)| {
                        left_observation
                            .observed_state
                            .epoch_unix_seconds
                            .max(right_observation.observed_state.epoch_unix_seconds)
                    },
                );
            let Some((target_epoch_unix_seconds, left_state, miss_distance_km)) =
                pairwise_close_approach(twin, left, right, base_epoch_unix_seconds, horizon_hours)
            else {
                continue;
            };
            if miss_distance_km > OVERVIEW_CLOSE_APPROACH_KM {
                continue;
            }

            let hours_until_event =
                seconds_to_f64(target_epoch_unix_seconds - base_epoch_unix_seconds) / 3600.0;
            let pair_criticality = left.risk.criticality.max(right.risk.criticality);
            let probability =
                close_approach_probability(miss_distance_km, hours_until_event, pair_criticality);
            let explanation = format!(
                "{} and {} are projected within {:.1} km in {:.1} h",
                left.name,
                right.name,
                miss_distance_km,
                hours_until_event.max(0.0)
            );
            let primary_object = if left.risk.criticality >= right.risk.criticality {
                left
            } else {
                right
            };
            let prediction = Prediction {
                object_id: primary_object.id.clone(),
                horizon_hours: hours_until_event.max(0.0),
                event: PredictedEvent::CloseApproach,
                probability,
                explanation: explanation.clone(),
                propagation_model: pair_propagation_model(left, right).to_string(),
                based_on_observation_epoch: Some(base_epoch_unix_seconds),
                predicted_state: Some(left_state),
                position_delta_km: Some(miss_distance_km),
                velocity_delta_km_s: None,
            };
            events.push(RankedEvent {
                event_id: format!("{pair_key}:native_close_approach"),
                object_id: primary_object.id.clone(),
                object_name: primary_object.name.clone(),
                risk: (probability * 0.75 + pair_criticality * 0.25).min(1.0),
                target_epoch_unix_seconds: Some(target_epoch_unix_seconds),
                event_type: RankedEventType::PredictedCloseApproach,
                prediction: Some(prediction),
                summary: explanation,
            });
        }
    }

    events
}

fn pairwise_close_approach(
    twin: &DigitalTwin,
    left: &SpaceObject,
    right: &SpaceObject,
    base_epoch_unix_seconds: i64,
    horizon_hours: f64,
) -> Option<(i64, OrbitalState, f64)> {
    let mut best: Option<(i64, OrbitalState, f64)> = None;
    for offset_hours in hourly_offsets_with_horizon(horizon_hours) {
        let target_epoch_unix_seconds =
            base_epoch_unix_seconds + rounded_seconds_from_hours(offset_hours);
        let left_state = twin.predicted_state(&left.id, target_epoch_unix_seconds)?;
        let right_state = twin.predicted_state(&right.id, target_epoch_unix_seconds)?;
        let miss_distance_km = left_state.position_km.distance_to(right_state.position_km);
        match best {
            Some((_, _, current_best_distance_km))
                if miss_distance_km >= current_best_distance_km => {}
            _ => best = Some((target_epoch_unix_seconds, left_state, miss_distance_km)),
        }
    }
    best
}

fn hourly_offsets_with_horizon(horizon_hours: f64) -> Vec<f64> {
    let mut offsets = timeline_offsets(horizon_hours);
    let rounded_horizon_hours = rounded_seconds_from_hours(horizon_hours.ceil()) / 3600;
    for hour in 1..=rounded_horizon_hours {
        let hour_offset = Duration::from_secs(hour.unsigned_abs()).as_secs_f64();
        if !offsets
            .iter()
            .any(|existing| (existing - hour_offset).abs() < f64::EPSILON)
        {
            offsets.push(hour_offset);
        }
    }
    offsets.sort_by(f64::total_cmp);
    offsets
}

fn close_approach_probability(
    miss_distance_km: f64,
    hours_until_event: f64,
    criticality: f64,
) -> f64 {
    let distance_factor = (1.0 - miss_distance_km / OVERVIEW_CLOSE_APPROACH_KM).clamp(0.0, 1.0);
    let urgency_factor = (1.0 - hours_until_event / DEFAULT_WINDOW_HOURS).clamp(0.0, 1.0);
    (distance_factor * 0.5 + urgency_factor * 0.2 + criticality * 0.3).clamp(0.0, 0.98)
}

fn pair_propagation_model(left: &SpaceObject, right: &SpaceObject) -> &'static str {
    match (left.tle.is_some(), right.tle.is_some()) {
        (true, true) => "sgp4-pair",
        (true, false) | (false, true) => "hybrid",
        (false, false) => "linear-pair",
    }
}

fn no_data_prediction() -> Prediction {
    Prediction {
        object_id: "unknown".to_string(),
        horizon_hours: 0.0,
        event: PredictedEvent::UnstableTrack,
        probability: 0.0,
        explanation: "no digital twin object is available".to_string(),
        propagation_model: "unavailable".to_string(),
        based_on_observation_epoch: None,
        predicted_state: None,
        position_delta_km: None,
        velocity_delta_km_s: None,
    }
}

fn compare_ranked_events(left: &RankedEvent, right: &RankedEvent) -> Ordering {
    right.risk.total_cmp(&left.risk).then_with(|| {
        match (
            left.target_epoch_unix_seconds,
            right.target_epoch_unix_seconds,
        ) {
            (Some(left_epoch), Some(right_epoch)) => left_epoch.cmp(&right_epoch),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => left.event_id.cmp(&right.event_id),
        }
    })
}

fn prediction_target_epoch_unix_seconds(prediction: &Prediction) -> Option<i64> {
    prediction
        .predicted_state
        .as_ref()
        .map(|state| state.epoch_unix_seconds)
        .or(prediction.based_on_observation_epoch)
}

fn assessment_diff(
    original: Option<&IntelligenceAssessment>,
    replayed: &IntelligenceAssessment,
) -> AssessmentDiff {
    let Some(original) = original else {
        return AssessmentDiff {
            changed_fields: vec![
                "status".to_string(),
                "behavior".to_string(),
                "explanation".to_string(),
                "prediction.event".to_string(),
            ],
            confidence_delta: replayed.confidence,
            prediction_probability_delta: replayed.prediction.probability,
            added_rule_hits: replayed
                .rule_hits
                .iter()
                .map(|rule| rule.rule_id.clone())
                .collect(),
            removed_rule_hits: Vec::new(),
            added_signals: replayed.signal_summary.clone(),
            removed_signals: Vec::new(),
        };
    };

    let mut changed_fields = Vec::new();
    if original.status != replayed.status {
        changed_fields.push("status".to_string());
    }
    if original.behavior != replayed.behavior {
        changed_fields.push("behavior".to_string());
    }
    if original.explanation != replayed.explanation {
        changed_fields.push("explanation".to_string());
    }
    if original.prediction.event != replayed.prediction.event {
        changed_fields.push("prediction.event".to_string());
    }

    AssessmentDiff {
        changed_fields,
        confidence_delta: replayed.confidence - original.confidence,
        prediction_probability_delta: replayed.prediction.probability
            - original.prediction.probability,
        added_rule_hits: string_set_difference(
            original.rule_hits.iter().map(|rule| rule.rule_id.as_str()),
            replayed.rule_hits.iter().map(|rule| rule.rule_id.as_str()),
        ),
        removed_rule_hits: string_set_difference(
            replayed.rule_hits.iter().map(|rule| rule.rule_id.as_str()),
            original.rule_hits.iter().map(|rule| rule.rule_id.as_str()),
        ),
        added_signals: string_set_difference(
            original.signal_summary.iter().map(String::as_str),
            replayed.signal_summary.iter().map(String::as_str),
        ),
        removed_signals: string_set_difference(
            replayed.signal_summary.iter().map(String::as_str),
            original.signal_summary.iter().map(String::as_str),
        ),
    }
}

fn decision_diff(
    original: Option<&OperationalDecision>,
    replayed: &OperationalDecision,
) -> DecisionDiff {
    let Some(original) = original else {
        return DecisionDiff {
            changed_fields: vec![
                "severity".to_string(),
                "recommendation".to_string(),
                "risk_score".to_string(),
            ],
            risk_score_delta: Some(replayed.risk_score),
            added_recipients: replayed
                .notifications
                .iter()
                .map(|notification| notification.recipient.clone())
                .collect(),
            removed_recipients: Vec::new(),
            added_escalation_rules: replayed
                .recipient_policy
                .escalation_rules
                .iter()
                .map(|rule| rule.rule_id.clone())
                .collect(),
            removed_escalation_rules: Vec::new(),
        };
    };

    let mut changed_fields = Vec::new();
    if original.severity != replayed.severity {
        changed_fields.push("severity".to_string());
    }
    if original.recommendation != replayed.recommendation {
        changed_fields.push("recommendation".to_string());
    }
    if (replayed.risk_score - original.risk_score).abs() > f64::EPSILON {
        changed_fields.push("risk_score".to_string());
    }

    DecisionDiff {
        changed_fields,
        risk_score_delta: Some(replayed.risk_score - original.risk_score),
        added_recipients: recipient_set_difference(
            original.notifications.iter().map(|item| &item.recipient),
            replayed.notifications.iter().map(|item| &item.recipient),
        ),
        removed_recipients: recipient_set_difference(
            replayed.notifications.iter().map(|item| &item.recipient),
            original.notifications.iter().map(|item| &item.recipient),
        ),
        added_escalation_rules: string_set_difference(
            original
                .recipient_policy
                .escalation_rules
                .iter()
                .map(|rule| rule.rule_id.as_str()),
            replayed
                .recipient_policy
                .escalation_rules
                .iter()
                .map(|rule| rule.rule_id.as_str()),
        ),
        removed_escalation_rules: string_set_difference(
            replayed
                .recipient_policy
                .escalation_rules
                .iter()
                .map(|rule| rule.rule_id.as_str()),
            original
                .recipient_policy
                .escalation_rules
                .iter()
                .map(|rule| rule.rule_id.as_str()),
        ),
    }
}

fn string_set_difference<'a>(
    baseline: impl Iterator<Item = &'a str>,
    candidate: impl Iterator<Item = &'a str>,
) -> Vec<String> {
    let baseline = baseline.collect::<BTreeSet<_>>();
    let candidate = candidate.collect::<BTreeSet<_>>();
    candidate
        .difference(&baseline)
        .map(|value| (*value).to_string())
        .collect()
}

fn recipient_set_difference<'a>(
    baseline: impl Iterator<Item = &'a IntelligenceRecipient>,
    candidate: impl Iterator<Item = &'a IntelligenceRecipient>,
) -> Vec<IntelligenceRecipient> {
    let baseline = baseline.collect::<BTreeSet<_>>();
    let candidate = candidate.collect::<BTreeSet<_>>();
    candidate
        .difference(&baseline)
        .map(|value| (*value).clone())
        .collect()
}

fn baseline_prediction(object_id: String, horizon_hours: f64) -> Prediction {
    Prediction {
        object_id,
        horizon_hours,
        event: PredictedEvent::ContinuedAnomalousBehavior,
        probability: 0.08,
        explanation: "no anomaly evidence is available for this horizon".to_string(),
        propagation_model: "baseline".to_string(),
        based_on_observation_epoch: None,
        predicted_state: None,
        position_delta_km: None,
        velocity_delta_km_s: None,
    }
}

#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
fn build_prediction(
    twin: &DigitalTwin,
    object: &SpaceObject,
    observation: Option<&Observation>,
    anomaly: Option<&AnomalyReport>,
    horizon_hours: f64,
    event: PredictedEvent,
    probability: f64,
    explanation: String,
) -> Prediction {
    let predicted_state = observation
        .and_then(|value| projected_state_for_prediction(twin, object, value, horizon_hours));
    let propagation_model = propagation_model_for(object).to_string();
    let explanation = match predicted_state.as_ref() {
        Some(state) => format!(
            "{}; {} projection to t+{:.1}h places object at {:.1} km altitude",
            explanation,
            propagation_model.to_uppercase(),
            horizon_hours,
            state.altitude_km()
        ),
        None => {
            format!("{explanation}; no future state projection is available from the current twin")
        }
    };

    Prediction {
        object_id: object.id.clone(),
        horizon_hours,
        event,
        probability,
        explanation,
        propagation_model,
        based_on_observation_epoch: observation
            .map(|value| value.observed_state.epoch_unix_seconds),
        predicted_state,
        position_delta_km: anomaly.map(|value| value.position_delta_km),
        velocity_delta_km_s: anomaly.map(|value| value.velocity_delta_km_s),
    }
}

fn projected_state_for_prediction(
    twin: &DigitalTwin,
    object: &SpaceObject,
    observation: &Observation,
    horizon_hours: f64,
) -> Option<OrbitalState> {
    let target_epoch =
        observation.observed_state.epoch_unix_seconds + rounded_seconds_from_hours(horizon_hours);
    twin.predicted_state(&object.id, target_epoch)
}

fn propagation_model_for(object: &SpaceObject) -> &'static str {
    if object.tle.is_some() {
        "sgp4"
    } else {
        "linear"
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn rounded_seconds_from_hours(hours: f64) -> i64 {
    let seconds = (hours * 3_600.0).round();
    if !seconds.is_finite() {
        return 0;
    }
    if seconds >= i64::MAX as f64 {
        i64::MAX
    } else if seconds <= i64::MIN as f64 {
        i64::MIN
    } else {
        seconds as i64
    }
}

fn timeline_offsets(horizon_hours: f64) -> Vec<f64> {
    let normalized_horizon = horizon_hours.max(0.0);
    let mut offsets = Vec::new();
    for candidate in [1.0, 6.0, 24.0, normalized_horizon] {
        if candidate > 0.0
            && !offsets
                .iter()
                .any(|existing| (existing - candidate).abs() < f64::EPSILON)
        {
            offsets.push(candidate);
        }
    }
    offsets.sort_by(f64::total_cmp);
    offsets
}

fn analysis_status_for_risk(risk_score: f64) -> AnalysisStatus {
    if risk_score >= 0.65 {
        AnalysisStatus::AnomalyDetected
    } else if risk_score >= 0.35 {
        AnalysisStatus::Watch
    } else {
        AnalysisStatus::Nominal
    }
}

fn behavior_label(anomaly: &AnomalyReport) -> &'static str {
    if anomaly.signals.contains(&AnomalySignal::UnexpectedManeuver) {
        "unexpected orbital adjustment"
    } else if anomaly.signals.contains(&AnomalySignal::PositionDrift) {
        "orbital behavior drift"
    } else if anomaly
        .signals
        .contains(&AnomalySignal::RadioFrequencyShift)
    {
        "radio-frequency behavior shift"
    } else {
        "baseline behavior"
    }
}

fn confidence_from_evidence(anomaly: &AnomalyReport) -> f64 {
    let evidence = match anomaly.signals.len() {
        0 => 0.0,
        1 => 0.12,
        2 => 0.24,
        3 => 0.36,
        _ => 0.48,
    };
    (0.44 + anomaly.score * 0.36 + evidence).clamp(0.0, 0.96)
}

fn recipients_for_alert(alert: &DecisionAlert, object: &SpaceObject) -> Vec<IntelligenceRecipient> {
    let mut recipients = vec![IntelligenceRecipient::Operator];
    let has_close_approach = alert
        .predictions
        .iter()
        .any(|prediction| prediction.event == PredictedEvent::CloseApproach);

    if matches!(
        alert.severity,
        AlertSeverity::Warning | AlertSeverity::Critical
    ) || has_close_approach
    {
        recipients.push(IntelligenceRecipient::SatelliteOwner);
        recipients.push(IntelligenceRecipient::SpaceTrafficCoordination);
    }

    if alert.severity == AlertSeverity::Critical
        || object.risk.mission_class == MissionClass::Defense
    {
        recipients.push(IntelligenceRecipient::NationalAuthority);
    }

    if object.risk.criticality >= 0.7 {
        recipients.push(IntelligenceRecipient::InsuranceRiskDesk);
    }

    recipients
}

fn primary_prediction_for_alert(alert: &DecisionAlert) -> Option<&Prediction> {
    alert.predictions.iter().max_by(|left, right| {
        prediction_priority(left)
            .cmp(&prediction_priority(right))
            .then_with(|| left.probability.total_cmp(&right.probability))
    })
}

fn prediction_priority(prediction: &Prediction) -> u8 {
    match prediction.event {
        PredictedEvent::CloseApproach => 3,
        PredictedEvent::UnstableTrack => 2,
        PredictedEvent::ContinuedAnomalousBehavior => 1,
    }
}

fn predicted_pairwise_close_approach(
    twin: &DigitalTwin,
    object: &SpaceObject,
    observation: &Observation,
    horizon_hours: f64,
) -> Option<Prediction> {
    let base_epoch_unix_seconds = observation.observed_state.epoch_unix_seconds;
    let best = twin
        .objects()
        .filter(|candidate| candidate.id != object.id)
        .filter_map(|candidate| {
            pairwise_close_approach(
                twin,
                object,
                candidate,
                base_epoch_unix_seconds,
                horizon_hours,
            )
            .map(
                |(target_epoch_unix_seconds, primary_state, miss_distance_km)| {
                    (
                        candidate,
                        target_epoch_unix_seconds,
                        primary_state,
                        miss_distance_km,
                    )
                },
            )
        })
        .min_by(|left, right| left.3.total_cmp(&right.3))?;
    if best.3 > OVERVIEW_CLOSE_APPROACH_KM {
        return None;
    }
    let horizon_to_event_hours = seconds_to_f64(best.1 - base_epoch_unix_seconds).max(0.0) / 3600.0;
    Some(Prediction {
        object_id: object.id.clone(),
        horizon_hours: horizon_to_event_hours,
        event: PredictedEvent::CloseApproach,
        probability: close_approach_probability(
            best.3,
            horizon_to_event_hours,
            object.risk.criticality.max(best.0.risk.criticality),
        ),
        explanation: format!(
            "{} and {} are projected within {:.1} km in {:.1} h",
            object.name, best.0.name, best.3, horizon_to_event_hours
        ),
        propagation_model: pair_propagation_model(object, best.0).to_string(),
        based_on_observation_epoch: Some(base_epoch_unix_seconds),
        predicted_state: Some(best.2),
        position_delta_km: Some(best.3),
        velocity_delta_km_s: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_object() -> SpaceObject {
        SpaceObject {
            id: "SAT-001".to_string(),
            name: "Alice Demo Sat".to_string(),
            regime: OrbitRegime::Leo,
            state: OrbitalState {
                epoch_unix_seconds: 1_800_000_000,
                position_km: Vector3 {
                    x: 6_900.0,
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
                operator: Some("SSS synthetic".to_string()),
                mission_class: MissionClass::Commercial,
                criticality: 0.55,
            },
        }
    }

    fn tle_backed_object() -> SpaceObject {
        SpaceObject {
            id: "25544".to_string(),
            name: "ISS (ZARYA)".to_string(),
            regime: OrbitRegime::Leo,
            state: OrbitalState {
                epoch_unix_seconds: 1_595_542_946,
                position_km: Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                velocity_km_s: Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            tle: Some(TleEphemeris {
                line1: "1 25544U 98067A   20194.88612269 -.00002218  00000-0 -31515-4 0  9992"
                    .to_string(),
                line2: "2 25544  51.6461 221.2784 0001413  89.1723 280.4612 15.49507896236008"
                    .to_string(),
            }),
            behavior: BehaviorSignature {
                nominal_rf_mhz: None,
                historical_maneuvers_per_week: 0.0,
                station_keeping_expected: false,
            },
            risk: RiskProfile {
                operator: Some("ISS".to_string()),
                mission_class: MissionClass::Civil,
                criticality: 0.5,
            },
        }
    }

    fn observed_state(position_km: Vector3, velocity_km_s: Vector3) -> OrbitalState {
        OrbitalState {
            epoch_unix_seconds: 1_800_000_060,
            position_km,
            velocity_km_s,
        }
    }

    fn normal_observation(object: &SpaceObject) -> Observation {
        Observation {
            object_id: object.id.clone(),
            observed_state: object.state.propagate_linear_seconds(60),
            rf_mhz: object.behavior.nominal_rf_mhz,
            maneuver_detected: false,
            source: ObservationSource::Synthetic,
        }
    }

    fn severe_observation(object: &SpaceObject) -> Observation {
        Observation {
            object_id: object.id.clone(),
            observed_state: observed_state(
                Vector3 {
                    x: 6_940.0,
                    y: 470.0,
                    z: 0.0,
                },
                Vector3 {
                    x: 0.05,
                    y: 7.63,
                    z: 0.0,
                },
            ),
            rf_mhz: Some(2_241.1),
            maneuver_detected: true,
            source: ObservationSource::Synthetic,
        }
    }

    fn close_neighbor_object() -> SpaceObject {
        SpaceObject {
            id: "SAT-002".to_string(),
            name: "Neighbor Sat".to_string(),
            regime: OrbitRegime::Leo,
            state: OrbitalState {
                epoch_unix_seconds: 1_800_000_000,
                position_km: Vector3 {
                    x: 6_900.0,
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
                operator: Some("Neighbor synthetic".to_string()),
                mission_class: MissionClass::Commercial,
                criticality: 0.72,
            },
        }
    }

    #[test]
    fn digital_twin_predicts_linear_future_state() {
        let twin = DigitalTwin::new(vec![sample_object()]);
        let predicted = twin
            .predicted_state("SAT-001", 1_800_000_060)
            .expect("known object");

        assert_eq!(predicted.epoch_unix_seconds, 1_800_000_060);
        assert!((predicted.position_km.x - 6_900.0).abs() < f64::EPSILON);
        assert!((predicted.position_km.y - 456.0).abs() < f64::EPSILON);
    }

    #[test]
    fn anomaly_engine_scores_position_velocity_rf_and_maneuver_signals() {
        let object = sample_object();
        let observation = severe_observation(&object);

        let report = AnomalyEngine::default().evaluate(&object, &observation);

        assert!(report.score > 0.8);
        assert!(report.signals.contains(&AnomalySignal::PositionDrift));
        assert!(report.signals.contains(&AnomalySignal::VelocityChange));
        assert!(report.signals.contains(&AnomalySignal::RadioFrequencyShift));
        assert!(report.signals.contains(&AnomalySignal::UnexpectedManeuver));
    }

    #[test]
    fn anticipation_engine_turns_anomaly_into_actionable_alert() {
        let object = sample_object();
        let twin = DigitalTwin::new(vec![object.clone()]);
        let observation = severe_observation(&object);

        let alert = AnticipationEngine::default()
            .evaluate_observation(&twin, &observation)
            .expect("known object");

        assert!(alert.risk_score > 0.7);
        assert!(alert.severity >= AlertSeverity::Warning);
        assert!((alert.predictions[0].horizon_hours - 72.0).abs() < f64::EPSILON);
        assert_eq!(
            alert.predictions[0].event,
            PredictedEvent::ContinuedAnomalousBehavior
        );
    }

    #[test]
    fn intelligence_layer_analyzes_object_from_latest_observation() {
        let object = sample_object();
        let mut layer = IntelligenceLayer::new(
            DigitalTwin::new(vec![object.clone()]),
            AnticipationEngine::default(),
        );
        layer.record_observation(severe_observation(&object));

        let response = layer.analyze_object(AnalyzeObjectRequest {
            object_id: object.id,
            timestamp_unix_seconds: Some(1_800_000_060),
        });

        assert_eq!(response.status, AnalysisStatus::AnomalyDetected);
        assert_eq!(response.behavior, "unexpected orbital adjustment");
        assert!(response.confidence > 0.9);
        assert!(response
            .recipients
            .contains(&IntelligenceRecipient::SpaceTrafficCoordination));
    }

    #[test]
    fn intelligence_layer_returns_ranked_space_overview() {
        let object = sample_object();
        let mut layer = IntelligenceLayer::new(
            DigitalTwin::new(vec![object.clone()]),
            AnticipationEngine::default(),
        );
        layer.record_observation(severe_observation(&object));

        let overview = layer.space_overview(SpaceOverviewRequest { window_hours: 24.0 });

        assert!((overview.window_hours - 24.0).abs() < f64::EPSILON);
        assert_eq!(overview.top_events.len(), 2);
        assert_eq!(overview.top_events[0].object_id, "SAT-001");
        assert_eq!(
            overview.top_events[0].event_type,
            RankedEventType::BehaviorShiftDetected
        );
        assert!(overview.top_events[0].risk > 0.8);
        assert!(overview.top_events[0].target_epoch_unix_seconds.is_some());
    }

    #[test]
    fn space_overview_includes_native_close_approach_events() {
        let primary = sample_object();
        let neighbor = close_neighbor_object();
        let mut layer = IntelligenceLayer::new(
            DigitalTwin::new(vec![primary.clone(), neighbor.clone()]),
            AnticipationEngine::default(),
        );
        layer.record_observation(normal_observation(&primary));
        layer.record_observation(normal_observation(&neighbor));

        let overview = layer.space_overview(SpaceOverviewRequest { window_hours: 24.0 });

        assert!(overview.top_events.iter().any(|event| {
            event.event_type == RankedEventType::PredictedCloseApproach
                && event.event_id.contains("native_close_approach")
                && event.summary.contains("Neighbor Sat")
        }));
    }

    #[test]
    fn analyze_object_promotes_pairwise_close_approach_into_notifications() {
        let primary = sample_object();
        let neighbor = close_neighbor_object();
        let mut layer = IntelligenceLayer::new(
            DigitalTwin::new(vec![primary.clone(), neighbor.clone()]),
            AnticipationEngine::default(),
        );
        layer.record_observation(normal_observation(&primary));
        layer.record_observation(normal_observation(&neighbor));

        let response = layer.analyze_object(AnalyzeObjectRequest {
            object_id: primary.id.clone(),
            timestamp_unix_seconds: Some(1_800_000_060),
        });

        assert_eq!(response.prediction.event, PredictedEvent::CloseApproach);
        assert!(response
            .decision
            .recipient_policy
            .escalation_rules
            .iter()
            .any(|rule| rule.rule_id == "predicted_close_approach_to_coordination"));
        assert!(response.decision.notifications.iter().any(|notification| {
            notification.recipient == IntelligenceRecipient::SpaceTrafficCoordination
                && notification
                    .notification_reason
                    .contains("projected within")
        }));
    }

    #[test]
    fn ranked_events_prefer_nearer_target_epoch_when_risk_matches() {
        let later_event = RankedEvent {
            event_id: "later".to_string(),
            object_id: "SAT-001".to_string(),
            object_name: "Test".to_string(),
            risk: 0.8,
            target_epoch_unix_seconds: Some(1_800_003_600),
            event_type: RankedEventType::PredictedCloseApproach,
            prediction: None,
            summary: "later".to_string(),
        };
        let earlier_event = RankedEvent {
            event_id: "earlier".to_string(),
            object_id: "SAT-002".to_string(),
            object_name: "Test".to_string(),
            risk: 0.8,
            target_epoch_unix_seconds: Some(1_800_000_600),
            event_type: RankedEventType::PredictedCloseApproach,
            prediction: None,
            summary: "earlier".to_string(),
        };

        let mut events = [later_event, earlier_event];
        events.sort_by(compare_ranked_events);

        assert_eq!(events[0].event_id, "earlier");
        assert_eq!(events[1].event_id, "later");
    }

    #[test]
    fn intelligence_layer_predicts_with_requested_horizon() {
        let object = sample_object();
        let layer = IntelligenceLayer::new(
            DigitalTwin::new(vec![object.clone()]),
            AnticipationEngine::default(),
        );

        let response = layer.predict(PredictRequest {
            object_id: object.id,
            horizon_hours: 36.0,
        });

        assert_eq!(response.predictions.len(), 1);
        assert!((response.predictions[0].horizon_hours - 36.0).abs() < f64::EPSILON);
        assert!((response.predictions[0].probability - 0.08).abs() < f64::EPSILON);
    }

    #[test]
    fn predict_response_includes_projected_state_and_sgp4_metadata_when_tle_is_available() {
        let object = tle_backed_object();
        let observation = Observation {
            object_id: object.id.clone(),
            observed_state: OrbitalState {
                epoch_unix_seconds: 1_595_542_946,
                position_km: Vector3 {
                    x: -4_681.422_784_831_853,
                    y: -4_663.517_498_114_294,
                    z: 1_173.198_631_786_711,
                },
                velocity_km_s: Vector3 {
                    x: 1.664_687_342_650_065,
                    y: -3.241_267_230_354_17,
                    z: -6.719_724_611_865_54,
                },
            },
            rf_mhz: None,
            maneuver_detected: false,
            source: ObservationSource::Catalog,
        };
        let mut layer = IntelligenceLayer::new(
            DigitalTwin::new(vec![object.clone()]),
            AnticipationEngine::default(),
        );
        layer.record_observation(observation);

        let response = layer.predict(PredictRequest {
            object_id: object.id,
            horizon_hours: 6.0,
        });

        assert_eq!(response.predictions.len(), 1);
        assert_eq!(response.predictions[0].propagation_model, "sgp4");
        assert_eq!(
            response.predictions[0].based_on_observation_epoch,
            Some(1_595_542_946)
        );
        assert!(response.predictions[0].predicted_state.is_some());
        assert!(response.predictions[0].explanation.contains("SGP4"));
    }

    #[test]
    fn small_deviation_remains_normal_but_auditable() {
        let object = sample_object();
        let mut layer = IntelligenceLayer::new(
            DigitalTwin::new(vec![object.clone()]),
            AnticipationEngine::default(),
        );
        layer.record_observation(normal_observation(&object));

        let response = layer.analyze_object(AnalyzeObjectRequest {
            object_id: object.id,
            timestamp_unix_seconds: Some(1_800_000_060),
        });

        assert_eq!(response.status, AnalysisStatus::Nominal);
        assert_eq!(response.finding.signals.len(), 0);
        assert_eq!(response.evidence_bundle.observations.len(), 1);
        assert_eq!(response.evidence_bundle.events.len(), 2);
        assert!(!response.evidence_bundle.bundle_hash.is_empty());
        assert_eq!(response.assessment.decision_version, "sss-decision-v1");
        assert!(response
            .decision
            .recipient_policy
            .escalation_rules
            .is_empty());
    }

    #[test]
    fn evidence_bundle_produces_replay_manifest_with_fixed_versions() {
        let object = sample_object();
        let mut layer = IntelligenceLayer::new(
            DigitalTwin::new(vec![object.clone()]),
            AnticipationEngine::default(),
        );
        layer.record_observation(severe_observation(&object));

        let response = layer.analyze_object(AnalyzeObjectRequest {
            object_id: object.id,
            timestamp_unix_seconds: Some(1_800_000_060),
        });
        let manifest = replay_manifest_for(&response.evidence_bundle);

        assert_eq!(manifest.object_id, "SAT-001");
        assert_eq!(
            manifest.event_ids.len(),
            response.evidence_bundle.events.len()
        );
        assert_eq!(
            manifest.versions.get("behavior_model").map(String::as_str),
            Some("sss-behavioral-mvp-v1")
        );
        assert!(!manifest.event_fingerprint.is_empty());
        assert!(!manifest.manifest_hash.is_empty());
    }

    #[test]
    fn single_signal_anomaly_has_lower_confidence_than_multi_signal_case() {
        let object = sample_object();
        let mut layer = IntelligenceLayer::new(
            DigitalTwin::new(vec![object.clone()]),
            AnticipationEngine::default(),
        );
        let mut observation = normal_observation(&object);
        observation.rf_mhz = Some(2_241.0);
        layer.record_observation(observation);

        let response = layer.analyze_object(AnalyzeObjectRequest {
            object_id: object.id,
            timestamp_unix_seconds: Some(1_800_000_060),
        });

        assert_eq!(
            response.finding.signals,
            vec![AnomalySignal::RadioFrequencyShift]
        );
        assert!(response.confidence < 0.8);
        assert!(response
            .assessment
            .rule_hits
            .iter()
            .any(|hit| hit.rule_id == "rf_shift"));
    }

    #[test]
    fn high_risk_prediction_creates_critical_operational_decision() {
        let mut object = sample_object();
        object.risk.criticality = 0.95;
        let mut layer = IntelligenceLayer::new(
            DigitalTwin::new(vec![object.clone()]),
            AnticipationEngine::default(),
        );
        layer.record_observation(severe_observation(&object));

        let response = layer.analyze_object(AnalyzeObjectRequest {
            object_id: object.id,
            timestamp_unix_seconds: Some(1_800_000_060),
        });

        assert_eq!(response.decision.severity, AlertSeverity::Critical);
        assert!(response.decision.risk_score > 0.9);
        assert!(response
            .decision
            .recipient_policy
            .escalation_rules
            .iter()
            .any(|rule| rule.rule_id == "critical_to_national_authority"));
    }

    #[test]
    fn defense_object_gets_institutional_escalation_reason() {
        let mut object = sample_object();
        object.risk.mission_class = MissionClass::Defense;
        let mut layer = IntelligenceLayer::new(
            DigitalTwin::new(vec![object.clone()]),
            AnticipationEngine::default(),
        );
        layer.record_observation(normal_observation(&object));

        let response = layer.analyze_object(AnalyzeObjectRequest {
            object_id: object.id,
            timestamp_unix_seconds: Some(1_800_000_060),
        });

        assert!(
            response
                .decision
                .notifications
                .iter()
                .any(|notification| notification.recipient
                    == IntelligenceRecipient::NationalAuthority)
        );
        assert!(response
            .decision
            .recipient_policy
            .escalation_rules
            .iter()
            .any(|rule| rule.rule_id == "defense_to_national_authority"));
    }

    #[test]
    fn replay_response_from_evidence_bundle_rebuilds_decision_and_diff() {
        let object = sample_object();
        let mut layer = IntelligenceLayer::new(
            DigitalTwin::new(vec![object.clone()]),
            AnticipationEngine::default(),
        );
        layer.record_observation(severe_observation(&object));

        let response = layer.analyze_object(AnalyzeObjectRequest {
            object_id: object.id.clone(),
            timestamp_unix_seconds: Some(1_800_000_060),
        });
        let replayed = replay_response_from_evidence_bundle(&response.evidence_bundle, &object);
        let diff = replay_diff(
            Some(&response.assessment),
            &replayed.assessment,
            Some(&response.decision),
            &replayed.decision,
        );

        assert_eq!(replayed.object_id, object.id);
        assert_eq!(replayed.assessment.status, response.assessment.status);
        assert!(diff.assessment.confidence_delta.abs() < f64::EPSILON);
        assert!(diff.decision.risk_score_delta.is_some());
    }
}
