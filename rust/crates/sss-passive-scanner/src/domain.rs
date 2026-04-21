use serde::{Deserialize, Serialize};
use serde_json::Value;
use sss_core::{
    EvidenceBundle, IntelligenceAssessment, ObservationFinding, OperationalDecision, RankedEvent,
    ReplayManifest,
};
use sss_site_registry::{Criticality, Site, SiteType};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PassiveSourceKind {
    Adsb,
    Satellite,
    Weather,
    FireSmoke,
    Notam,
    Orbital,
    InfraMap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PassiveThreatType {
    LowAltitudeOverflight,
    RestrictedAirspaceActivity,
    HailExposure,
    WindLoadRisk,
    IrradianceDropExpected,
    SmokeExposure,
    OrbitalReentryContext,
    SurfaceChangeDetected,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenInfraSiteSeed {
    pub name: String,
    pub site_type: SiteType,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation_m: f32,
    pub timezone: String,
    pub country_code: String,
    pub criticality: Criticality,
    pub operator_name: String,
    pub observation_radius_km: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PassiveSiteProfile {
    pub site: Site,
    pub observation_radius_km: f64,
    pub source_references: Vec<String>,
    pub passive_tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PassiveObservation {
    pub observation_id: Uuid,
    pub source_kind: PassiveSourceKind,
    pub observed_at_unix_seconds: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude_m: f32,
    pub coverage_radius_km: f64,
    pub severity_hint: f64,
    pub summary: String,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PassiveEvent {
    pub event_id: Uuid,
    pub site_id: Uuid,
    pub site_name: String,
    pub source_kind: PassiveSourceKind,
    pub threat_type: PassiveThreatType,
    #[serde(default)]
    pub phenomenon_key: String,
    pub observed_at_unix_seconds: i64,
    pub distance_to_site_km: f64,
    pub risk_score: f64,
    pub summary: String,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PassiveRiskRecord {
    pub site_id: Uuid,
    pub site_name: String,
    pub window_start_unix_seconds: i64,
    pub window_end_unix_seconds: i64,
    pub cumulative_risk: f64,
    pub peak_risk: f64,
    pub event_count: usize,
    pub dominant_threats: Vec<PassiveThreatType>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternSignal {
    pub site_id: Uuid,
    pub site_name: String,
    pub threat_type: PassiveThreatType,
    pub recurring_events: usize,
    pub average_risk: f64,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PassiveCoreEnvelope {
    pub finding: ObservationFinding,
    pub assessment: IntelligenceAssessment,
    pub decision: OperationalDecision,
    pub evidence_bundle: EvidenceBundle,
    pub replay_manifest: ReplayManifest,
    pub ranked_event: RankedEvent,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PassiveScanInput {
    pub window_start_unix_seconds: i64,
    pub window_end_unix_seconds: i64,
    pub infra_sites: Vec<OpenInfraSiteSeed>,
    pub adsb_observations: Vec<AdsbFlightObservation>,
    pub weather_observations: Vec<WeatherObservation>,
    pub fire_smoke_observations: Vec<FireSmokeObservation>,
    pub orbital_observations: Vec<OrbitalPassObservation>,
    pub satellite_observations: Vec<SatelliteSceneObservation>,
    pub notam_observations: Vec<NotamObservation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PassiveScanOutput {
    pub sites: Vec<PassiveSiteProfile>,
    pub observations: Vec<PassiveObservation>,
    pub events: Vec<PassiveEvent>,
    pub risk_history: Vec<PassiveRiskRecord>,
    pub patterns: Vec<PatternSignal>,
    pub core_envelopes: Vec<PassiveCoreEnvelope>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdsbFlightObservation {
    pub flight_id: String,
    pub observed_at_unix_seconds: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude_m: f32,
    pub speed_kts: f32,
    pub vertical_rate_m_s: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherObservation {
    pub observed_at_unix_seconds: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub wind_kph: f32,
    pub hail_probability: f32,
    pub irradiance_drop_ratio: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FireSmokeObservation {
    pub observed_at_unix_seconds: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub fire_radiative_power: f32,
    pub smoke_density_index: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrbitalPassObservation {
    pub object_id: String,
    pub observed_at_unix_seconds: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub reentry_probability: f32,
    pub footprint_radius_km: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SatelliteSceneObservation {
    pub scene_id: String,
    pub observed_at_unix_seconds: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub change_score: f32,
    pub cloud_cover_ratio: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotamObservation {
    pub notam_id: String,
    pub observed_at_unix_seconds: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_km: f64,
    pub restriction_summary: String,
    pub severity_hint: f32,
}
