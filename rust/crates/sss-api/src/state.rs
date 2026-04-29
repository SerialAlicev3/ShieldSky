use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::handlers::{EventsTimelineBucket, EventsTimelineResponse};
use sss_core::{
    replay_manifest_for, AlertSeverity, AnalyzeObjectRequest, AnalyzeObjectResponse,
    AnticipationEngine, BehaviorSignature, DigitalTwin, EvidenceBundle, MissionClass,
    NotificationDecision, Observation, ObservationSource, OrbitRegime, OrbitalState,
    PredictedEvent, Prediction, ReplayManifest, RiskProfile, SpaceObject, Vector3,
};
use sss_forecast::{
    load_forecast, recommend_action, scenario_forecast, solar_forecast, LoadForecastResponse,
    OrchestratorRecommendation, OrchestratorRecommendationRequest, ScenarioForecastResponse,
    SiteForecastInput, SolarForecastResponse, WeatherNowSignal,
};
use sss_ingest::{normalize_tle_record, parse_tle_records, IngestError};
use sss_passive_scanner::{
    AdsbFlightObservation, FireSmokeObservation, OpenInfraSiteSeed, PassiveEvent,
    PassiveObservation, PassiveRiskRecord, PassiveScanInput, PassiveScanOutput, PassiveScanner,
    PassiveSiteProfile, PassiveSourceKind, PatternSignal, WeatherObservation,
};
use sss_site_registry::{Criticality, Site, SiteType};
use sss_storage::{
    CanonicalPassiveEvent, IngestBatchLog, NotificationDeliveryLog, PassiveRecommendationReview,
    PassiveRecommendationReviewState, PassiveRegionLease, PassiveRegionRunLog,
    PassiveRegionRunStatus, PassiveRegionTarget, PassiveRunOrigin, PassiveSeedClassificationStatus,
    PassiveSeedRecord, PassiveSeedStatus, PassiveSourceHealthSample, PassiveWorkerHeartbeat,
    PredictionSnapshot, RequestResponseLog, SqliteStore, StorageError,
};

#[derive(Debug, Clone)]
pub struct AppState {
    intelligence: Arc<RwLock<sss_core::IntelligenceLayer>>,
    passive_scanner: PassiveScanner,
    storage: SqliteStore,
    http_client: reqwest::Client,
    webhook_endpoint: Option<String>,
    webhook_delivery_policy: WebhookDeliveryPolicy,
    nasa_api_key: String,
    nasa_api_base_url: String,
    open_meteo_base_url: String,
    firms_api_base_url: String,
    firms_map_key: Option<String>,
    firms_source: String,
    opensky_api_base_url: String,
    opensky_bearer_token: Option<String>,
    overpass_api_base_url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WebhookDeliveryPolicy {
    pub max_attempts: u32,
    pub retry_delay: Duration,
}

impl Default for WebhookDeliveryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            retry_delay: Duration::from_millis(500),
        }
    }
}

#[derive(Debug)]
pub enum AppError {
    Ingest(IngestError),
    Storage(StorageError),
    Serde(serde_json::Error),
    InvalidRequest(String),
    InvalidDateRange(String),
    NasaApiUnavailable(String),
    SourceUnavailable(String),
    NotificationUnavailable(String),
    EventNotFound(String),
    SiteNotFound(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ingest(error) => write!(f, "{error}"),
            Self::Storage(error) => write!(f, "{error}"),
            Self::Serde(error) => write!(f, "{error}"),
            Self::InvalidRequest(message)
            | Self::InvalidDateRange(message)
            | Self::NasaApiUnavailable(message)
            | Self::SourceUnavailable(message)
            | Self::NotificationUnavailable(message)
            | Self::EventNotFound(message)
            | Self::SiteNotFound(message) => {
                write!(f, "{message}")
            }
        }
    }
}

impl std::error::Error for AppError {}

impl From<IngestError> for AppError {
    fn from(value: IngestError) -> Self {
        Self::Ingest(value)
    }
}

impl From<StorageError> for AppError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestOutcome {
    pub records_received: usize,
    pub observations_created: usize,
    pub object_ids: Vec<String>,
    pub skipped_duplicate: bool,
    pub freshness_seconds: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestSourceStatus {
    pub source: String,
    pub latest_request_id: String,
    pub latest_timestamp_unix_seconds: i64,
    pub records_received: usize,
    pub observations_created: usize,
    pub object_count: usize,
    pub freshness_seconds: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RankedEventsFilter {
    pub object_id: Option<String>,
    pub event_type: Option<sss_core::RankedEventType>,
    pub future_only: bool,
    pub after_unix_seconds: Option<i64>,
    pub before_unix_seconds: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NotificationDispatchResponse {
    pub analysis: AnalyzeObjectResponse,
    pub deliveries: Vec<NotificationDeliveryLog>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EventDispatchResponse {
    pub event: sss_core::RankedEvent,
    pub dispatch: NotificationDispatchResponse,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ApodBriefing {
    pub title: String,
    pub date: String,
    pub explanation: String,
    pub media_type: String,
    pub url: String,
    pub hdurl: Option<String>,
    pub copyright: Option<String>,
    pub service_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NeoRiskObject {
    pub neo_reference_id: String,
    pub name: String,
    pub nasa_jpl_url: Option<String>,
    pub hazardous: bool,
    pub close_approach_date: Option<String>,
    pub miss_distance_km: Option<f64>,
    pub relative_velocity_km_s: Option<f64>,
    pub estimated_diameter_min_m: Option<f64>,
    pub estimated_diameter_max_m: Option<f64>,
    pub priority_score: f64,
    pub briefing_summary: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NeoRiskBriefing {
    pub generated_at_unix_seconds: i64,
    pub start_date: String,
    pub end_date: String,
    pub total_objects: usize,
    pub hazardous_objects: usize,
    pub highest_priority: Vec<NeoRiskObject>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NeoRiskFeed {
    pub generated_at_unix_seconds: i64,
    pub start_date: String,
    pub end_date: String,
    pub total_objects: usize,
    pub hazardous_objects: usize,
    pub objects: Vec<NeoRiskObject>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CloseApproachCandidate {
    pub counterpart_object_id: String,
    pub counterpart_object_name: String,
    pub target_epoch_unix_seconds: i64,
    pub miss_distance_km: f64,
    pub priority_score: f64,
    pub propagation_model: String,
    pub summary: String,
    pub primary_state: OrbitalState,
    pub counterpart_state: OrbitalState,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CloseApproachBriefing {
    pub generated_at_unix_seconds: i64,
    pub object_id: String,
    pub object_name: String,
    pub base_epoch_unix_seconds: i64,
    pub horizon_hours: f64,
    pub threshold_km: f64,
    pub scanned_objects: usize,
    pub close_approaches: Vec<CloseApproachCandidate>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveLiveScanRequest {
    pub window_hours: Option<u64>,
    pub infra_sites: Vec<OpenInfraSiteSeed>,
    pub include_adsb: Option<bool>,
    pub include_weather: Option<bool>,
    pub include_fire_smoke: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveLiveSourceStatus {
    pub source_kind: PassiveSourceKind,
    pub fetched: bool,
    pub observations_collected: usize,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveLiveScanResponse {
    pub generated_at_unix_seconds: i64,
    pub window_start_unix_seconds: i64,
    pub window_end_unix_seconds: i64,
    pub source_statuses: Vec<PassiveLiveSourceStatus>,
    pub scan: PassiveScanOutput,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveTemporalPoint {
    pub window_start_unix_seconds: i64,
    pub window_end_unix_seconds: i64,
    pub cumulative_risk: f64,
    pub peak_risk: f64,
    pub event_count: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RiskHistoryNarrative {
    pub site_id: String,
    pub site_name: String,
    pub window_days: u32,
    pub generated_at_unix_seconds: i64,
    pub event_count: usize,
    pub recurring_pattern_count: usize,
    pub critical_event_count: usize,
    pub average_cumulative_risk: f64,
    pub latest_peak_risk: f64,
    pub observation_confidence: f64,
    pub risk_direction: String,
    pub narrative: String,
    pub provenance: NarrativeProvenance,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NarrativeProvenance {
    pub canonical_event_ids: Vec<String>,
    pub bundle_hashes: Vec<String>,
    pub manifest_hashes: Vec<String>,
    pub site_overview_path: String,
    pub site_narrative_path: String,
    pub canonical_event_paths: Vec<String>,
    pub evidence_paths: Vec<String>,
    pub replay_paths: Vec<String>,
    pub top_drivers: Vec<NarrativeProvenanceDriver>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NarrativeProvenanceDriver {
    pub canonical_event_id: String,
    pub event_type: String,
    pub status: String,
    pub risk_score: f64,
    pub confidence: f64,
    pub support_count: usize,
    pub summary: String,
    pub bundle_hashes: Vec<String>,
    pub manifest_hashes: Vec<String>,
    pub canonical_event_path: String,
    pub evidence_paths: Vec<String>,
    pub replay_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveSiteOverview {
    pub site: PassiveSiteProfile,
    pub seed_lifecycle: Option<PassiveSeedRecord>,
    pub recent_events: Vec<PassiveEvent>,
    pub latest_risk: Option<PassiveRiskRecord>,
    pub recurring_patterns: Vec<PatternSignal>,
    pub temporal_evolution: Vec<PassiveTemporalPoint>,
    pub risk_history_narrative: RiskHistoryNarrative,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveInfraDiscoverRequest {
    pub south: f64,
    pub west: f64,
    pub north: f64,
    pub east: f64,
    pub site_types: Option<Vec<SiteType>>,
    pub timezone: Option<String>,
    pub country_code: Option<String>,
    pub default_operator_name: Option<String>,
    pub default_criticality: Option<Criticality>,
    pub observation_radius_km: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveInfraDiscoverResponse {
    pub generated_at_unix_seconds: i64,
    pub source: String,
    pub south: f64,
    pub west: f64,
    pub north: f64,
    pub east: f64,
    pub seeds: Vec<OpenInfraSiteSeed>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveDiscoverAndScanRequest {
    pub discovery: PassiveInfraDiscoverRequest,
    pub window_hours: Option<u64>,
    pub include_adsb: Option<bool>,
    pub include_weather: Option<bool>,
    pub include_fire_smoke: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveDiscoverAndScanResponse {
    pub generated_at_unix_seconds: i64,
    pub discovery: PassiveInfraDiscoverResponse,
    pub live_scan: PassiveLiveScanResponse,
    pub execution_summary: PassiveExecutionSummary,
    pub site_overviews: Vec<PassiveSiteOverview>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveExecutionSummary {
    pub discovered_seed_count: usize,
    pub scanned_seed_count: usize,
    pub event_count: usize,
    pub elevated_seed_count: usize,
    pub critical_event_count: usize,
    pub highest_scan_priority: f64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveScheduleScanRequest {
    pub limit: Option<usize>,
    pub window_hours: Option<u64>,
    pub include_adsb: Option<bool>,
    pub include_weather: Option<bool>,
    pub include_fire_smoke: Option<bool>,
    pub minimum_priority: Option<f64>,
    pub force: Option<bool>,
    pub dry_run: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveScheduledSeed {
    pub seed_key: String,
    pub site_name: String,
    pub site_type: SiteType,
    pub scan_priority: f64,
    pub cadence_seconds: i64,
    pub overdue_seconds: i64,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveScheduleScanResponse {
    pub generated_at_unix_seconds: i64,
    pub evaluated_seed_count: usize,
    pub selected_seed_count: usize,
    pub skipped_seed_count: usize,
    pub skipped_not_due_count: usize,
    pub skipped_below_priority_count: usize,
    pub dry_run: bool,
    pub planned_seeds: Vec<PassiveScheduledSeed>,
    pub live_scan: Option<PassiveLiveScanResponse>,
    pub execution_summary: Option<PassiveExecutionSummary>,
    pub site_overviews: Vec<PassiveSiteOverview>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveRegionTargetRequest {
    pub region_id: Option<String>,
    pub name: String,
    pub south: f64,
    pub west: f64,
    pub north: f64,
    pub east: f64,
    pub site_types: Option<Vec<SiteType>>,
    pub timezone: Option<String>,
    pub country_code: Option<String>,
    pub default_operator_name: Option<String>,
    pub default_criticality: Option<Criticality>,
    pub observation_radius_km: Option<f64>,
    pub discovery_cadence_seconds: Option<i64>,
    pub scan_limit: Option<usize>,
    pub minimum_priority: Option<f64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub struct PassiveRegionDefaultsBootstrapRequest {
    pub overwrite_existing: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveRegionDefaultsBootstrapResponse {
    pub generated_at_unix_seconds: i64,
    pub total_defaults: usize,
    pub inserted_count: usize,
    pub updated_count: usize,
    pub skipped_count: usize,
    pub inserted_region_ids: Vec<String>,
    pub updated_region_ids: Vec<String>,
    pub skipped_region_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveRegionRunRequest {
    pub region_ids: Option<Vec<String>>,
    pub force_discovery: Option<bool>,
    pub dry_run: Option<bool>,
    pub window_hours: Option<u64>,
    pub include_adsb: Option<bool>,
    pub include_weather: Option<bool>,
    pub include_fire_smoke: Option<bool>,
}

/// Provides a curated list of default passive region target requests for bootstrapping passive-region configuration.
///
/// Each returned `PassiveRegionTargetRequest` encodes a geographic bounding box, preferred site types, inferred
/// criticality, discovery cadence (seconds), per-run scan limit, minimum scan priority, and an optional
/// observation radius (kilometers) used by the region bootstrap workflow.
///
/// # Examples
///
/// ```
/// let defaults = default_passive_region_requests();
/// assert!(!defaults.is_empty());
/// // ensure entries contain region ids (slugs)
/// assert!(defaults.iter().all(|r| !r.region_id.is_empty()));
/// ```
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn default_passive_region_requests() -> Vec<PassiveRegionTargetRequest> {
    let broad_site_types = Some(vec![
        SiteType::Substation,
        SiteType::SolarPlant,
        SiteType::DataCenter,
    ]);
    vec![
        default_region(
            "portugal-iberian-watch-36.900--9.500",
            "Portugal Iberian Watch",
            36.9,
            -9.5,
            42.1,
            -6.2,
            Some("PT"),
            broad_site_types.clone(),
            Some(Criticality::Critical),
            86_400,
            220,
            0.0,
            Some(42.0),
        ),
        default_region(
            "alentejo-solar-belt-37.200--8.800",
            "Alentejo Solar Belt",
            37.2,
            -8.8,
            38.8,
            -7.0,
            Some("PT"),
            Some(vec![SiteType::SolarPlant, SiteType::Substation]),
            Some(Criticality::Critical),
            86_400,
            220,
            0.0,
            Some(36.0),
        ),
        default_region(
            "iberia-strategic-corridor",
            "Iberia Strategic Corridor",
            35.7,
            -10.2,
            43.9,
            3.5,
            None,
            broad_site_types.clone(),
            Some(Criticality::High),
            129_600,
            140,
            0.08,
            Some(45.0),
        ),
        default_region(
            "western-europe-grid-arc",
            "Western Europe Grid Arc",
            43.0,
            -5.5,
            56.2,
            12.8,
            None,
            broad_site_types.clone(),
            Some(Criticality::High),
            172_800,
            120,
            0.16,
            Some(50.0),
        ),
        default_region(
            "central-europe-industrial-belt",
            "Central Europe Industrial Belt",
            46.2,
            8.0,
            55.4,
            24.5,
            None,
            broad_site_types.clone(),
            Some(Criticality::High),
            172_800,
            120,
            0.18,
            Some(48.0),
        ),
        default_region(
            "balkans-eastern-med-gateway",
            "Balkans Eastern Med Gateway",
            36.0,
            13.5,
            47.8,
            30.5,
            None,
            broad_site_types.clone(),
            Some(Criticality::High),
            172_800,
            110,
            0.18,
            Some(48.0),
        ),
        default_region(
            "ukraine-black-sea-corridor",
            "Ukraine Black Sea Corridor",
            43.0,
            22.0,
            52.5,
            40.0,
            Some("UA"),
            broad_site_types.clone(),
            Some(Criticality::Critical),
            172_800,
            110,
            0.22,
            Some(50.0),
        ),
        default_region(
            "russia-west-strategic-watch",
            "Russia West Strategic Watch",
            54.0,
            25.0,
            61.5,
            40.5,
            Some("RU"),
            broad_site_types.clone(),
            Some(Criticality::High),
            259_200,
            90,
            0.24,
            Some(55.0),
        ),
        default_region(
            "caucasus-caspian-arc",
            "Caucasus Caspian Arc",
            36.2,
            41.0,
            47.5,
            55.8,
            None,
            broad_site_types.clone(),
            Some(Criticality::High),
            259_200,
            90,
            0.22,
            Some(48.0),
        ),
        default_region(
            "turkey-anatolian-grid",
            "Turkey Anatolian Grid",
            35.6,
            25.5,
            42.7,
            45.5,
            Some("TR"),
            broad_site_types.clone(),
            Some(Criticality::High),
            172_800,
            100,
            0.18,
            Some(46.0),
        ),
        default_region(
            "levant-eastern-med-watch",
            "Levant Eastern Med Watch",
            29.2,
            31.0,
            37.5,
            39.8,
            None,
            broad_site_types.clone(),
            Some(Criticality::High),
            172_800,
            100,
            0.18,
            Some(44.0),
        ),
        default_region(
            "arabian-peninsula-critical-corridor",
            "Arabian Peninsula Critical Corridor",
            15.0,
            34.0,
            32.8,
            56.8,
            None,
            broad_site_types.clone(),
            Some(Criticality::High),
            259_200,
            100,
            0.24,
            Some(52.0),
        ),
        default_region(
            "gulf-infrastructure-belt",
            "Gulf Infrastructure Belt",
            22.0,
            47.0,
            31.8,
            58.5,
            None,
            broad_site_types.clone(),
            Some(Criticality::Critical),
            172_800,
            110,
            0.22,
            Some(46.0),
        ),
        default_region(
            "egypt-red-sea-gate",
            "Egypt Red Sea Gate",
            21.2,
            24.0,
            32.5,
            37.5,
            Some("EG"),
            broad_site_types.clone(),
            Some(Criticality::High),
            172_800,
            90,
            0.2,
            Some(44.0),
        ),
        default_region(
            "maghreb-atlantic-solar-arc",
            "Maghreb Atlantic Solar Arc",
            27.0,
            -12.5,
            37.0,
            11.8,
            None,
            broad_site_types.clone(),
            Some(Criticality::High),
            259_200,
            100,
            0.22,
            Some(52.0),
        ),
        default_region(
            "iran-plateau-strategic-watch",
            "Iran Plateau Strategic Watch",
            24.0,
            43.0,
            40.0,
            64.5,
            Some("IR"),
            broad_site_types,
            Some(Criticality::Critical),
            259_200,
            100,
            0.24,
            Some(55.0),
        ),
    ]
}

/// Construct a `PassiveRegionTargetRequest` for a default region using the given bounding box and configuration.
///
/// The returned request uses the supplied region identifier, name, bbox coordinates, and optional overrides
/// for country code, site types, criticality, and observation radius. Fields not provided here are left as
/// `None`. The request is enabled and includes the provided discovery cadence, scan limit, and minimum priority.
///
/// # Examples
///
/// ```
/// let req = default_region(
///     "region-1",
///     "Test Region",
///     -10.0, -20.0, 10.0, 20.0,
///     Some("US"),
///     Some(vec![SiteType::Solar]),
///     Some(Criticality::High),
///     86400,
///     25,
///     0.1,
///     Some(15.0),
/// );
/// assert_eq!(req.region_id.as_deref(), Some("region-1"));
/// assert_eq!(req.enabled, Some(true));
/// assert_eq!(req.discovery_cadence_seconds, Some(86400));
/// ```
fn default_region(
    region_id: &str,
    name: &str,
    south: f64,
    west: f64,
    north: f64,
    east: f64,
    country_code: Option<&str>,
    site_types: Option<Vec<SiteType>>,
    default_criticality: Option<Criticality>,
    discovery_cadence_seconds: i64,
    scan_limit: usize,
    minimum_priority: f64,
    observation_radius_km: Option<f64>,
) -> PassiveRegionTargetRequest {
    PassiveRegionTargetRequest {
        region_id: Some(region_id.to_string()),
        name: name.to_string(),
        south,
        west,
        north,
        east,
        site_types,
        timezone: None,
        country_code: country_code.map(std::string::ToString::to_string),
        default_operator_name: None,
        default_criticality,
        observation_radius_km,
        discovery_cadence_seconds: Some(discovery_cadence_seconds),
        scan_limit: Some(scan_limit),
        minimum_priority: Some(minimum_priority),
        enabled: Some(true),
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveRegionRunRecord {
    pub region: PassiveRegionTarget,
    pub due: bool,
    pub skipped_reason: Option<String>,
    pub discovery: Option<PassiveInfraDiscoverResponse>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveRegionRunResponse {
    pub generated_at_unix_seconds: i64,
    pub evaluated_region_count: usize,
    pub discovered_region_count: usize,
    pub skipped_region_count: usize,
    pub discovered_seed_count: usize,
    pub scheduler: Option<PassiveScheduleScanResponse>,
    pub regions: Vec<PassiveRegionRunRecord>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveRegionSiteSummary {
    pub seed_key: String,
    pub site_id: Option<String>,
    pub site_name: String,
    pub site_type: SiteType,
    pub criticality: Criticality,
    pub scan_priority: f64,
    pub observation_confidence: f64,
    pub recent_event_count: usize,
    pub critical_event_count: usize,
    pub latest_peak_risk: Option<f64>,
    pub risk_direction: String,
    pub narrative: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveRegionOverview {
    pub generated_at_unix_seconds: i64,
    pub region: PassiveRegionTarget,
    pub seed_count: usize,
    pub observed_seed_count: usize,
    pub elevated_seed_count: usize,
    pub recent_event_count: usize,
    pub critical_event_count: usize,
    pub highest_scan_priority: f64,
    pub average_observation_confidence: f64,
    pub discovery_due: bool,
    pub discovery_overdue_seconds: i64,
    pub top_sites: Vec<PassiveRegionSiteSummary>,
    pub narrative: String,
    pub provenance: PassiveRegionNarrativeProvenance,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveRegionNarrativeProvenance {
    pub region_overview_path: String,
    pub semantic_timeline_path: String,
    pub operational_timeline_path: String,
    pub remediation_path: String,
    pub runs_path: String,
    pub source_health_path: String,
    pub top_site_overview_paths: Vec<String>,
    pub top_site_narrative_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveOperationsStatus {
    pub generated_at_unix_seconds: i64,
    pub total_region_count: usize,
    pub enabled_region_count: usize,
    pub due_region_count: usize,
    pub active_region_lease_count: usize,
    pub stale_region_lease_count: usize,
    pub worker_heartbeat_count: usize,
    pub stale_worker_heartbeat_count: usize,
    pub seed_count: usize,
    pub observed_seed_count: usize,
    pub elevated_seed_count: usize,
    pub latest_region_run: Option<PassiveRegionRunLog>,
    pub active_region_leases: Vec<PassiveRegionLease>,
    pub worker_heartbeats: Vec<PassiveWorkerHeartbeat>,
    pub recommendation: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveWorkerDiagnostics {
    pub generated_at_unix_seconds: i64,
    pub region_id: Option<String>,
    pub stale_cutoff_unix_seconds: i64,
    pub total_worker_heartbeat_count: usize,
    pub stale_worker_heartbeat_count: usize,
    pub active_worker_heartbeat_count: usize,
    pub active_region_lease_count: usize,
    pub stale_region_lease_count: usize,
    pub stale_worker_heartbeats: Vec<PassiveWorkerHeartbeat>,
    pub active_region_leases: Vec<PassiveRegionLeaseDiagnostic>,
    pub stale_region_leases: Vec<PassiveRegionLeaseDiagnostic>,
    pub region_metrics: Vec<PassiveWorkerRegionMetrics>,
    pub source_metrics: Vec<PassiveWorkerSourceMetrics>,
    pub recommendation: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveRegionLeaseDiagnostic {
    pub region_id: String,
    pub worker_id: String,
    pub run_id: String,
    pub acquired_at_unix_seconds: i64,
    pub heartbeat_at_unix_seconds: i64,
    pub expires_at_unix_seconds: i64,
    pub heartbeat_lag_seconds: i64,
    pub expires_in_seconds: i64,
    pub stale: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveWorkerRegionMetrics {
    pub region_id: String,
    pub run_count: usize,
    pub completed_run_count: usize,
    pub partial_run_count: usize,
    pub failed_run_count: usize,
    pub discovered_seed_count: usize,
    pub selected_seed_count: usize,
    pub event_count: usize,
    pub source_error_count: usize,
    pub active_lease_count: usize,
    pub stale_lease_count: usize,
    pub active_worker_count: usize,
    pub stale_worker_count: usize,
    pub latest_run_status: Option<String>,
    pub last_finished_at_unix_seconds: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveWorkerSourceMetrics {
    pub source: String,
    pub region_id: Option<String>,
    pub sample_count: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub consecutive_failure_count: usize,
    pub success_rate: f64,
    pub reliability_score: f64,
    pub health_status: PassiveSourceHealthStatus,
    pub latest_generated_at_unix_seconds: Option<i64>,
    pub staleness_seconds: Option<i64>,
    pub last_error: Option<String>,
    pub recovery_hint: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PassiveSourceHealthStatus {
    Healthy,
    Watch,
    Degraded,
    Stale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PassiveSourceReadinessLevel {
    Ready,
    AwaitingData,
    NeedsConfig,
    Degraded,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PassiveSourceReadiness {
    pub source_id: String,
    pub label: String,
    pub configured: bool,
    pub enabled: bool,
    pub readiness: PassiveSourceReadinessLevel,
    pub reason: String,
    pub latest_sample_at_unix_seconds: Option<i64>,
    pub sample_count: usize,
    pub last_error: Option<String>,
}

#[derive(Debug, Default)]
struct PassiveWorkerRegionMetricsAccumulator {
    run_count: usize,
    completed_run_count: usize,
    partial_run_count: usize,
    failed_run_count: usize,
    discovered_seed_count: usize,
    selected_seed_count: usize,
    event_count: usize,
    source_error_count: usize,
    active_lease_count: usize,
    stale_lease_count: usize,
    active_worker_count: usize,
    stale_worker_count: usize,
    latest_run_status: Option<String>,
    last_finished_at_unix_seconds: Option<i64>,
}

#[derive(Debug, Default)]
struct PassiveWorkerSourceMetricsAccumulator {
    sample_count: usize,
    success_count: usize,
    failure_count: usize,
    consecutive_failure_count: usize,
    failure_streak_open: bool,
    latest_generated_at_unix_seconds: Option<i64>,
    last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct WebhookNotificationPayload {
    request_id: String,
    object_id: String,
    severity: AlertSeverity,
    risk_score: f64,
    recommendation: String,
    recipient_policy_id: String,
    notification: NotificationDecision,
    evidence_bundle_hash: String,
    prediction: sss_core::Prediction,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct NasaApodApiResponse {
    title: String,
    date: String,
    explanation: String,
    media_type: String,
    url: String,
    hdurl: Option<String>,
    copyright: Option<String>,
    service_version: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct NasaNeoWsFeedResponse {
    element_count: usize,
    near_earth_objects: std::collections::BTreeMap<String, Vec<NasaNeoWsObject>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct NasaNeoWsObject {
    id: String,
    name: String,
    nasa_jpl_url: Option<String>,
    is_potentially_hazardous_asteroid: bool,
    estimated_diameter: NasaNeoWsEstimatedDiameter,
    close_approach_data: Vec<NasaNeoWsCloseApproach>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct NasaNeoWsEstimatedDiameter {
    meters: NasaNeoWsDiameterRange,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct NasaNeoWsDiameterRange {
    estimated_diameter_min: f64,
    estimated_diameter_max: f64,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct NasaNeoWsCloseApproach {
    close_approach_date: Option<String>,
    relative_velocity: NasaNeoWsVelocity,
    miss_distance: NasaNeoWsMissDistance,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct NasaNeoWsVelocity {
    kilometers_per_second: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct NasaNeoWsMissDistance {
    kilometers: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct OpenMeteoForecastResponse {
    current: Option<OpenMeteoCurrent>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct OpenMeteoCurrent {
    time: i64,
    wind_speed_10m: Option<f32>,
    precipitation_probability: Option<f32>,
    shortwave_radiation: Option<f32>,
    cloud_cover: Option<f32>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct OpenSkyStatesResponse {
    time: i64,
    states: Option<Vec<Vec<serde_json::Value>>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct OverpassResponse {
    elements: Vec<OverpassElement>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct OverpassElement {
    id: i64,
    #[serde(rename = "type")]
    element_type: String,
    lat: Option<f64>,
    lon: Option<f64>,
    center: Option<OverpassCenter>,
    tags: Option<std::collections::BTreeMap<String, String>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct OverpassCenter {
    lat: f64,
    lon: f64,
}

impl AppState {
    #[must_use]
    pub fn new(intelligence: sss_core::IntelligenceLayer, storage: SqliteStore) -> Self {
        Self::with_http_client(intelligence, storage, default_http_client())
    }

    #[must_use]
    pub fn with_http_client(
        intelligence: sss_core::IntelligenceLayer,
        storage: SqliteStore,
        http_client: reqwest::Client,
    ) -> Self {
        Self {
            intelligence: Arc::new(RwLock::new(intelligence)),
            passive_scanner: PassiveScanner::new(),
            storage,
            http_client,
            webhook_endpoint: None,
            webhook_delivery_policy: WebhookDeliveryPolicy::default(),
            nasa_api_key: "DEMO_KEY".to_string(),
            nasa_api_base_url: "https://api.nasa.gov".to_string(),
            open_meteo_base_url: "https://api.open-meteo.com".to_string(),
            firms_api_base_url: "https://firms.modaps.eosdis.nasa.gov".to_string(),
            firms_map_key: None,
            firms_source: "VIIRS_SNPP_NRT".to_string(),
            opensky_api_base_url: "https://opensky-network.org/api".to_string(),
            opensky_bearer_token: None,
            overpass_api_base_url: "https://overpass-api.de/api/interpreter".to_string(),
        }
    }

    #[must_use]
    pub fn with_webhook_endpoint(mut self, webhook_endpoint: impl Into<String>) -> Self {
        self.webhook_endpoint = Some(webhook_endpoint.into());
        self
    }

    #[must_use]
    pub fn with_webhook_delivery_policy(mut self, policy: WebhookDeliveryPolicy) -> Self {
        self.webhook_delivery_policy = policy;
        self
    }

    #[must_use]
    pub fn with_nasa_api_key(mut self, nasa_api_key: impl Into<String>) -> Self {
        self.nasa_api_key = nasa_api_key.into();
        self
    }

    #[must_use]
    pub fn with_nasa_api_base_url(mut self, nasa_api_base_url: impl Into<String>) -> Self {
        self.nasa_api_base_url = nasa_api_base_url.into();
        self
    }

    #[must_use]
    pub fn with_open_meteo_base_url(mut self, open_meteo_base_url: impl Into<String>) -> Self {
        self.open_meteo_base_url = open_meteo_base_url.into();
        self
    }

    #[must_use]
    pub fn with_firms_api_base_url(mut self, firms_api_base_url: impl Into<String>) -> Self {
        self.firms_api_base_url = firms_api_base_url.into();
        self
    }

    #[must_use]
    pub fn with_firms_map_key(mut self, firms_map_key: impl Into<String>) -> Self {
        self.firms_map_key = Some(firms_map_key.into());
        self
    }

    #[must_use]
    pub fn with_firms_source(mut self, firms_source: impl Into<String>) -> Self {
        self.firms_source = firms_source.into();
        self
    }

    #[must_use]
    pub fn with_opensky_api_base_url(mut self, opensky_api_base_url: impl Into<String>) -> Self {
        self.opensky_api_base_url = opensky_api_base_url.into();
        self
    }

    #[must_use]
    pub fn with_opensky_bearer_token(mut self, opensky_bearer_token: impl Into<String>) -> Self {
        self.opensky_bearer_token = Some(opensky_bearer_token.into());
        self
    }

    #[must_use]
    pub fn with_overpass_api_base_url(mut self, overpass_api_base_url: impl Into<String>) -> Self {
        self.overpass_api_base_url = overpass_api_base_url.into();
        self
    }

    pub fn demo() -> Result<Self, StorageError> {
        Self::demo_with_storage(SqliteStore::in_memory()?)
    }

    pub fn demo_with_storage(storage: SqliteStore) -> Result<Self, StorageError> {
        let object = demo_object();
        let mut layer = sss_core::IntelligenceLayer::new(
            DigitalTwin::new(vec![object.clone()]),
            AnticipationEngine::default(),
        );
        layer.record_observation(demo_observation(&object));
        Ok(Self::new(layer, storage))
    }

    pub fn from_storage(storage: SqliteStore) -> Result<Self, StorageError> {
        let objects = storage.space_objects()?;
        let observations = storage.latest_observations()?;
        let mut layer = sss_core::IntelligenceLayer::new(
            DigitalTwin::new(objects),
            AnticipationEngine::default(),
        );
        for observation in observations {
            layer.record_observation(observation);
        }
        Ok(Self::new(layer, storage))
    }

    pub fn analyze(
        &self,
        request_id: &str,
        request: sss_core::AnalyzeObjectRequest,
    ) -> Result<sss_core::AnalyzeObjectResponse, StorageError> {
        let request_json = serde_json::to_string(&request)?;
        let object_id = request.object_id.clone();
        let response = self
            .intelligence
            .read()
            .expect("intelligence lock should not be poisoned")
            .analyze_object(request);
        self.store_evidence_bundle(&response.evidence_bundle)?;
        self.storage.store_assessment(
            &response.object_id,
            response.finding.observed_at_unix_seconds,
            &response.assessment,
        )?;
        self.storage
            .store_prediction_snapshot(&prediction_snapshot_from_analysis(request_id, &response))?;
        self.storage.log_exchange(&RequestResponseLog {
            request_id: request_id.to_string(),
            endpoint: "/v1/analyze-object".to_string(),
            object_id: Some(object_id),
            timestamp_unix_seconds: now_unix_seconds(),
            request_json,
            response_json: serde_json::to_string(&response)?,
        })?;
        Ok(response)
    }

    #[allow(clippy::too_many_lines)]
    pub async fn dispatch_notifications_for_analysis(
        &self,
        request_id: &str,
        request: AnalyzeObjectRequest,
    ) -> Result<NotificationDispatchResponse, AppError> {
        let Some(webhook_endpoint) = self.webhook_endpoint.as_ref() else {
            return Err(AppError::NotificationUnavailable(
                "webhook delivery is not configured".to_string(),
            ));
        };

        let analysis = self.analyze(request_id, request)?;
        let mut deliveries = Vec::with_capacity(analysis.decision.notifications.len());
        let max_attempts = self.webhook_delivery_policy.max_attempts.max(1);

        for (index, notification) in analysis.decision.notifications.iter().enumerate() {
            let payload = WebhookNotificationPayload {
                request_id: request_id.to_string(),
                object_id: analysis.object_id.clone(),
                severity: analysis.decision.severity.clone(),
                risk_score: analysis.decision.risk_score,
                recommendation: analysis.decision.recommendation.clone(),
                recipient_policy_id: analysis.decision.recipient_policy.policy_id.clone(),
                notification: notification.clone(),
                evidence_bundle_hash: analysis.evidence_bundle.bundle_hash.clone(),
                prediction: analysis.prediction.clone(),
            };
            let payload_json = serde_json::to_string(&payload)?;
            for attempt_number in 1..=max_attempts {
                let timestamp_unix_seconds = now_unix_seconds();
                let outcome = self
                    .http_client
                    .post(webhook_endpoint)
                    .header("content-type", "application/json")
                    .body(payload_json.clone())
                    .send()
                    .await;
                let (status, status_code, response_body, should_retry) = match outcome {
                    Ok(response) => {
                        let status_code = response.status().as_u16();
                        let response_body = response
                            .text()
                            .await
                            .map_err(|error| AppError::SourceUnavailable(error.to_string()))?;
                        if (200..300).contains(&status_code) {
                            (
                                "delivered".to_string(),
                                Some(status_code),
                                Some(response_body),
                                false,
                            )
                        } else if attempt_number < max_attempts {
                            (
                                "retrying".to_string(),
                                Some(status_code),
                                Some(response_body),
                                true,
                            )
                        } else {
                            (
                                "failed".to_string(),
                                Some(status_code),
                                Some(response_body),
                                false,
                            )
                        }
                    }
                    Err(error) => {
                        let response_body = Some(error.to_string());
                        if attempt_number < max_attempts {
                            ("retrying".to_string(), None, response_body, true)
                        } else {
                            ("failed".to_string(), None, response_body, false)
                        }
                    }
                };
                let delivery = NotificationDeliveryLog {
                    delivery_id: format!(
                        "{request_id}:{}:{index}:{attempt_number}",
                        analysis.object_id
                    ),
                    request_id: request_id.to_string(),
                    object_id: analysis.object_id.clone(),
                    recipient: notification.recipient.clone(),
                    channel: "webhook".to_string(),
                    target: webhook_endpoint.clone(),
                    status,
                    status_code,
                    timestamp_unix_seconds,
                    attempt_number,
                    max_attempts,
                    notification_reason: notification.notification_reason.clone(),
                    payload_json: payload_json.clone(),
                    response_body,
                };
                self.storage.log_notification_delivery(&delivery)?;
                let delivered = delivery.status == "delivered";
                deliveries.push(delivery);
                if delivered {
                    break;
                }
                if should_retry {
                    tokio::time::sleep(self.webhook_delivery_policy.retry_delay).await;
                }
            }
        }

        Ok(NotificationDispatchResponse {
            analysis,
            deliveries,
        })
    }

    pub async fn dispatch_notifications_for_event(
        &self,
        request_id: &str,
        event_id: &str,
    ) -> Result<EventDispatchResponse, AppError> {
        let Some(event) = self.storage.ranked_event(event_id)? else {
            return Err(AppError::EventNotFound(format!(
                "ranked event not found: {event_id}"
            )));
        };
        let dispatch = self
            .dispatch_notifications_for_analysis(
                request_id,
                AnalyzeObjectRequest {
                    object_id: event.object_id.clone(),
                    timestamp_unix_seconds: event.target_epoch_unix_seconds,
                },
            )
            .await?;
        self.storage.log_exchange(&RequestResponseLog {
            request_id: request_id.to_string(),
            endpoint: "/v1/events/dispatch".to_string(),
            object_id: Some(event.object_id.clone()),
            timestamp_unix_seconds: now_unix_seconds(),
            request_json: serde_json::json!({ "event_id": event_id }).to_string(),
            response_json: serde_json::to_string(&event)?,
        })?;
        Ok(EventDispatchResponse { event, dispatch })
    }

    pub fn ingest_tle(
        &self,
        request_id: &str,
        source: &str,
        payload: &str,
    ) -> Result<IngestOutcome, AppError> {
        let now = now_unix_seconds();
        if let Some(previous_batch) = self.storage.latest_ingest_batch_for_source(source)? {
            if previous_batch.raw_payload == payload {
                return Ok(IngestOutcome {
                    records_received: previous_batch.records_received,
                    observations_created: previous_batch.observations_created,
                    object_ids: previous_batch.object_ids,
                    skipped_duplicate: true,
                    freshness_seconds: now.saturating_sub(previous_batch.timestamp_unix_seconds),
                });
            }
        }

        let records = parse_tle_records(payload)?;
        let mut object_ids = Vec::with_capacity(records.len());
        let mut observations_created = 0_usize;

        {
            let mut intelligence = self
                .intelligence
                .write()
                .expect("intelligence lock should not be poisoned");
            for record in &records {
                let normalized = normalize_tle_record(record)?;
                object_ids.push(normalized.object.id.clone());
                self.storage.store_space_object(&normalized.object)?;
                self.storage.store_observation(&normalized.observation)?;
                intelligence.upsert_object(normalized.object);
                intelligence.record_observation(normalized.observation);
                observations_created += 1;
            }
        }

        self.storage.log_ingest_batch(&IngestBatchLog {
            request_id: request_id.to_string(),
            source: source.to_string(),
            timestamp_unix_seconds: now,
            records_received: records.len(),
            observations_created,
            object_ids: object_ids.clone(),
            raw_payload: payload.to_string(),
        })?;

        Ok(IngestOutcome {
            records_received: records.len(),
            observations_created,
            object_ids,
            skipped_duplicate: false,
            freshness_seconds: 0,
        })
    }

    pub async fn ingest_live_tle_source(
        &self,
        request_id: &str,
        source: &str,
        url: &str,
    ) -> Result<(String, IngestOutcome), AppError> {
        let response = self
            .http_client
            .get(url)
            .send()
            .await
            .map_err(|error| AppError::SourceUnavailable(error.to_string()))?;
        let payload = response
            .error_for_status()
            .map_err(|error| AppError::SourceUnavailable(error.to_string()))?
            .text()
            .await
            .map_err(|error| AppError::SourceUnavailable(error.to_string()))?;
        let outcome = self.ingest_tle(request_id, source, &payload)?;
        Ok((payload, outcome))
    }

    pub fn overview(
        &self,
        request_id: &str,
        request: sss_core::SpaceOverviewRequest,
    ) -> Result<sss_core::SpaceOverviewResponse, StorageError> {
        let request_json = serde_json::to_string(&request)?;
        let response = self
            .intelligence
            .read()
            .expect("intelligence lock should not be poisoned")
            .space_overview(request);
        self.storage.store_ranked_events(&response.top_events)?;
        self.storage.log_exchange(&RequestResponseLog {
            request_id: request_id.to_string(),
            endpoint: "/v1/space-overview".to_string(),
            object_id: None,
            timestamp_unix_seconds: now_unix_seconds(),
            request_json,
            response_json: serde_json::to_string(&response)?,
        })?;
        Ok(response)
    }

    pub fn predict(
        &self,
        request_id: &str,
        request: sss_core::PredictRequest,
    ) -> Result<sss_core::PredictResponse, StorageError> {
        let request_json = serde_json::to_string(&request)?;
        let object_id = request.object_id.clone();
        let response = self
            .intelligence
            .read()
            .expect("intelligence lock should not be poisoned")
            .predict(request);
        self.storage
            .store_prediction_snapshot(&prediction_snapshot_from_predict_response(
                request_id, &response,
            ))?;
        self.storage.log_exchange(&RequestResponseLog {
            request_id: request_id.to_string(),
            endpoint: "/v1/predict".to_string(),
            object_id: Some(object_id),
            timestamp_unix_seconds: now_unix_seconds(),
            request_json,
            response_json: serde_json::to_string(&response)?,
        })?;
        Ok(response)
    }

    pub fn prediction_snapshots(
        &self,
        object_id: &str,
        limit: usize,
    ) -> Result<Vec<PredictionSnapshot>, StorageError> {
        self.storage.prediction_snapshots(object_id, limit)
    }

    pub async fn passive_site_solar_forecast(
        &self,
        request_id: &str,
        site_id: &str,
        horizon_hours: u32,
    ) -> Result<SolarForecastResponse, AppError> {
        let context = self
            .passive_site_forecast_context(site_id, horizon_hours)
            .await?;
        let response = solar_forecast(&context);
        self.store_forecast_snapshot(
            request_id,
            "/v1/forecast/sites/:site_id/solar",
            site_id,
            horizon_hours,
            &response.narrative,
            response.confidence,
        )?;
        Ok(response)
    }

    pub async fn passive_site_load_forecast(
        &self,
        request_id: &str,
        site_id: &str,
        horizon_hours: u32,
    ) -> Result<LoadForecastResponse, AppError> {
        let context = self
            .passive_site_forecast_context(site_id, horizon_hours)
            .await?;
        let response = load_forecast(&context);
        self.store_forecast_snapshot(
            request_id,
            "/v1/forecast/sites/:site_id/load",
            site_id,
            horizon_hours,
            &response.narrative,
            response.confidence,
        )?;
        Ok(response)
    }

    pub async fn passive_site_scenario_forecast(
        &self,
        request_id: &str,
        site_id: &str,
        horizon_hours: u32,
    ) -> Result<ScenarioForecastResponse, AppError> {
        let context = self
            .passive_site_forecast_context(site_id, horizon_hours)
            .await?;
        let response = scenario_forecast(&context);
        self.store_forecast_snapshot(
            request_id,
            "/v1/forecast/sites/:site_id/scenario",
            site_id,
            horizon_hours,
            &response.summary,
            response.confidence,
        )?;
        Ok(response)
    }

    pub async fn passive_site_recommendation(
        &self,
        request_id: &str,
        site_id: &str,
        request: &OrchestratorRecommendationRequest,
    ) -> Result<OrchestratorRecommendation, AppError> {
        let horizon_hours = request.horizon_hours.unwrap_or(24).clamp(1, 72);
        let context = self
            .passive_site_forecast_context(site_id, horizon_hours)
            .await?;
        let response = recommend_action(&context, request);
        self.store_forecast_snapshot(
            request_id,
            "/v1/orchestrator/sites/:site_id/recommend",
            site_id,
            horizon_hours,
            &response.operational_reason,
            response.confidence,
        )?;
        Ok(response)
    }

    pub fn passive_site_orchestrator_decisions(
        &self,
        site_id: &str,
        limit: usize,
    ) -> Result<Vec<PredictionSnapshot>, StorageError> {
        Ok(self
            .storage
            .prediction_snapshots(site_id, limit)?
            .into_iter()
            .filter(|snapshot| snapshot.endpoint == "/v1/orchestrator/sites/:site_id/recommend")
            .collect())
    }

    pub fn passive_site_recommendation_reviews(
        &self,
        site_id: &str,
        limit: usize,
    ) -> Result<Vec<PassiveRecommendationReview>, StorageError> {
        self.storage.passive_recommendation_reviews(site_id, limit)
    }

    pub fn set_passive_site_recommendation_review(
        &self,
        site_id: &str,
        state: PassiveRecommendationReviewState,
        actor: Option<String>,
        rationale: Option<String>,
        snapshot_id: Option<&String>,
    ) -> Result<PassiveRecommendationReview, AppError> {
        self.storage
            .passive_site_profiles()?
            .into_iter()
            .find(|site| site.site.site_id.to_string() == site_id)
            .ok_or_else(|| AppError::SiteNotFound(format!("passive site not found: {site_id}")))?;
        let decisions = self.passive_site_orchestrator_decisions(site_id, 50)?;
        let snapshot = if let Some(snapshot_id) = snapshot_id {
            decisions
                .into_iter()
                .find(|candidate| candidate.snapshot_id == *snapshot_id)
        } else {
            decisions.into_iter().next()
        }
        .ok_or_else(|| {
            AppError::InvalidRequest(format!(
                "no orchestrator recommendation snapshot found for site: {site_id}"
            ))
        })?;
        let review = PassiveRecommendationReview {
            review_id: format!(
                "{}:review:{}:{}",
                snapshot.snapshot_id,
                format!("{state:?}").to_lowercase(),
                now_unix_seconds()
            ),
            site_id: site_id.to_string(),
            snapshot_id: snapshot.snapshot_id.clone(),
            request_id: snapshot.request_id.clone(),
            endpoint: snapshot.endpoint.clone(),
            state,
            actor,
            rationale,
            evidence_bundle_hash: snapshot.evidence_bundle_hash.clone(),
            decided_at_unix_seconds: now_unix_seconds(),
        };
        self.storage
            .store_passive_recommendation_review(&review)
            .map_err(AppError::Storage)?;
        Ok(review)
    }

    fn store_forecast_snapshot(
        &self,
        request_id: &str,
        endpoint: &str,
        site_id: &str,
        horizon_hours: u32,
        explanation: &str,
        confidence: f64,
    ) -> Result<(), AppError> {
        self.storage
            .store_prediction_snapshot(&forecast_prediction_snapshot(
                request_id,
                endpoint,
                site_id,
                horizon_hours,
                explanation,
                confidence,
            ))
            .map_err(AppError::Storage)
    }

    pub fn timeline(
        &self,
        request_id: &str,
        request: sss_core::TimelineRequest,
    ) -> Result<Option<sss_core::ObjectTimelineResponse>, StorageError> {
        let request_json = serde_json::to_string(&request)?;
        let object_id = request.object_id.clone();
        let response = self
            .intelligence
            .read()
            .expect("intelligence lock should not be poisoned")
            .timeline(request);
        if let Some(payload) = response.as_ref() {
            self.storage.log_exchange(&RequestResponseLog {
                request_id: request_id.to_string(),
                endpoint: "/v1/objects/{id}/timeline".to_string(),
                object_id: Some(object_id),
                timestamp_unix_seconds: now_unix_seconds(),
                request_json,
                response_json: serde_json::to_string(payload)?,
            })?;
        }
        Ok(response)
    }

    pub fn close_approaches(
        &self,
        request_id: &str,
        object_id: &str,
        horizon_hours: f64,
        threshold_km: f64,
        limit: usize,
    ) -> Result<Option<CloseApproachBriefing>, StorageError> {
        let request_json = serde_json::json!({
            "object_id": object_id,
            "horizon_hours": horizon_hours,
            "threshold_km": threshold_km,
            "limit": limit,
        })
        .to_string();
        let intelligence = self
            .intelligence
            .read()
            .expect("intelligence lock should not be poisoned");
        let Some(primary_object) = intelligence.object(object_id) else {
            return Ok(None);
        };
        let base_epoch_unix_seconds = intelligence
            .timeline(sss_core::TimelineRequest {
                object_id: object_id.to_string(),
                horizon_hours,
            })
            .map_or(primary_object.state.epoch_unix_seconds, |timeline| {
                timeline.base_epoch_unix_seconds
            });
        let epochs = close_approach_epochs(base_epoch_unix_seconds, horizon_hours);
        let mut approaches = intelligence
            .objects()
            .into_iter()
            .filter(|object| object.id != object_id)
            .filter_map(|counterpart| {
                let best = epochs
                    .iter()
                    .filter_map(|epoch| {
                        let primary_state = intelligence.predicted_state(object_id, *epoch)?;
                        let counterpart_state =
                            intelligence.predicted_state(&counterpart.id, *epoch)?;
                        let miss_distance_km = primary_state
                            .position_km
                            .distance_to(counterpart_state.position_km);
                        Some((*epoch, primary_state, counterpart_state, miss_distance_km))
                    })
                    .min_by(|left, right| left.3.total_cmp(&right.3))?;
                if best.3 > threshold_km {
                    return None;
                }
                let hours_until = seconds_to_hours(best.0 - base_epoch_unix_seconds);
                let priority_score = close_approach_priority_score(
                    best.3,
                    hours_until,
                    primary_object
                        .risk
                        .criticality
                        .max(counterpart.risk.criticality),
                    threshold_km,
                );
                Some(CloseApproachCandidate {
                    counterpart_object_id: counterpart.id.clone(),
                    counterpart_object_name: counterpart.name.clone(),
                    target_epoch_unix_seconds: best.0,
                    miss_distance_km: best.3,
                    priority_score,
                    propagation_model: propagation_model_for_close_approach(
                        &primary_object,
                        &counterpart,
                    )
                    .to_string(),
                    summary: format!(
                        "{} predicted within {:.1} km of {} in {:.1} h",
                        counterpart.name,
                        best.3,
                        primary_object.name,
                        hours_until.max(0.0)
                    ),
                    primary_state: best.1,
                    counterpart_state: best.2,
                })
            })
            .collect::<Vec<_>>();
        approaches.sort_by(compare_close_approaches);
        approaches.truncate(limit);
        let response = CloseApproachBriefing {
            generated_at_unix_seconds: now_unix_seconds(),
            object_id: primary_object.id.clone(),
            object_name: primary_object.name,
            base_epoch_unix_seconds,
            horizon_hours,
            threshold_km,
            scanned_objects: intelligence.objects().len().saturating_sub(1),
            close_approaches: approaches,
        };
        self.storage.log_exchange(&RequestResponseLog {
            request_id: request_id.to_string(),
            endpoint: "/v1/objects/{id}/close-approaches".to_string(),
            object_id: Some(object_id.to_string()),
            timestamp_unix_seconds: now_unix_seconds(),
            request_json,
            response_json: serde_json::to_string(&response)?,
        })?;
        Ok(Some(response))
    }

    pub fn evidence_bundle(
        &self,
        bundle_hash: &str,
    ) -> Result<Option<EvidenceBundle>, StorageError> {
        self.storage.evidence_bundle(bundle_hash)
    }

    pub fn replay_manifest(
        &self,
        manifest_hash: &str,
    ) -> Result<Option<ReplayManifest>, StorageError> {
        self.storage.replay_manifest(manifest_hash)
    }

    pub fn replay_manifest_for_bundle(
        &self,
        bundle_hash: &str,
    ) -> Result<Option<ReplayManifest>, StorageError> {
        Ok(self
            .storage
            .evidence_bundle(bundle_hash)?
            .map(|bundle| replay_manifest_for(&bundle)))
    }

    pub fn execute_replay(
        &self,
        manifest_hash: &str,
    ) -> Result<Option<sss_core::ReplayExecution>, StorageError> {
        let Some(manifest) = self.storage.replay_manifest(manifest_hash)? else {
            return Ok(None);
        };
        let Some(evidence_bundle) = self.storage.evidence_bundle_for_manifest(manifest_hash)?
        else {
            return Ok(None);
        };
        let original_assessment = self
            .storage
            .assessment_history(&manifest.object_id)?
            .into_iter()
            .last();
        let original_analysis = self
            .storage
            .request_history(&manifest.object_id)?
            .into_iter()
            .rev()
            .find(|entry| entry.endpoint == "/v1/analyze-object")
            .and_then(|entry| {
                serde_json::from_str::<AnalyzeObjectResponse>(&entry.response_json).ok()
            });
        let object = self
            .storage
            .space_objects()?
            .into_iter()
            .find(|object| object.id == manifest.object_id)
            .or_else(|| {
                self.intelligence
                    .read()
                    .expect("intelligence lock should not be poisoned")
                    .object(&manifest.object_id)
            });
        let Some(object) = object else {
            return Ok(None);
        };
        let replayed_analysis =
            sss_core::replay_response_from_evidence_bundle(&evidence_bundle, &object);
        let replayed_assessment = replayed_analysis.assessment.clone();
        let replayed_decision = replayed_analysis.decision.clone();
        let original_decision = original_analysis
            .as_ref()
            .map(|analysis| analysis.decision.clone());
        let diff = sss_core::replay_diff(
            original_assessment.as_ref(),
            &replayed_assessment,
            original_decision.as_ref(),
            &replayed_decision,
        );
        let drift_detected = !diff.assessment.changed_fields.is_empty()
            || diff.assessment.confidence_delta.abs() > f64::EPSILON
            || diff.assessment.prediction_probability_delta.abs() > f64::EPSILON
            || !diff.assessment.added_rule_hits.is_empty()
            || !diff.assessment.removed_rule_hits.is_empty()
            || !diff.assessment.added_signals.is_empty()
            || !diff.assessment.removed_signals.is_empty()
            || !diff.decision.changed_fields.is_empty()
            || diff
                .decision
                .risk_score_delta
                .is_some_and(|delta| delta.abs() > f64::EPSILON)
            || !diff.decision.added_recipients.is_empty()
            || !diff.decision.removed_recipients.is_empty()
            || !diff.decision.added_escalation_rules.is_empty()
            || !diff.decision.removed_escalation_rules.is_empty();

        Ok(Some(sss_core::ReplayExecution {
            manifest,
            evidence_bundle,
            original_assessment,
            replayed_assessment,
            original_decision,
            replayed_decision,
            diff,
            drift_detected,
        }))
    }

    pub fn ranked_events(
        &self,
        limit: usize,
        filter: &RankedEventsFilter,
    ) -> Result<Vec<sss_core::RankedEvent>, StorageError> {
        let fetch_limit = limit.saturating_mul(5).clamp(limit, 500);
        let now = now_unix_seconds();
        let mut events = self.storage.top_ranked_events(fetch_limit)?;
        events.retain(|event| ranked_event_matches_filter(event, filter, now));
        events.truncate(limit);
        Ok(events)
    }

    pub fn ranked_events_timeline(
        &self,
        horizon_hours: f64,
        limit_per_bucket: usize,
        reference_unix_seconds: i64,
        filter: &RankedEventsFilter,
    ) -> Result<EventsTimelineResponse, StorageError> {
        let horizon_unix_seconds =
            reference_unix_seconds + rounded_seconds_from_hours(horizon_hours);
        let fetch_limit = limit_per_bucket
            .saturating_mul(20)
            .clamp(limit_per_bucket, 500);
        let mut events = self.storage.top_ranked_events(fetch_limit)?;
        let future_filter = RankedEventsFilter {
            object_id: filter.object_id.clone(),
            event_type: filter.event_type.clone(),
            future_only: true,
            after_unix_seconds: Some(reference_unix_seconds),
            before_unix_seconds: Some(horizon_unix_seconds),
        };
        events.retain(|event| {
            ranked_event_matches_filter(event, &future_filter, reference_unix_seconds)
        });

        let mut buckets = timeline_bucket_specs(horizon_hours)
            .into_iter()
            .map(|(label, start_offset_hours, end_offset_hours)| {
                let start_unix_seconds =
                    reference_unix_seconds + rounded_seconds_from_hours(start_offset_hours);
                let end_unix_seconds =
                    reference_unix_seconds + rounded_seconds_from_hours(end_offset_hours);
                let mut bucket_events = events
                    .iter()
                    .filter(|event| {
                        event.target_epoch_unix_seconds.is_some_and(|target_epoch| {
                            target_epoch >= start_unix_seconds && target_epoch <= end_unix_seconds
                        })
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                bucket_events.sort_by(compare_timeline_events);
                bucket_events.truncate(limit_per_bucket);
                EventsTimelineBucket {
                    label: label.to_string(),
                    start_offset_hours,
                    end_offset_hours,
                    events: bucket_events,
                }
            })
            .collect::<Vec<_>>();

        buckets.retain(|bucket| !bucket.events.is_empty());
        let total_events = buckets.iter().map(|bucket| bucket.events.len()).sum();

        Ok(EventsTimelineResponse {
            generated_at_unix_seconds: reference_unix_seconds,
            horizon_hours,
            total_events,
            buckets,
        })
    }

    pub fn ingest_batches(&self, limit: usize) -> Result<Vec<IngestBatchLog>, StorageError> {
        self.storage.latest_ingest_batches(limit)
    }

    pub fn notification_deliveries(
        &self,
        limit: usize,
        object_id: Option<&str>,
    ) -> Result<Vec<NotificationDeliveryLog>, StorageError> {
        self.storage.notification_deliveries(limit, object_id)
    }

    pub fn passive_scan(
        &self,
        request_id: &str,
        request: &PassiveScanInput,
    ) -> Result<PassiveScanOutput, StorageError> {
        let request_json = serde_json::to_string(&request)?;
        let response = self.passive_scanner.scan(request);
        self.store_or_update_seed_lifecycle(
            &request.infra_sites,
            Some(&response),
            now_unix_seconds(),
        )?;
        self.store_passive_scan_response(request_id, "/v1/passive/scan", &request_json, &response)?;
        Ok(response)
    }

    pub async fn passive_scan_live(
        &self,
        request_id: &str,
        request: &PassiveLiveScanRequest,
    ) -> Result<PassiveLiveScanResponse, AppError> {
        let request_json = serde_json::to_string(request)?;
        let scan_live_response = self.build_passive_live_scan_response(request).await?;
        let scan = scan_live_response.scan.clone();
        self.store_or_update_seed_lifecycle(&request.infra_sites, Some(&scan), now_unix_seconds())
            .map_err(AppError::Storage)?;
        self.store_passive_source_health_samples(request_id, None, &scan_live_response)
            .map_err(AppError::Storage)?;
        self.store_passive_scan_response(request_id, "/v1/passive/scan/live", &request_json, &scan)
            .map_err(AppError::Storage)?;
        Ok(scan_live_response)
    }

    pub fn passive_sites(&self) -> Result<Vec<PassiveSiteProfile>, StorageError> {
        self.storage.passive_site_profiles()
    }

    pub async fn discover_passive_infra_sites(
        &self,
        request_id: &str,
        request: &PassiveInfraDiscoverRequest,
    ) -> Result<PassiveInfraDiscoverResponse, AppError> {
        if request.south >= request.north || request.west >= request.east {
            return Err(AppError::InvalidRequest(
                "invalid bbox: south/west must be lower than north/east".to_string(),
            ));
        }
        let request_json = serde_json::to_string(request)?;
        let query = build_overpass_query(request);
        let payload: OverpassResponse = self
            .http_client
            .post(&self.overpass_api_base_url)
            .form(&[("data", query)])
            .send()
            .await
            .map_err(|error| AppError::SourceUnavailable(error.to_string()))?
            .error_for_status()
            .map_err(|error| AppError::SourceUnavailable(error.to_string()))?
            .json::<OverpassResponse>()
            .await
            .map_err(|error| AppError::SourceUnavailable(error.to_string()))?;
        let seeds = overpass_elements_to_seeds(request, payload.elements);
        self.store_discovered_site_profiles(&seeds)
            .map_err(AppError::Storage)?;
        self.store_or_update_seed_lifecycle(&seeds, None, now_unix_seconds())
            .map_err(AppError::Storage)?;
        let response = PassiveInfraDiscoverResponse {
            generated_at_unix_seconds: now_unix_seconds(),
            source: self.overpass_api_base_url.clone(),
            south: request.south,
            west: request.west,
            north: request.north,
            east: request.east,
            seeds,
        };
        self.storage.log_exchange(&RequestResponseLog {
            request_id: request_id.to_string(),
            endpoint: "/v1/passive/discover-sites".to_string(),
            object_id: None,
            timestamp_unix_seconds: now_unix_seconds(),
            request_json,
            response_json: serde_json::to_string(&response)?,
        })?;
        Ok(response)
    }

    pub async fn discover_and_scan_passive_sites(
        &self,
        request_id: &str,
        request: &PassiveDiscoverAndScanRequest,
    ) -> Result<PassiveDiscoverAndScanResponse, AppError> {
        let discovery_request_id = format!("{request_id}:discover");
        let live_scan_request_id = format!("{request_id}:scan");
        let discovery = self
            .discover_passive_infra_sites(&discovery_request_id, &request.discovery)
            .await?;
        let live_scan = self
            .passive_scan_live(
                &live_scan_request_id,
                &PassiveLiveScanRequest {
                    window_hours: request.window_hours,
                    infra_sites: discovery.seeds.clone(),
                    include_adsb: request.include_adsb,
                    include_weather: request.include_weather,
                    include_fire_smoke: request.include_fire_smoke,
                },
            )
            .await?;
        let mut site_overviews = Vec::new();
        for site in &live_scan.scan.sites {
            site_overviews.push(self.passive_site_overview(
                &site.site.site_id.to_string(),
                10,
                30,
            )?);
        }
        let execution_summary = build_execution_summary(&site_overviews, &live_scan.scan);

        Ok(PassiveDiscoverAndScanResponse {
            generated_at_unix_seconds: now_unix_seconds(),
            discovery,
            live_scan,
            execution_summary,
            site_overviews,
        })
    }

    pub fn passive_seed_records(
        &self,
        limit: usize,
    ) -> Result<Vec<PassiveSeedRecord>, StorageError> {
        self.storage.passive_seed_records(limit)
    }

    pub fn passive_seed_record(
        &self,
        seed_key: &str,
    ) -> Result<Option<PassiveSeedRecord>, StorageError> {
        self.storage.passive_seed_record(seed_key)
    }

    pub async fn run_passive_scheduler(
        &self,
        request_id: &str,
        request: &PassiveScheduleScanRequest,
    ) -> Result<PassiveScheduleScanResponse, AppError> {
        let request_json = serde_json::to_string(request)?;
        let now = now_unix_seconds();
        let limit = request.limit.unwrap_or(25).clamp(1, 200);
        let minimum_priority = request.minimum_priority.unwrap_or(0.0).clamp(0.0, 1.0);
        let force = request.force.unwrap_or(false);
        let dry_run = request.dry_run.unwrap_or(false);
        let all_records = self.storage.passive_seed_records(10_000)?;
        let mut seen_seed_keys = std::collections::BTreeSet::new();
        let mut planned_seeds = Vec::new();
        let mut selected_records = Vec::new();
        let mut skipped_below_priority_count = 0;
        let mut skipped_not_due_count = 0;

        for record in all_records {
            if !seen_seed_keys.insert(record.seed_key.clone()) {
                continue;
            }
            if record.scan_priority < minimum_priority {
                skipped_below_priority_count += 1;
                continue;
            }
            let cadence_seconds = passive_scan_cadence_seconds(&record);
            let overdue_seconds = passive_scan_overdue_seconds(&record, now, cadence_seconds);
            if !force && overdue_seconds < 0 {
                skipped_not_due_count += 1;
                continue;
            }
            planned_seeds.push(PassiveScheduledSeed {
                seed_key: record.seed_key.clone(),
                site_name: record.seed.name.clone(),
                site_type: record.seed.site_type,
                scan_priority: record.scan_priority,
                cadence_seconds,
                overdue_seconds: overdue_seconds.max(0),
                reason: passive_schedule_reason(&record, force, overdue_seconds),
            });
            selected_records.push(record);
            if selected_records.len() >= limit {
                break;
            }
        }

        let evaluated_seed_count = seen_seed_keys.len();
        let selected_seed_count = selected_records.len();
        let skipped_seed_count = evaluated_seed_count.saturating_sub(selected_seed_count);
        let mut response = PassiveScheduleScanResponse {
            generated_at_unix_seconds: now_unix_seconds(),
            evaluated_seed_count,
            selected_seed_count,
            skipped_seed_count,
            skipped_not_due_count,
            skipped_below_priority_count,
            dry_run,
            planned_seeds,
            live_scan: None,
            execution_summary: None,
            site_overviews: Vec::new(),
        };

        if !dry_run && !selected_records.is_empty() {
            let live_scan = self
                .passive_scan_live(
                    &format!("{request_id}:scheduled-scan"),
                    &PassiveLiveScanRequest {
                        window_hours: request.window_hours,
                        infra_sites: selected_records
                            .iter()
                            .map(|record| record.seed.clone())
                            .collect(),
                        include_adsb: request.include_adsb,
                        include_weather: request.include_weather,
                        include_fire_smoke: request.include_fire_smoke,
                    },
                )
                .await?;
            let mut site_overviews = Vec::new();
            for site in &live_scan.scan.sites {
                site_overviews.push(self.passive_site_overview(
                    &site.site.site_id.to_string(),
                    10,
                    30,
                )?);
            }
            response.execution_summary =
                Some(build_execution_summary(&site_overviews, &live_scan.scan));
            response.site_overviews = site_overviews;
            response.live_scan = Some(live_scan);
        }

        self.storage.log_exchange(&RequestResponseLog {
            request_id: request_id.to_string(),
            endpoint: "/v1/passive/scheduler/run".to_string(),
            object_id: None,
            timestamp_unix_seconds: now_unix_seconds(),
            request_json,
            response_json: serde_json::to_string(&response)?,
        })?;
        Ok(response)
    }

    /// Create or update a passive region target from the provided request.
    ///
    /// Validates the request (name must be non-empty and bbox must have south < north and west < east),
    /// fills default and clamped values for cadence, scan limit, minimum priority, and enabled flag,
    /// preserves creation timestamp when updating an existing region, and persists the target to storage.
    ///
    /// # Errors
    ///
    /// Returns `AppError::InvalidRequest` when the name is empty or the bounding box is invalid.
    ///
    /// # Returns
    ///
    /// A `PassiveRegionTarget` representing the inserted or updated region target.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let state = AppState::demo();
    /// let req = PassiveRegionTargetRequest {
    ///     name: "Test Region".to_string(),
    ///     south: 10.0,
    ///     west: 20.0,
    ///     north: 11.0,
    ///     east: 21.0,
    ///     ..Default::default()
    /// };
    /// let target = state.upsert_passive_region_target(&req).unwrap();
    /// assert_eq!(target.name, "Test Region");
    /// ```
    pub fn upsert_passive_region_target(
        &self,
        request: &PassiveRegionTargetRequest,
    ) -> Result<PassiveRegionTarget, AppError> {
        if request.name.trim().is_empty() {
            return Err(AppError::InvalidRequest(
                "region name must not be empty".to_string(),
            ));
        }
        if request.south >= request.north || request.west >= request.east {
            return Err(AppError::InvalidRequest(
                "invalid region bbox: south/west must be lower than north/east".to_string(),
            ));
        }

        let now = now_unix_seconds();
        let region_id = request
            .region_id
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .cloned()
            .unwrap_or_else(|| passive_region_id(&request.name, request.south, request.west));
        let existing = self.storage.passive_region_target(&region_id)?;
        let target = PassiveRegionTarget {
            region_id,
            name: request.name.trim().to_string(),
            south: request.south,
            west: request.west,
            north: request.north,
            east: request.east,
            site_types: request.site_types.clone(),
            timezone: request.timezone.clone(),
            country_code: request.country_code.clone(),
            default_operator_name: request.default_operator_name.clone(),
            default_criticality: request.default_criticality,
            observation_radius_km: request.observation_radius_km,
            discovery_cadence_seconds: request
                .discovery_cadence_seconds
                .unwrap_or(86_400)
                .clamp(3_600, 2_592_000),
            scan_limit: request.scan_limit.unwrap_or(25).clamp(1, 200),
            minimum_priority: request.minimum_priority.unwrap_or(0.0).clamp(0.0, 1.0),
            enabled: request.enabled.unwrap_or(true),
            created_at_unix_seconds: existing
                .as_ref()
                .map_or(now, |target| target.created_at_unix_seconds),
            updated_at_unix_seconds: now,
            last_discovered_at_unix_seconds: existing
                .as_ref()
                .and_then(|target| target.last_discovered_at_unix_seconds),
            last_scheduler_run_at_unix_seconds: existing
                .as_ref()
                .and_then(|target| target.last_scheduler_run_at_unix_seconds),
        };
        self.storage.store_passive_region_target(&target)?;
        Ok(target)
    }

    /// Bootstraps a predefined set of passive region targets into storage.
    ///
    /// When `request.overwrite_existing` is `true`, existing regions from the defaults are updated; otherwise
    /// existing regions are left unchanged and counted as skipped. The method returns counts and lists of
    /// region IDs that were inserted, updated, or skipped, along with the generation timestamp.
    ///
    /// # Parameters
    ///
    /// - `request`: controls whether existing regions should be overwritten (`overwrite_existing`).
    ///
    /// # Returns
    ///
    /// A `PassiveRegionDefaultsBootstrapResponse` containing the generation time, total defaults processed,
    /// counts of inserted/updated/skipped regions, and vectors of the affected region IDs.
    ///
    /// # Examples
    ///
    /// ```
    /// let req = PassiveRegionDefaultsBootstrapRequest { overwrite_existing: Some(false) };
    /// let resp = app_state.bootstrap_default_passive_regions(&req).unwrap();
    /// assert!(resp.total_defaults >= resp.inserted_count + resp.updated_count + resp.skipped_count);
    /// ```
    pub fn bootstrap_default_passive_regions(
        &self,
        request: &PassiveRegionDefaultsBootstrapRequest,
    ) -> Result<PassiveRegionDefaultsBootstrapResponse, AppError> {
        let overwrite_existing = request.overwrite_existing.unwrap_or(false);
        let existing = self
            .storage
            .passive_region_targets(1_000, false)?
            .into_iter()
            .map(|region| region.region_id)
            .collect::<BTreeSet<_>>();
        let defaults = default_passive_region_requests();

        let mut inserted_region_ids = Vec::new();
        let mut updated_region_ids = Vec::new();
        let mut skipped_region_ids = Vec::new();

        for region in defaults {
            let region_id = region.region_id.clone().unwrap_or_default();
            if existing.contains(&region_id) && !overwrite_existing {
                skipped_region_ids.push(region_id);
                continue;
            }
            let stored = self.upsert_passive_region_target(&region)?;
            if existing.contains(&stored.region_id) {
                updated_region_ids.push(stored.region_id);
            } else {
                inserted_region_ids.push(stored.region_id);
            }
        }

        Ok(PassiveRegionDefaultsBootstrapResponse {
            generated_at_unix_seconds: now_unix_seconds(),
            total_defaults: inserted_region_ids.len()
                + updated_region_ids.len()
                + skipped_region_ids.len(),
            inserted_count: inserted_region_ids.len(),
            updated_count: updated_region_ids.len(),
            skipped_count: skipped_region_ids.len(),
            inserted_region_ids,
            updated_region_ids,
            skipped_region_ids,
        })
    }

    /// Retrieve passive region targets from storage.
    ///
    /// Returns a list of stored `PassiveRegionTarget` records, limited to `limit` entries.
    /// When `enabled_only` is true, only targets with `enabled == true` are returned.
    ///
    /// # Errors
    ///
    /// Returns a `StorageError` if the underlying storage retrieval fails.
    ///
    /// # Examples
    ///
    /// ```
    /// // `state` is an instance of `AppState`
    /// let targets = state.passive_region_targets(10, true).unwrap();
    /// assert!(targets.len() <= 10);
    /// ```
    pub fn passive_region_targets(
        &self,
        limit: usize,
        enabled_only: bool,
    ) -> Result<Vec<PassiveRegionTarget>, StorageError> {
        self.storage.passive_region_targets(limit, enabled_only)
    }

    pub fn passive_region_target(
        &self,
        region_id: &str,
    ) -> Result<Option<PassiveRegionTarget>, StorageError> {
        self.storage.passive_region_target(region_id)
    }

    pub fn passive_region_run_logs(
        &self,
        limit: usize,
        region_id: Option<&str>,
    ) -> Result<Vec<PassiveRegionRunLog>, StorageError> {
        self.storage.passive_region_run_logs(limit, region_id)
    }

    pub fn passive_region_run_log(
        &self,
        run_id: &str,
    ) -> Result<Option<PassiveRegionRunLog>, StorageError> {
        self.storage.passive_region_run_log(run_id)
    }

    pub fn store_passive_region_run_log_entry(
        &self,
        log: &PassiveRegionRunLog,
    ) -> Result<(), StorageError> {
        self.storage.store_passive_region_run_log(log)
    }

    pub fn acquire_passive_region_lease(
        &self,
        region_id: &str,
        worker_id: &str,
        run_id: &str,
        now_unix_seconds: i64,
        ttl_seconds: i64,
    ) -> Result<Option<PassiveRegionLease>, StorageError> {
        self.storage.acquire_passive_region_lease(
            region_id,
            worker_id,
            run_id,
            now_unix_seconds,
            ttl_seconds,
        )
    }

    pub fn refresh_passive_region_lease(
        &self,
        region_id: &str,
        worker_id: &str,
        run_id: &str,
        now_unix_seconds: i64,
        ttl_seconds: i64,
    ) -> Result<bool, StorageError> {
        self.storage.refresh_passive_region_lease(
            region_id,
            worker_id,
            run_id,
            now_unix_seconds,
            ttl_seconds,
        )
    }

    pub fn release_passive_region_lease(
        &self,
        region_id: &str,
        worker_id: &str,
        run_id: &str,
    ) -> Result<bool, StorageError> {
        self.storage
            .release_passive_region_lease(region_id, worker_id, run_id)
    }

    pub fn passive_region_leases(
        &self,
        limit: usize,
    ) -> Result<Vec<PassiveRegionLease>, StorageError> {
        self.storage.passive_region_leases(limit)
    }

    pub fn purge_expired_passive_region_leases(
        &self,
        now_unix_seconds: i64,
    ) -> Result<usize, StorageError> {
        self.storage
            .purge_expired_passive_region_leases(now_unix_seconds)
    }

    pub fn store_passive_worker_heartbeat(
        &self,
        heartbeat: &PassiveWorkerHeartbeat,
    ) -> Result<(), StorageError> {
        self.storage.store_passive_worker_heartbeat(heartbeat)
    }

    pub fn passive_worker_heartbeats(
        &self,
        limit: usize,
    ) -> Result<Vec<PassiveWorkerHeartbeat>, StorageError> {
        self.storage.passive_worker_heartbeats(limit)
    }

    pub fn purge_stale_passive_worker_heartbeats(
        &self,
        stale_before_unix_seconds: i64,
    ) -> Result<usize, StorageError> {
        self.storage
            .purge_stale_passive_worker_heartbeats(stale_before_unix_seconds)
    }

    pub fn stale_passive_worker_heartbeats(
        &self,
        limit: usize,
        stale_before_unix_seconds: i64,
    ) -> Result<Vec<PassiveWorkerHeartbeat>, StorageError> {
        self.storage
            .stale_passive_worker_heartbeats(limit, stale_before_unix_seconds)
    }

    pub fn count_stale_passive_worker_heartbeats(
        &self,
        stale_before_unix_seconds: i64,
    ) -> Result<usize, StorageError> {
        self.storage
            .count_stale_passive_worker_heartbeats(stale_before_unix_seconds)
    }

    pub fn passive_source_health_samples(
        &self,
        limit: usize,
        source_kind: Option<PassiveSourceKind>,
        region_id: Option<&str>,
    ) -> Result<Vec<PassiveSourceHealthSample>, StorageError> {
        self.storage
            .passive_source_health_samples(limit, source_kind, region_id)
    }

    pub fn purge_passive_source_health_samples(
        &self,
        older_than_unix_seconds: i64,
        source_kind: Option<PassiveSourceKind>,
        region_id: Option<&str>,
    ) -> Result<usize, StorageError> {
        self.storage.purge_passive_source_health_samples(
            older_than_unix_seconds,
            source_kind,
            region_id,
        )
    }

    pub fn count_passive_source_health_samples_before(
        &self,
        older_than_unix_seconds: i64,
        source_kind: Option<PassiveSourceKind>,
        region_id: Option<&str>,
    ) -> Result<usize, StorageError> {
        self.storage.count_passive_source_health_samples_before(
            older_than_unix_seconds,
            source_kind,
            region_id,
        )
    }

    pub fn store_passive_source_health_sample_entries(
        &self,
        samples: &[PassiveSourceHealthSample],
    ) -> Result<(), StorageError> {
        self.storage.store_passive_source_health_samples(samples)
    }

    pub fn passive_source_readiness(
        &self,
        region_id: Option<&str>,
    ) -> Result<Vec<PassiveSourceReadiness>, AppError> {
        let samples = self.passive_source_health_samples(500, None, region_id)?;
        let runs = self.passive_region_run_logs(25, region_id)?;

        let weather = readiness_from_samples(
            "open_meteo",
            "Open-Meteo",
            true,
            true,
            samples
                .iter()
                .filter(|sample| sample.source_kind == PassiveSourceKind::Weather),
            "Weather forecasting is enabled by default and will begin reporting once passive scans run.",
        );
        let firms = readiness_from_samples(
            "firms",
            "NASA FIRMS",
            self.firms_map_key.is_some(),
            self.firms_map_key.is_some(),
            samples
                .iter()
                .filter(|sample| sample.source_kind == PassiveSourceKind::FireSmoke),
            "Set SSS_FIRMS_MAP_KEY to enable fire and smoke anomaly scans.",
        );
        let opensky = readiness_from_samples(
            "opensky",
            "OpenSky ADS-B",
            self.opensky_bearer_token.is_some(),
            self.opensky_bearer_token.is_some(),
            samples
                .iter()
                .filter(|sample| sample.source_kind == PassiveSourceKind::Adsb),
            "Set SSS_OPENSKY_BEARER_TOKEN to enable correlated air-traffic scans.",
        );

        let overpass_error = runs
            .iter()
            .flat_map(|run| run.source_errors.iter())
            .find(|error| error.to_ascii_lowercase().contains("overpass"))
            .cloned();
        let overpass_used = runs.iter().any(|run| {
            run.sources_used
                .iter()
                .any(|source| source.to_ascii_lowercase().contains("overpass"))
        });
        let overpass = PassiveSourceReadiness {
            source_id: "overpass".to_string(),
            label: "Overpass Discovery".to_string(),
            configured: true,
            enabled: true,
            readiness: if overpass_error.is_some() {
                PassiveSourceReadinessLevel::Degraded
            } else if overpass_used {
                PassiveSourceReadinessLevel::Ready
            } else {
                PassiveSourceReadinessLevel::AwaitingData
            },
            reason: if let Some(error) = overpass_error.clone() {
                format!("Discovery endpoint failed recently: {error}")
            } else if overpass_used {
                "Discovery endpoint has already contributed seeds to passive regions.".to_string()
            } else {
                "Discovery endpoint configured; next region discovery run will populate site candidates.".to_string()
            },
            latest_sample_at_unix_seconds: None,
            sample_count: usize::from(overpass_used),
            last_error: overpass_error,
        };

        let nasa_configured = !self.nasa_api_key.trim().is_empty();
        let nasa_mode_reason = if self.nasa_api_key.trim() == "DEMO_KEY" {
            "NASA briefings are using DEMO_KEY; expect lower limits until a private API key is configured."
        } else {
            "NASA briefing endpoints are configured for APOD and NeoWS context."
        };
        let nasa = PassiveSourceReadiness {
            source_id: "nasa_briefings".to_string(),
            label: "NASA Briefings".to_string(),
            configured: nasa_configured,
            enabled: true,
            readiness: if nasa_configured {
                PassiveSourceReadinessLevel::Ready
            } else {
                PassiveSourceReadinessLevel::NeedsConfig
            },
            reason: nasa_mode_reason.to_string(),
            latest_sample_at_unix_seconds: None,
            sample_count: 0,
            last_error: None,
        };

        Ok(vec![weather, firms, opensky, overpass, nasa])
    }

    pub fn canonical_passive_events(
        &self,
        limit: usize,
        region_id: Option<&str>,
        site_id: Option<&str>,
    ) -> Result<Vec<CanonicalPassiveEvent>, AppError> {
        let mut events = self
            .storage
            .canonical_passive_events(limit.saturating_mul(10), site_id)?;
        if let Some(region_id) = region_id {
            let Some(region) = self.storage.passive_region_target(region_id)? else {
                return Ok(Vec::new());
            };
            let records = self.storage.passive_seed_records(10_000)?;
            events.retain(|event| {
                records.iter().any(|record| {
                    record.site_id.as_deref() == Some(event.site_id.as_str())
                        && passive_seed_matches_region(record, &region)
                })
            });
        }
        events.sort_by(|left, right| {
            right
                .max_risk_score
                .total_cmp(&left.max_risk_score)
                .then_with(|| {
                    right
                        .last_seen_at_unix_seconds
                        .cmp(&left.last_seen_at_unix_seconds)
                })
        });
        events.truncate(limit);
        Ok(events)
    }

    pub fn canonical_passive_event(
        &self,
        canonical_event_id: &str,
    ) -> Result<Option<sss_storage::CanonicalPassiveEvent>, StorageError> {
        self.storage.canonical_passive_event(canonical_event_id)
    }

    pub fn passive_operations_status(&self) -> Result<PassiveOperationsStatus, AppError> {
        let now = now_unix_seconds();
        let all_regions = self.storage.passive_region_targets(10_000, false)?;
        let enabled_regions = all_regions
            .iter()
            .filter(|region| region.enabled)
            .collect::<Vec<_>>();
        let due_region_count = enabled_regions
            .iter()
            .filter(|region| {
                passive_region_overdue_seconds(region, now, region.discovery_cadence_seconds) >= 0
            })
            .count();
        let seeds = self.storage.passive_seed_records(10_000)?;
        let observed_seed_count = seeds
            .iter()
            .filter(|record| record.last_scanned_at_unix_seconds.is_some())
            .count();
        let elevated_seed_count = seeds
            .iter()
            .filter(|record| passive_seed_is_elevated(record))
            .count();
        let run_logs = self.storage.passive_region_run_logs(50, None)?;
        let latest_region_run = run_logs
            .iter()
            .find(|log| log.origin == PassiveRunOrigin::Worker)
            .cloned()
            .or_else(|| run_logs.into_iter().next());
        let all_region_leases = self.storage.passive_region_leases(100)?;
        let active_region_leases = self
            .passive_region_leases(100)?
            .into_iter()
            .filter(|lease| lease.expires_at_unix_seconds > now)
            .collect::<Vec<_>>();
        let worker_heartbeats = self.storage.passive_worker_heartbeats(100)?;
        let stale_worker_cutoff = now.saturating_sub(180);
        let stale_region_lease_count = all_region_leases
            .iter()
            .filter(|lease| lease.expires_at_unix_seconds <= now)
            .count();
        let stale_worker_heartbeat_count = worker_heartbeats
            .iter()
            .filter(|heartbeat| heartbeat.last_heartbeat_unix_seconds < stale_worker_cutoff)
            .count();

        Ok(PassiveOperationsStatus {
            generated_at_unix_seconds: now,
            total_region_count: all_regions.len(),
            enabled_region_count: enabled_regions.len(),
            due_region_count,
            active_region_lease_count: active_region_leases.len(),
            stale_region_lease_count,
            worker_heartbeat_count: worker_heartbeats.len(),
            stale_worker_heartbeat_count,
            seed_count: seeds.len(),
            observed_seed_count,
            elevated_seed_count,
            latest_region_run,
            active_region_leases,
            worker_heartbeats,
            recommendation: passive_operations_recommendation(
                enabled_regions.len(),
                due_region_count,
                seeds.len(),
                observed_seed_count,
                elevated_seed_count,
            ),
        })
    }

    pub fn passive_worker_diagnostics(
        &self,
        limit: usize,
        stale_after_seconds: i64,
        region_id: Option<&str>,
    ) -> Result<PassiveWorkerDiagnostics, AppError> {
        let now = now_unix_seconds();
        let stale_cutoff_unix_seconds = now.saturating_sub(stale_after_seconds.max(1));
        let all_region_leases = self.storage.passive_region_leases(500)?;
        let scoped_region_leases = all_region_leases
            .iter()
            .filter(|lease| region_id.is_none_or(|expected| lease.region_id == expected))
            .cloned()
            .collect::<Vec<_>>();
        let active_region_leases = scoped_region_leases
            .iter()
            .filter(|lease| lease.expires_at_unix_seconds > now)
            .map(|lease| passive_region_lease_diagnostic(lease, now))
            .collect::<Vec<_>>();
        let stale_region_leases = scoped_region_leases
            .iter()
            .filter(|lease| lease.expires_at_unix_seconds <= now)
            .map(|lease| passive_region_lease_diagnostic(lease, now))
            .collect::<Vec<_>>();
        let worker_heartbeats = self
            .storage
            .passive_worker_heartbeats(500)?
            .into_iter()
            .filter(|heartbeat| {
                region_id
                    .is_none_or(|expected| heartbeat.current_region_id.as_deref() == Some(expected))
            })
            .collect::<Vec<_>>();
        let stale_worker_heartbeats = worker_heartbeats
            .iter()
            .filter(|heartbeat| heartbeat.last_heartbeat_unix_seconds < stale_cutoff_unix_seconds)
            .take(limit)
            .cloned()
            .collect::<Vec<_>>();
        let total_worker_heartbeat_count = worker_heartbeats.len();
        let stale_worker_heartbeat_count = worker_heartbeats
            .iter()
            .filter(|heartbeat| heartbeat.last_heartbeat_unix_seconds < stale_cutoff_unix_seconds)
            .count();
        let active_worker_heartbeat_count =
            total_worker_heartbeat_count.saturating_sub(stale_worker_heartbeat_count);
        let region_run_logs = self.storage.passive_region_run_logs(250, region_id)?;
        let region_metrics = passive_worker_region_metrics(
            &region_run_logs,
            &scoped_region_leases,
            &worker_heartbeats,
            now,
            stale_cutoff_unix_seconds,
            limit,
        );
        let source_metrics = passive_worker_source_metrics(
            &self
                .storage
                .passive_source_health_samples(500, None, region_id)?,
            region_id,
            limit.saturating_mul(3),
        );
        let recommendation = if stale_worker_heartbeat_count > 0 || !stale_region_leases.is_empty()
        {
            format!(
                "{} stale worker heartbeats and {} stale region leases detected. Review worker liveness, lease ownership, and prune abandoned entries if they are no longer valid.",
                stale_worker_heartbeat_count,
                stale_region_leases.len()
            )
        } else if total_worker_heartbeat_count == 0 {
            if let Some(region_id) = region_id {
                format!(
                    "No worker heartbeats are currently scoped to region {region_id}. Check worker cadence or open global diagnostics."
                )
            } else {
                "No worker heartbeats are currently registered. Start sss-passive-worker to restore autonomous observation."
                    .to_string()
            }
        } else {
            "Worker heartbeat diagnostics look healthy.".to_string()
        };

        Ok(PassiveWorkerDiagnostics {
            generated_at_unix_seconds: now,
            region_id: region_id.map(str::to_string),
            stale_cutoff_unix_seconds,
            total_worker_heartbeat_count,
            stale_worker_heartbeat_count,
            active_worker_heartbeat_count,
            active_region_lease_count: active_region_leases.len(),
            stale_region_lease_count: stale_region_leases.len(),
            stale_worker_heartbeats,
            active_region_leases,
            stale_region_leases,
            region_metrics,
            source_metrics,
            recommendation,
        })
    }

    pub fn passive_region_overview(
        &self,
        region_id: &str,
        site_limit: usize,
        days: u32,
    ) -> Result<Option<PassiveRegionOverview>, AppError> {
        let Some(region) = self.storage.passive_region_target(region_id)? else {
            return Ok(None);
        };
        let now = now_unix_seconds();
        let records = self.storage.passive_seed_records(10_000)?;
        let mut region_records = records
            .into_iter()
            .filter(|record| passive_seed_matches_region(record, &region))
            .collect::<Vec<_>>();
        let seed_count = region_records.len();
        let observed_seed_count = region_records
            .iter()
            .filter(|record| record.last_scanned_at_unix_seconds.is_some())
            .count();
        let elevated_seed_count = region_records
            .iter()
            .filter(|record| passive_seed_is_elevated(record))
            .count();
        let highest_scan_priority = region_records
            .iter()
            .map(|record| record.scan_priority)
            .fold(0.0, f64::max);
        let average_observation_confidence =
            average_seed_confidence(&region_records).unwrap_or_default();
        let mut summaries = Vec::new();

        for record in region_records.drain(..) {
            summaries.push(self.passive_region_site_summary(record, days)?);
        }
        summaries.sort_by(|left, right| {
            region_site_sort_score(right)
                .partial_cmp(&region_site_sort_score(left))
                .unwrap_or(Ordering::Equal)
        });
        summaries.truncate(site_limit.clamp(1, 50));

        let recent_event_count = summaries.iter().map(|site| site.recent_event_count).sum();
        let critical_event_count = summaries.iter().map(|site| site.critical_event_count).sum();
        let discovery_overdue_seconds =
            passive_region_overdue_seconds(&region, now, region.discovery_cadence_seconds);
        let discovery_due = discovery_overdue_seconds >= 0;
        let narrative_stats = PassiveRegionNarrativeStats {
            days,
            seed_count,
            observed_seed_count,
            recent_event_count,
            critical_event_count,
            highest_scan_priority,
            average_observation_confidence,
            discovery_due,
        };
        let provenance = passive_region_narrative_provenance(&region.region_id, &summaries);

        Ok(Some(PassiveRegionOverview {
            generated_at_unix_seconds: now,
            narrative: passive_region_overview_narrative(&region, &narrative_stats),
            region,
            seed_count,
            observed_seed_count,
            elevated_seed_count,
            recent_event_count,
            critical_event_count,
            highest_scan_priority,
            average_observation_confidence,
            discovery_due,
            discovery_overdue_seconds,
            top_sites: summaries,
            provenance,
        }))
    }

    fn passive_region_site_summary(
        &self,
        record: PassiveSeedRecord,
        days: u32,
    ) -> Result<PassiveRegionSiteSummary, AppError> {
        let overview = record
            .site_id
            .as_ref()
            .map(|site_id| self.passive_site_overview(site_id, 10, days))
            .transpose()?;
        Ok(passive_region_site_summary(record, overview.as_ref()))
    }

    pub async fn run_passive_regions(
        &self,
        request_id: &str,
        request: &PassiveRegionRunRequest,
    ) -> Result<PassiveRegionRunResponse, AppError> {
        let request_json = serde_json::to_string(request)?;
        let started_at_unix_seconds = now_unix_seconds();
        let started_at = std::time::Instant::now();
        let now = now_unix_seconds();
        let force_discovery = request.force_discovery.unwrap_or(false);
        let dry_run = request.dry_run.unwrap_or(false);
        let regions = self.passive_regions_for_run(request)?;

        let evaluated_region_count = regions.len();
        let mut run_records = Vec::new();
        let mut discovered_seed_count = 0usize;
        let mut discovered_region_count = 0usize;
        let mut scan_limit = 0usize;
        let mut minimum_priority: Option<f64> = None;

        for mut region in regions {
            let overdue_seconds =
                passive_region_overdue_seconds(&region, now, region.discovery_cadence_seconds);
            let due = force_discovery || overdue_seconds >= 0;
            if !due {
                run_records.push(PassiveRegionRunRecord {
                    region,
                    due: false,
                    skipped_reason: Some(format!(
                        "discovery cadence is fresh for {} more seconds",
                        overdue_seconds.saturating_abs()
                    )),
                    discovery: None,
                });
                continue;
            }

            scan_limit = scan_limit.saturating_add(region.scan_limit).clamp(1, 200);
            minimum_priority = Some(minimum_priority.map_or(region.minimum_priority, |current| {
                current.min(region.minimum_priority)
            }));

            if dry_run {
                run_records.push(PassiveRegionRunRecord {
                    region,
                    due: true,
                    skipped_reason: Some("dry run requested; discovery not executed".to_string()),
                    discovery: None,
                });
                continue;
            }

            let discovery = self
                .discover_passive_infra_sites(
                    &format!("{request_id}:discover:{}", region.region_id),
                    &passive_region_discovery_request(&region),
                )
                .await?;
            discovered_seed_count = discovered_seed_count.saturating_add(discovery.seeds.len());
            discovered_region_count = discovered_region_count.saturating_add(1);
            region.last_discovered_at_unix_seconds = Some(now);
            region.updated_at_unix_seconds = now;
            self.storage.store_passive_region_target(&region)?;
            run_records.push(PassiveRegionRunRecord {
                region,
                due: true,
                skipped_reason: None,
                discovery: Some(discovery),
            });
        }

        let scheduler = if scan_limit > 0 {
            let scheduler_response = self
                .run_passive_scheduler(
                    &format!("{request_id}:scheduler"),
                    &passive_region_scheduler_request(
                        request,
                        scan_limit,
                        minimum_priority,
                        dry_run,
                    ),
                )
                .await?;
            Some(scheduler_response)
        } else {
            None
        };

        if !dry_run && scheduler.is_some() {
            self.mark_passive_region_scheduler_run(&mut run_records, now)?;
        }

        let response = PassiveRegionRunResponse {
            generated_at_unix_seconds: now_unix_seconds(),
            evaluated_region_count,
            discovered_region_count,
            skipped_region_count: evaluated_region_count.saturating_sub(discovered_region_count),
            discovered_seed_count,
            scheduler,
            regions: run_records,
        };
        self.store_passive_region_run_log(
            request_id,
            request,
            &response,
            started_at_unix_seconds,
            started_at.elapsed(),
        )?;
        self.log_passive_region_run_response(request_id, &request_json, &response)?;
        Ok(response)
    }

    fn passive_regions_for_run(
        &self,
        request: &PassiveRegionRunRequest,
    ) -> Result<Vec<PassiveRegionTarget>, AppError> {
        let requested_region_ids = request.region_ids.as_ref().map(|ids| {
            ids.iter()
                .cloned()
                .collect::<std::collections::BTreeSet<_>>()
        });
        let mut regions = self.storage.passive_region_targets(1_000, true)?;
        if let Some(region_ids) = requested_region_ids.as_ref() {
            regions.retain(|region| region_ids.contains(&region.region_id));
        }
        Ok(regions)
    }

    fn store_passive_region_run_log(
        &self,
        request_id: &str,
        request: &PassiveRegionRunRequest,
        response: &PassiveRegionRunResponse,
        started_at_unix_seconds: i64,
        elapsed: std::time::Duration,
    ) -> Result<(), AppError> {
        let selected_seed_count = response
            .scheduler
            .as_ref()
            .map_or(0, |scheduler| scheduler.selected_seed_count);
        let event_count = response
            .scheduler
            .as_ref()
            .and_then(|scheduler| scheduler.execution_summary.as_ref())
            .map_or(0, |summary| summary.event_count);
        let sources_used = response
            .scheduler
            .as_ref()
            .and_then(|scheduler| scheduler.live_scan.as_ref())
            .map_or_else(Vec::new, |scan| {
                scan.source_statuses
                    .iter()
                    .filter(|status| status.fetched)
                    .map(|status| format!("{:?}", status.source_kind))
                    .collect()
            });
        let source_errors = response
            .scheduler
            .as_ref()
            .and_then(|scheduler| scheduler.live_scan.as_ref())
            .map_or_else(Vec::new, |scan| {
                scan.source_statuses
                    .iter()
                    .filter(|status| !status.fetched)
                    .map(|status| status.detail.clone())
                    .collect()
            });
        let log = PassiveRegionRunLog {
            run_id: format!("{request_id}:region-run"),
            request_id: request_id.to_string(),
            origin: PassiveRunOrigin::Api,
            started_at_unix_seconds,
            finished_at_unix_seconds: now_unix_seconds(),
            duration_ms: u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX),
            status: passive_region_run_status(request, response),
            region_ids: response
                .regions
                .iter()
                .map(|record| record.region.region_id.clone())
                .collect(),
            discovery_triggered: response.discovered_region_count > 0,
            evaluated_region_count: response.evaluated_region_count,
            discovered_region_count: response.discovered_region_count,
            skipped_region_count: response.skipped_region_count,
            discovered_seed_count: response.discovered_seed_count,
            selected_seed_count,
            event_count,
            sources_used,
            source_errors,
            next_run_at_unix_seconds: response
                .regions
                .iter()
                .filter_map(|record| {
                    record
                        .region
                        .last_discovered_at_unix_seconds
                        .map(|last_discovered| {
                            last_discovered.saturating_add(record.region.discovery_cadence_seconds)
                        })
                })
                .max(),
        };
        self.storage.store_passive_region_run_log(&log)?;
        if let Some(live_scan) = response
            .scheduler
            .as_ref()
            .and_then(|scheduler| scheduler.live_scan.as_ref())
        {
            for region_id in &log.region_ids {
                self.store_passive_source_health_samples(request_id, Some(region_id), live_scan)?;
            }
        }
        Ok(())
    }

    fn log_passive_region_run_response(
        &self,
        request_id: &str,
        request_json: &str,
        response: &PassiveRegionRunResponse,
    ) -> Result<(), AppError> {
        self.storage.log_exchange(&RequestResponseLog {
            request_id: request_id.to_string(),
            endpoint: "/v1/passive/regions/run".to_string(),
            object_id: None,
            timestamp_unix_seconds: now_unix_seconds(),
            request_json: request_json.to_string(),
            response_json: serde_json::to_string(response)?,
        })?;
        Ok(())
    }

    fn mark_passive_region_scheduler_run(
        &self,
        run_records: &mut [PassiveRegionRunRecord],
        now_unix_seconds: i64,
    ) -> Result<(), AppError> {
        for record in run_records {
            if record.due {
                record.region.last_scheduler_run_at_unix_seconds = Some(now_unix_seconds);
                record.region.updated_at_unix_seconds = now_unix_seconds;
                self.storage.store_passive_region_target(&record.region)?;
            }
        }
        Ok(())
    }

    pub fn passive_observations(
        &self,
        limit: usize,
        source_kind: Option<PassiveSourceKind>,
    ) -> Result<Vec<PassiveObservation>, StorageError> {
        self.storage.passive_observations(limit, source_kind)
    }

    pub fn passive_events_for_site(
        &self,
        site_id: &str,
        limit: usize,
    ) -> Result<Vec<PassiveEvent>, StorageError> {
        self.storage.passive_events_for_site(site_id, limit)
    }

    pub fn passive_risk_history(
        &self,
        site_id: &str,
        limit: usize,
    ) -> Result<Vec<PassiveRiskRecord>, StorageError> {
        self.storage.passive_risk_history(site_id, limit)
    }

    pub fn passive_patterns(
        &self,
        site_id: &str,
        limit: usize,
    ) -> Result<Vec<PatternSignal>, StorageError> {
        self.storage.passive_patterns(site_id, limit)
    }

    pub fn passive_site_overview(
        &self,
        site_id: &str,
        limit: usize,
        narrative_days: u32,
    ) -> Result<PassiveSiteOverview, AppError> {
        let site = self
            .storage
            .passive_site_profiles()?
            .into_iter()
            .find(|candidate| candidate.site.site_id.to_string() == site_id)
            .ok_or_else(|| AppError::SiteNotFound(format!("passive site not found: {site_id}")))?;
        let seed_lifecycle = self.seed_record_for_site(&site)?;
        let recent_events = self.storage.passive_events_for_site(site_id, limit)?;
        let risk_history = self.storage.passive_risk_history(site_id, limit)?;
        let recurring_patterns = self.storage.passive_patterns(site_id, limit)?;
        let latest_risk = risk_history.first().cloned();
        let temporal_evolution = risk_history
            .iter()
            .rev()
            .map(|record| PassiveTemporalPoint {
                window_start_unix_seconds: record.window_start_unix_seconds,
                window_end_unix_seconds: record.window_end_unix_seconds,
                cumulative_risk: record.cumulative_risk,
                peak_risk: record.peak_risk,
                event_count: record.event_count,
            })
            .collect::<Vec<_>>();
        let risk_history_narrative =
            self.build_risk_history_narrative(site_id, narrative_days, Some(&site.site.name))?;

        Ok(PassiveSiteOverview {
            site,
            seed_lifecycle,
            recent_events,
            latest_risk,
            recurring_patterns,
            temporal_evolution,
            risk_history_narrative,
        })
    }

    pub fn passive_risk_history_narrative(
        &self,
        site_id: &str,
        days: u32,
    ) -> Result<RiskHistoryNarrative, AppError> {
        self.build_risk_history_narrative(site_id, days, None)
    }

    async fn passive_site_forecast_context(
        &self,
        site_id: &str,
        horizon_hours: u32,
    ) -> Result<SiteForecastInput, AppError> {
        let site_profile = self
            .storage
            .passive_site_profiles()?
            .into_iter()
            .find(|candidate| candidate.site.site_id.to_string() == site_id)
            .ok_or_else(|| AppError::SiteNotFound(format!("passive site not found: {site_id}")))?;
        let seed_lifecycle = self.seed_record_for_site(&site_profile)?;
        let latest_risk = self
            .storage
            .passive_risk_history(site_id, 1)?
            .into_iter()
            .next()
            .map_or(0.0, |risk| risk.peak_risk);
        let weather_now = self.fetch_site_weather_now(&site_profile.site).await?;

        Ok(SiteForecastInput {
            site: site_profile.site,
            generated_at_unix_seconds: now_unix_seconds(),
            horizon_hours: horizon_hours.clamp(1, 72),
            observation_confidence: seed_lifecycle.map_or(0.52, |record| record.confidence),
            latest_risk,
            weather_now,
        })
    }

    pub async fn apod_briefing(
        &self,
        request_id: &str,
        date: Option<&str>,
    ) -> Result<ApodBriefing, AppError> {
        let request_json = serde_json::to_string(&serde_json::json!({
            "source": "nasa-apod",
            "date": date,
        }))?;
        let mut request = self.http_client.get(format!(
            "{}/planetary/apod",
            self.nasa_api_base_url.trim_end_matches('/')
        ));
        request = request.query(&[("api_key", self.nasa_api_key.as_str())]);
        if let Some(date) = date {
            request = request.query(&[("date", date)]);
        }
        let response = request
            .send()
            .await
            .map_err(|error| AppError::SourceUnavailable(error.to_string()))?;
        let response = response
            .error_for_status()
            .map_err(|error| AppError::SourceUnavailable(error.to_string()))?;
        let payload = response
            .text()
            .await
            .map_err(|error| AppError::SourceUnavailable(error.to_string()))?;
        let payload: NasaApodApiResponse = serde_json::from_str(&payload)?;
        let briefing = ApodBriefing {
            title: payload.title,
            date: payload.date,
            explanation: payload.explanation,
            media_type: payload.media_type,
            url: payload.url,
            hdurl: payload.hdurl,
            copyright: payload.copyright,
            service_version: payload.service_version,
        };
        self.storage.log_exchange(&RequestResponseLog {
            request_id: request_id.to_string(),
            endpoint: "/v1/briefing/apod".to_string(),
            object_id: None,
            timestamp_unix_seconds: now_unix_seconds(),
            request_json,
            response_json: serde_json::to_string(&briefing)?,
        })?;
        Ok(briefing)
    }

    pub async fn neows_briefing(
        &self,
        request_id: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<NeoRiskBriefing, AppError> {
        validate_neows_date_range(start_date, end_date)?;
        let feed = self.neows_feed(request_id, start_date, end_date).await?;
        Ok(NeoRiskBriefing {
            generated_at_unix_seconds: feed.generated_at_unix_seconds,
            start_date: feed.start_date,
            end_date: feed.end_date,
            total_objects: feed.total_objects,
            hazardous_objects: feed.hazardous_objects,
            highest_priority: feed.objects.into_iter().take(3).collect(),
        })
    }

    #[allow(clippy::too_many_lines)]
    pub async fn neows_feed(
        &self,
        request_id: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<NeoRiskFeed, AppError> {
        validate_neows_date_range(start_date, end_date)?;
        let request_json = serde_json::to_string(&serde_json::json!({
            "source": "nasa-neows",
            "start_date": start_date,
            "end_date": end_date,
        }))?;
        let response = self
            .http_client
            .get(format!(
                "{}/neo/rest/v1/feed",
                self.nasa_api_base_url.trim_end_matches('/')
            ))
            .query(&[
                ("api_key", self.nasa_api_key.as_str()),
                ("start_date", start_date),
                ("end_date", end_date),
            ])
            .send()
            .await
            .map_err(|error| AppError::NasaApiUnavailable(error.to_string()))?;
        let response = response
            .error_for_status()
            .map_err(|error| AppError::NasaApiUnavailable(error.to_string()))?;
        let payload = response
            .text()
            .await
            .map_err(|error| AppError::NasaApiUnavailable(error.to_string()))?;
        let payload: NasaNeoWsFeedResponse = serde_json::from_str(&payload)?;
        let mut objects = payload
            .near_earth_objects
            .into_values()
            .flatten()
            .map(|object| {
                let approach = object.close_approach_data.first();
                let hazardous = object.is_potentially_hazardous_asteroid;
                let miss_distance_km =
                    approach.and_then(|item| item.miss_distance.kilometers.parse::<f64>().ok());
                let relative_velocity_km_s = approach.and_then(|item| {
                    item.relative_velocity
                        .kilometers_per_second
                        .parse::<f64>()
                        .ok()
                });
                let estimated_diameter_min_m =
                    Some(object.estimated_diameter.meters.estimated_diameter_min);
                let estimated_diameter_max_m =
                    Some(object.estimated_diameter.meters.estimated_diameter_max);
                let close_approach_date =
                    approach.and_then(|item| item.close_approach_date.clone());
                let priority_score = neows_priority_score(
                    hazardous,
                    miss_distance_km,
                    relative_velocity_km_s,
                    estimated_diameter_max_m,
                    close_approach_date.as_deref(),
                    start_date,
                );
                NeoRiskObject {
                    neo_reference_id: object.id,
                    name: object.name,
                    nasa_jpl_url: object.nasa_jpl_url,
                    hazardous,
                    close_approach_date: close_approach_date.clone(),
                    miss_distance_km,
                    relative_velocity_km_s,
                    estimated_diameter_min_m,
                    estimated_diameter_max_m,
                    priority_score,
                    briefing_summary: neows_briefing_summary(
                        hazardous,
                        close_approach_date.as_deref(),
                    ),
                }
            })
            .collect::<Vec<_>>();
        objects.sort_by(|left, right| {
            right
                .priority_score
                .total_cmp(&left.priority_score)
                .then_with(|| left.name.cmp(&right.name))
        });
        let hazardous_count = objects.iter().filter(|object| object.hazardous).count();
        let feed = NeoRiskFeed {
            generated_at_unix_seconds: now_unix_seconds(),
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
            total_objects: payload.element_count,
            hazardous_objects: hazardous_count,
            objects,
        };
        self.storage.log_exchange(&RequestResponseLog {
            request_id: request_id.to_string(),
            endpoint: "/v1/briefing/neows/feed".to_string(),
            object_id: None,
            timestamp_unix_seconds: now_unix_seconds(),
            request_json,
            response_json: serde_json::to_string(&feed)?,
        })?;
        Ok(feed)
    }

    pub fn ingest_source_status(
        &self,
        source: &str,
    ) -> Result<Option<IngestSourceStatus>, StorageError> {
        let Some(batch) = self.storage.latest_ingest_batch_for_source(source)? else {
            return Ok(None);
        };
        Ok(Some(IngestSourceStatus {
            source: batch.source,
            latest_request_id: batch.request_id,
            latest_timestamp_unix_seconds: batch.timestamp_unix_seconds,
            records_received: batch.records_received,
            observations_created: batch.observations_created,
            object_count: batch.object_ids.len(),
            freshness_seconds: now_unix_seconds().saturating_sub(batch.timestamp_unix_seconds),
        }))
    }

    #[must_use]
    pub fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    pub fn assessment_history(
        &self,
        object_id: &str,
    ) -> Result<Vec<sss_core::IntelligenceAssessment>, StorageError> {
        self.storage.assessment_history(object_id)
    }

    pub fn recent_request_logs(
        &self,
        limit: usize,
        endpoint: Option<&str>,
    ) -> Result<Vec<RequestResponseLog>, StorageError> {
        self.storage.recent_request_logs(limit, endpoint)
    }

    pub fn cached_neows_feed(&self) -> Result<Option<NeoRiskFeed>, AppError> {
        self.storage
            .recent_request_logs(1, Some("/v1/briefing/neows/feed"))?
            .into_iter()
            .next()
            .map(|entry| serde_json::from_str::<NeoRiskFeed>(&entry.response_json))
            .transpose()
            .map_err(AppError::from)
    }

    fn store_evidence_bundle(&self, bundle: &EvidenceBundle) -> Result<(), StorageError> {
        let manifest = replay_manifest_for(bundle);
        self.storage.store_evidence_bundle(bundle)?;
        self.storage.store_replay_manifest(&manifest)
    }

    fn store_passive_scan_response(
        &self,
        request_id: &str,
        endpoint: &str,
        request_json: &str,
        response: &PassiveScanOutput,
    ) -> Result<(), StorageError> {
        self.storage.store_passive_scan_output(response)?;
        self.storage.log_exchange(&RequestResponseLog {
            request_id: request_id.to_string(),
            endpoint: endpoint.to_string(),
            object_id: None,
            timestamp_unix_seconds: now_unix_seconds(),
            request_json: request_json.to_string(),
            response_json: serde_json::to_string(response)?,
        })
    }

    fn store_passive_source_health_samples(
        &self,
        request_id: &str,
        region_id: Option<&str>,
        response: &PassiveLiveScanResponse,
    ) -> Result<(), StorageError> {
        let region_key = region_id.unwrap_or("global");
        let samples = response
            .source_statuses
            .iter()
            .map(|status| PassiveSourceHealthSample {
                sample_id: format!(
                    "{request_id}:{region_key}:{:?}:{}",
                    status.source_kind, response.generated_at_unix_seconds
                ),
                request_id: request_id.to_string(),
                region_id: region_id.map(ToOwned::to_owned),
                source_kind: status.source_kind,
                fetched: status.fetched,
                observations_collected: status.observations_collected,
                detail: status.detail.clone(),
                generated_at_unix_seconds: response.generated_at_unix_seconds,
                window_start_unix_seconds: response.window_start_unix_seconds,
                window_end_unix_seconds: response.window_end_unix_seconds,
            })
            .collect::<Vec<_>>();
        self.storage.store_passive_source_health_samples(&samples)
    }

    async fn build_passive_live_scan_response(
        &self,
        request: &PassiveLiveScanRequest,
    ) -> Result<PassiveLiveScanResponse, AppError> {
        let window_hours = request.window_hours.unwrap_or(24).clamp(1, 168);
        let window_end_unix_seconds = now_unix_seconds();
        let window_start_unix_seconds = window_end_unix_seconds.saturating_sub(
            (window_hours.saturating_mul(3_600))
                .try_into()
                .unwrap_or(i64::MAX),
        );
        let include_adsb = request.include_adsb.unwrap_or(true);
        let include_weather = request.include_weather.unwrap_or(true);
        let include_fire_smoke = request.include_fire_smoke.unwrap_or(true);
        let mut adsb_observations = Vec::new();
        let mut weather_observations = Vec::new();
        let mut fire_smoke_observations = Vec::new();
        let mut source_statuses = Vec::new();

        if include_adsb {
            if self.opensky_bearer_token.is_some() {
                let mut collected = Vec::new();
                for site in &request.infra_sites {
                    collected.extend(self.fetch_opensky_observations(site).await?);
                }
                source_statuses.push(PassiveLiveSourceStatus {
                    source_kind: PassiveSourceKind::Adsb,
                    fetched: true,
                    observations_collected: collected.len(),
                    detail: "OpenSky state vectors correlated against passive site bounding boxes."
                        .to_string(),
                });
                adsb_observations = collected;
            } else {
                source_statuses.push(PassiveLiveSourceStatus {
                    source_kind: PassiveSourceKind::Adsb,
                    fetched: false,
                    observations_collected: 0,
                    detail:
                        "OpenSky bearer token not configured; ADS-B collection skipped for this scan."
                            .to_string(),
                });
            }
        }

        if include_weather {
            let mut collected = Vec::new();
            for site in &request.infra_sites {
                collected.push(self.fetch_open_meteo_observation(site).await?);
            }
            source_statuses.push(PassiveLiveSourceStatus {
                source_kind: PassiveSourceKind::Weather,
                fetched: true,
                observations_collected: collected.len(),
                detail: "Open-Meteo current weather pulled per site.".to_string(),
            });
            weather_observations = collected;
        }

        if include_fire_smoke {
            if self.firms_map_key.is_some() {
                let mut collected = Vec::new();
                for site in &request.infra_sites {
                    collected.extend(self.fetch_firms_observations(site).await?);
                }
                source_statuses.push(PassiveLiveSourceStatus {
                    source_kind: PassiveSourceKind::FireSmoke,
                    fetched: true,
                    observations_collected: collected.len(),
                    detail: "NASA FIRMS area CSV pulled around each passive site.".to_string(),
                });
                fire_smoke_observations = collected;
            } else {
                source_statuses.push(PassiveLiveSourceStatus {
                    source_kind: PassiveSourceKind::FireSmoke,
                    fetched: false,
                    observations_collected: 0,
                    detail:
                        "FIRMS MAP_KEY not configured; fire and smoke collection skipped for this scan."
                            .to_string(),
                });
            }
        }

        let scan_input = PassiveScanInput {
            window_start_unix_seconds,
            window_end_unix_seconds,
            infra_sites: request.infra_sites.clone(),
            adsb_observations,
            weather_observations,
            fire_smoke_observations,
            orbital_observations: Vec::new(),
            satellite_observations: Vec::new(),
            notam_observations: Vec::new(),
        };
        let scan = self.passive_scanner.scan(&scan_input);

        Ok(PassiveLiveScanResponse {
            generated_at_unix_seconds: now_unix_seconds(),
            window_start_unix_seconds,
            window_end_unix_seconds,
            source_statuses,
            scan,
        })
    }

    fn build_risk_history_narrative(
        &self,
        site_id: &str,
        days: u32,
        site_name_hint: Option<&str>,
    ) -> Result<RiskHistoryNarrative, AppError> {
        let risk_history = self.storage.passive_risk_history(site_id, 256)?;
        let events = self.storage.passive_events_for_site(site_id, 256)?;
        let patterns = self.storage.passive_patterns(site_id, 256)?;
        let seed_confidence = self
            .storage
            .passive_site_profiles()?
            .into_iter()
            .find(|candidate| candidate.site.site_id.to_string() == site_id)
            .map(|site| self.seed_record_for_site(&site))
            .transpose()?
            .flatten()
            .map_or(0.0, |record| record.confidence);
        let site_name = site_name_hint.map_or_else(
            || {
                risk_history
                    .first()
                    .map(|record| record.site_name.clone())
                    .or_else(|| events.first().map(|event| event.site_name.clone()))
                    .unwrap_or_else(|| site_id.to_string())
            },
            std::string::ToString::to_string,
        );
        let anchor_unix_seconds = risk_history
            .iter()
            .map(|record| record.window_end_unix_seconds)
            .chain(events.iter().map(|event| event.observed_at_unix_seconds))
            .max()
            .unwrap_or_else(now_unix_seconds);
        let window_start_unix_seconds =
            anchor_unix_seconds.saturating_sub(i64::from(days.max(1)).saturating_mul(86_400));
        let filtered_records = risk_history
            .iter()
            .filter(|record| record.window_end_unix_seconds >= window_start_unix_seconds)
            .collect::<Vec<_>>();
        let filtered_events = events
            .iter()
            .filter(|event| event.observed_at_unix_seconds >= window_start_unix_seconds)
            .collect::<Vec<_>>();
        let filtered_patterns = patterns
            .iter()
            .filter(|pattern| pattern.recurring_events > 1)
            .collect::<Vec<_>>();
        if filtered_records.is_empty() && filtered_events.is_empty() && filtered_patterns.is_empty()
        {
            return self.build_seed_candidate_narrative(
                site_id,
                days,
                site_name,
                seed_confidence,
                window_start_unix_seconds,
                anchor_unix_seconds,
            );
        }
        let event_count = filtered_events.len();
        let recurring_pattern_count = filtered_patterns.len();
        let critical_event_count = filtered_events
            .iter()
            .filter(|event| event.risk_score >= 0.8)
            .count();
        let average_cumulative_risk = average_f64(
            &filtered_records
                .iter()
                .map(|record| record.cumulative_risk)
                .collect::<Vec<_>>(),
        );
        let latest_peak_risk = filtered_records
            .first()
            .map_or(0.0, |record| record.peak_risk);
        let baseline_peak_risk = filtered_records
            .last()
            .map_or(latest_peak_risk, |record| record.peak_risk);
        let risk_direction = risk_direction_label(baseline_peak_risk, latest_peak_risk).to_string();
        let narrative = format!(
            "Nos ultimos {days} dias: {event_count} eventos de anomalia, {recurring_pattern_count} padroes recorrentes e {critical_event_count} eventos criticos antecipados. O risco medio acumulado foi {average_cumulative_risk:.2}, o pico mais recente ficou em {latest_peak_risk:.2}, a confianca observacional atual esta em {seed_confidence:.2} e a tendencia de risco esta {risk_direction}."
        );
        let provenance = self.build_narrative_provenance(
            site_id,
            window_start_unix_seconds,
            anchor_unix_seconds,
        )?;

        Ok(RiskHistoryNarrative {
            site_id: site_id.to_string(),
            site_name,
            window_days: days.max(1),
            generated_at_unix_seconds: now_unix_seconds(),
            event_count,
            recurring_pattern_count,
            critical_event_count,
            average_cumulative_risk,
            latest_peak_risk,
            observation_confidence: seed_confidence,
            risk_direction,
            narrative,
            provenance,
        })
    }

    fn build_seed_candidate_narrative(
        &self,
        site_id: &str,
        days: u32,
        site_name: String,
        seed_confidence: f64,
        window_start_unix_seconds: i64,
        anchor_unix_seconds: i64,
    ) -> Result<RiskHistoryNarrative, AppError> {
        let narrative = format!(
            "Nos ultimos {days} dias: este site foi descoberto e perfilado, mas ainda nao acumulou historico operacional suficiente. A seed esta registada, pronta para observacao continua, e a confianca atual do candidato esta em {seed_confidence:.2}."
        );
        let provenance = self.build_narrative_provenance(
            site_id,
            window_start_unix_seconds,
            anchor_unix_seconds,
        )?;

        Ok(RiskHistoryNarrative {
            site_id: site_id.to_string(),
            site_name,
            window_days: days.max(1),
            generated_at_unix_seconds: now_unix_seconds(),
            event_count: 0,
            recurring_pattern_count: 0,
            critical_event_count: 0,
            average_cumulative_risk: 0.0,
            latest_peak_risk: 0.0,
            observation_confidence: seed_confidence,
            risk_direction: "flat".to_string(),
            narrative,
            provenance,
        })
    }

    fn build_narrative_provenance(
        &self,
        site_id: &str,
        window_start_unix_seconds: i64,
        anchor_unix_seconds: i64,
    ) -> Result<NarrativeProvenance, StorageError> {
        let mut history = self.storage.canonical_passive_events(512, Some(site_id))?;
        history.sort_by(|left, right| {
            left.first_seen_at_unix_seconds
                .cmp(&right.first_seen_at_unix_seconds)
                .then_with(|| {
                    left.last_seen_at_unix_seconds
                        .cmp(&right.last_seen_at_unix_seconds)
                })
        });
        let mut events = history
            .iter()
            .filter(|event| event.last_seen_at_unix_seconds >= window_start_unix_seconds)
            .cloned()
            .collect::<Vec<_>>();
        events.sort_by(|left, right| {
            right
                .max_risk_score
                .total_cmp(&left.max_risk_score)
                .then_with(|| right.occurrence_count.cmp(&left.occurrence_count))
                .then_with(|| {
                    right
                        .last_seen_at_unix_seconds
                        .cmp(&left.last_seen_at_unix_seconds)
                })
        });

        let mut bundle_hashes = BTreeSet::new();
        let mut manifest_hashes = BTreeSet::new();
        for event in &events {
            bundle_hashes.extend(event.evidence_bundle_hashes.iter().cloned());
            manifest_hashes.extend(event.replay_manifest_hashes.iter().cloned());
        }

        let top_drivers = events
            .iter()
            .take(5)
            .map(|event| {
                let projection = crate::canonical_views::project_canonical_event(
                    &history,
                    event,
                    anchor_unix_seconds,
                );
                NarrativeProvenanceDriver {
                    canonical_event_id: event.canonical_id.clone(),
                    event_type: format!("{:?}", event.threat_type),
                    status: format!("{:?}", projection.status),
                    risk_score: event.max_risk_score,
                    confidence: event.avg_confidence,
                    support_count: usize::try_from(event.occurrence_count).unwrap_or(usize::MAX),
                    summary: event.summary.clone(),
                    bundle_hashes: event.evidence_bundle_hashes.clone(),
                    manifest_hashes: event.replay_manifest_hashes.clone(),
                    canonical_event_path: canonical_event_path(&event.canonical_id),
                    evidence_paths: event
                        .evidence_bundle_hashes
                        .iter()
                        .map(|hash| evidence_path(hash))
                        .collect(),
                    replay_paths: event
                        .replay_manifest_hashes
                        .iter()
                        .map(|hash| replay_path(hash))
                        .collect(),
                }
            })
            .collect::<Vec<_>>();

        let canonical_event_ids = events
            .iter()
            .map(|event| event.canonical_id.clone())
            .collect::<Vec<_>>();
        let bundle_hashes = bundle_hashes.into_iter().collect::<Vec<_>>();
        let manifest_hashes = manifest_hashes.into_iter().collect::<Vec<_>>();

        Ok(NarrativeProvenance {
            canonical_event_paths: canonical_event_ids
                .iter()
                .map(|id| canonical_event_path(id))
                .collect(),
            evidence_paths: bundle_hashes
                .iter()
                .map(|hash| evidence_path(hash))
                .collect(),
            replay_paths: manifest_hashes
                .iter()
                .map(|hash| replay_path(hash))
                .collect(),
            canonical_event_ids,
            bundle_hashes,
            manifest_hashes,
            site_overview_path: format!("/v1/passive/sites/{site_id}/overview"),
            site_narrative_path: format!("/v1/passive/sites/{site_id}/narrative?days=30"),
            top_drivers,
        })
    }

    fn seed_record_for_site(
        &self,
        site: &PassiveSiteProfile,
    ) -> Result<Option<PassiveSeedRecord>, StorageError> {
        for source_reference in &site.source_references {
            if let Some(record) = self.storage.passive_seed_record(source_reference)? {
                return Ok(Some(record));
            }
        }
        Ok(None)
    }

    fn store_discovered_site_profiles(
        &self,
        seeds: &[OpenInfraSiteSeed],
    ) -> Result<(), StorageError> {
        let mut profiles_by_site_id = BTreeMap::<String, PassiveSiteProfile>::new();
        for seed in seeds {
            let profile = sss_passive_scanner::sources::infra_mapper::site_profile_from_seed(seed);
            let site_id = profile.site.site_id.to_string();
            if let Some(existing) = profiles_by_site_id.get_mut(&site_id) {
                for source_reference in profile.source_references {
                    if !existing.source_references.contains(&source_reference) {
                        existing.source_references.push(source_reference);
                    }
                }
                for passive_tag in profile.passive_tags {
                    if !existing.passive_tags.contains(&passive_tag) {
                        existing.passive_tags.push(passive_tag);
                    }
                }
                existing.observation_radius_km = existing
                    .observation_radius_km
                    .max(profile.observation_radius_km);
            } else {
                profiles_by_site_id.insert(site_id, profile);
            }
        }
        self.storage
            .store_passive_site_profiles(&profiles_by_site_id.into_values().collect::<Vec<_>>())
    }

    fn store_or_update_seed_lifecycle(
        &self,
        seeds: &[OpenInfraSiteSeed],
        scan: Option<&PassiveScanOutput>,
        observed_at_unix_seconds: i64,
    ) -> Result<(), StorageError> {
        let existing_records = self
            .storage
            .passive_seed_records(10_000)?
            .into_iter()
            .map(|record| (record.seed_key.clone(), record))
            .collect::<std::collections::BTreeMap<_, _>>();
        let mut records = Vec::with_capacity(seeds.len());
        for seed in seeds {
            let existing = existing_records.get(&seed.source_reference);
            records.push(build_seed_record(
                seed,
                existing,
                scan,
                observed_at_unix_seconds,
            ));
        }
        self.storage.store_passive_seed_records(&records)
    }

    async fn fetch_site_weather_now(&self, site: &Site) -> Result<WeatherNowSignal, AppError> {
        let payload: OpenMeteoForecastResponse = self
            .http_client
            .get(format!(
                "{}/v1/forecast",
                self.open_meteo_base_url.trim_end_matches('/')
            ))
            .query(&[
                ("latitude", site.latitude.to_string()),
                ("longitude", site.longitude.to_string()),
                (
                    "current",
                    "wind_speed_10m,precipitation_probability,shortwave_radiation,cloud_cover"
                        .to_string(),
                ),
                ("forecast_days", "1".to_string()),
                ("timezone", "GMT".to_string()),
                ("timeformat", "unixtime".to_string()),
            ])
            .send()
            .await
            .map_err(|error| AppError::SourceUnavailable(error.to_string()))?
            .error_for_status()
            .map_err(|error| AppError::SourceUnavailable(error.to_string()))?
            .json::<OpenMeteoForecastResponse>()
            .await
            .map_err(|error| AppError::SourceUnavailable(error.to_string()))?;
        let current = payload.current.ok_or_else(|| {
            AppError::SourceUnavailable("Open-Meteo current payload missing.".to_string())
        })?;
        let irradiance_ratio = current
            .shortwave_radiation
            .map_or(0.0_f64, |value| 1.0 - (f64::from(value) / 1_000.0));
        let cloud_cover_ratio = current
            .cloud_cover
            .map_or(0.0_f64, |value| f64::from(value) / 100.0);
        Ok(WeatherNowSignal {
            observed_at_unix_seconds: current.time,
            wind_kph: f64::from(current.wind_speed_10m.unwrap_or_default()),
            hail_probability: (f64::from(current.precipitation_probability.unwrap_or_default())
                / 100.0)
                .clamp(0.0, 1.0),
            irradiance_drop_ratio: irradiance_ratio.max(cloud_cover_ratio).clamp(0.0, 1.0),
            cloud_cover_ratio: cloud_cover_ratio.clamp(0.0, 1.0),
        })
    }

    async fn fetch_open_meteo_observation(
        &self,
        site: &OpenInfraSiteSeed,
    ) -> Result<WeatherObservation, AppError> {
        let payload: OpenMeteoForecastResponse = self
            .http_client
            .get(format!(
                "{}/v1/forecast",
                self.open_meteo_base_url.trim_end_matches('/')
            ))
            .query(&[
                ("latitude", site.latitude.to_string()),
                ("longitude", site.longitude.to_string()),
                (
                    "current",
                    "wind_speed_10m,precipitation_probability,shortwave_radiation,cloud_cover"
                        .to_string(),
                ),
                ("forecast_days", "1".to_string()),
                ("timezone", "GMT".to_string()),
                ("timeformat", "unixtime".to_string()),
            ])
            .send()
            .await
            .map_err(|error| AppError::SourceUnavailable(error.to_string()))?
            .error_for_status()
            .map_err(|error| AppError::SourceUnavailable(error.to_string()))?
            .json::<OpenMeteoForecastResponse>()
            .await
            .map_err(|error| AppError::SourceUnavailable(error.to_string()))?;
        let current = payload.current.ok_or_else(|| {
            AppError::SourceUnavailable("Open-Meteo current payload missing.".to_string())
        })?;
        let wind_kph = current.wind_speed_10m.unwrap_or_default();
        let hail_probability =
            (current.precipitation_probability.unwrap_or_default() / 100.0).clamp(0.0, 1.0);
        let irradiance_ratio = current
            .shortwave_radiation
            .map_or(0.0, |value| 1.0 - (value / 1_000.0));
        let cloud_cover_ratio = current.cloud_cover.map_or(0.0, |value| value / 100.0);
        Ok(WeatherObservation {
            observed_at_unix_seconds: current.time,
            latitude: site.latitude,
            longitude: site.longitude,
            wind_kph,
            hail_probability: hail_probability.max(0.05),
            irradiance_drop_ratio: irradiance_ratio.max(cloud_cover_ratio).clamp(0.0, 1.0),
        })
    }

    async fn fetch_firms_observations(
        &self,
        site: &OpenInfraSiteSeed,
    ) -> Result<Vec<FireSmokeObservation>, AppError> {
        let Some(map_key) = self.firms_map_key.as_ref() else {
            return Ok(Vec::new());
        };
        let (west, south, east, north) =
            bounding_box(site.latitude, site.longitude, site.observation_radius_km);
        let url = format!(
            "{}/api/area/csv/{}/{}/{west:.4},{south:.4},{east:.4},{north:.4}/1",
            self.firms_api_base_url.trim_end_matches('/'),
            map_key,
            self.firms_source
        );
        let payload = self
            .http_client
            .get(url)
            .send()
            .await
            .map_err(|error| AppError::SourceUnavailable(error.to_string()))?
            .error_for_status()
            .map_err(|error| AppError::SourceUnavailable(error.to_string()))?
            .text()
            .await
            .map_err(|error| AppError::SourceUnavailable(error.to_string()))?;
        let mut reader = csv::Reader::from_reader(payload.as_bytes());
        let headers = reader
            .headers()
            .map_err(|error| AppError::SourceUnavailable(error.to_string()))?
            .clone();
        let latitude_index = csv_index(&headers, &["latitude", "lat"])?;
        let longitude_index = csv_index(&headers, &["longitude", "lon", "lng"])?;
        let frp_index = csv_index(&headers, &["frp"]).ok();
        let confidence_index = csv_index(&headers, &["confidence"]).ok();

        let mut observations = Vec::new();
        for record in reader.records() {
            let record = record.map_err(|error| AppError::SourceUnavailable(error.to_string()))?;
            let latitude = record
                .get(latitude_index)
                .and_then(|value| value.parse::<f64>().ok());
            let longitude = record
                .get(longitude_index)
                .and_then(|value| value.parse::<f64>().ok());
            let Some(latitude) = latitude else {
                continue;
            };
            let Some(longitude) = longitude else {
                continue;
            };
            let fire_radiative_power = frp_index
                .and_then(|index| record.get(index))
                .and_then(|value| value.parse::<f32>().ok())
                .unwrap_or(12.0);
            let smoke_density_index = confidence_index
                .and_then(|index| record.get(index))
                .and_then(|value| value.parse::<f32>().ok())
                .unwrap_or_else(|| (fire_radiative_power * 1.2).clamp(5.0, 100.0));
            observations.push(FireSmokeObservation {
                observed_at_unix_seconds: now_unix_seconds(),
                latitude,
                longitude,
                fire_radiative_power,
                smoke_density_index,
            });
        }
        Ok(observations)
    }

    async fn fetch_opensky_observations(
        &self,
        site: &OpenInfraSiteSeed,
    ) -> Result<Vec<AdsbFlightObservation>, AppError> {
        let Some(token) = self.opensky_bearer_token.as_ref() else {
            return Ok(Vec::new());
        };
        let (west, south, east, north) =
            bounding_box(site.latitude, site.longitude, site.observation_radius_km);
        let payload: OpenSkyStatesResponse = self
            .http_client
            .get(format!(
                "{}/states/all",
                self.opensky_api_base_url.trim_end_matches('/')
            ))
            .bearer_auth(token)
            .query(&[
                ("lamin", south.to_string()),
                ("lomin", west.to_string()),
                ("lamax", north.to_string()),
                ("lomax", east.to_string()),
            ])
            .send()
            .await
            .map_err(|error| AppError::SourceUnavailable(error.to_string()))?
            .error_for_status()
            .map_err(|error| AppError::SourceUnavailable(error.to_string()))?
            .json::<OpenSkyStatesResponse>()
            .await
            .map_err(|error| AppError::SourceUnavailable(error.to_string()))?;
        let observed_at_unix_seconds = payload.time;
        let rows = payload.states.unwrap_or_default();
        let mut observations = Vec::new();
        for row in rows {
            let flight_id = row
                .first()
                .and_then(serde_json::Value::as_str)
                .filter(|value: &&str| !value.trim().is_empty())
                .map(str::trim)
                .map(str::to_string)
                .or_else(|| {
                    row.get(1)
                        .and_then(serde_json::Value::as_str)
                        .map(str::trim)
                        .filter(|value: &&str| !value.is_empty())
                        .map(str::to_string)
                });
            let longitude = row.get(5).and_then(serde_json::Value::as_f64);
            let latitude = row.get(6).and_then(serde_json::Value::as_f64);
            let Some(flight_id) = flight_id else {
                continue;
            };
            let Some(longitude) = longitude else {
                continue;
            };
            let Some(latitude) = latitude else {
                continue;
            };
            let altitude_m = row
                .get(7)
                .and_then(serde_json::Value::as_f64)
                .map_or(0.0, f64_to_f32);
            let speed_kts = row
                .get(9)
                .and_then(serde_json::Value::as_f64)
                .map(|value| value * 1.943_844)
                .map_or(0.0, f64_to_f32);
            let vertical_rate_m_s = row
                .get(11)
                .and_then(serde_json::Value::as_f64)
                .map_or(0.0, f64_to_f32);
            observations.push(AdsbFlightObservation {
                flight_id,
                observed_at_unix_seconds,
                latitude,
                longitude,
                altitude_m,
                speed_kts,
                vertical_rate_m_s,
            });
        }
        Ok(observations)
    }
}

fn ranked_event_matches_filter(
    event: &sss_core::RankedEvent,
    filter: &RankedEventsFilter,
    now_unix_seconds: i64,
) -> bool {
    if let Some(object_id) = filter.object_id.as_ref() {
        if &event.object_id != object_id {
            return false;
        }
    }

    if let Some(event_type) = filter.event_type.as_ref() {
        if &event.event_type != event_type {
            return false;
        }
    }

    if filter.future_only
        && event
            .target_epoch_unix_seconds
            .is_none_or(|target_epoch| target_epoch < now_unix_seconds)
    {
        return false;
    }

    if let Some(after_unix_seconds) = filter.after_unix_seconds {
        if event
            .target_epoch_unix_seconds
            .is_none_or(|target_epoch| target_epoch < after_unix_seconds)
        {
            return false;
        }
    }

    if let Some(before_unix_seconds) = filter.before_unix_seconds {
        if event
            .target_epoch_unix_seconds
            .is_none_or(|target_epoch| target_epoch > before_unix_seconds)
        {
            return false;
        }
    }

    true
}

fn compare_timeline_events(
    left: &sss_core::RankedEvent,
    right: &sss_core::RankedEvent,
) -> Ordering {
    match (
        left.target_epoch_unix_seconds,
        right.target_epoch_unix_seconds,
    ) {
        (Some(left_epoch), Some(right_epoch)) => left_epoch
            .cmp(&right_epoch)
            .then_with(|| right.risk.total_cmp(&left.risk))
            .then_with(|| left.event_id.cmp(&right.event_id)),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => right
            .risk
            .total_cmp(&left.risk)
            .then_with(|| left.event_id.cmp(&right.event_id)),
    }
}

fn prediction_snapshot_from_analysis(
    request_id: &str,
    response: &AnalyzeObjectResponse,
) -> PredictionSnapshot {
    PredictionSnapshot {
        snapshot_id: format!("{request_id}:analysis:{}", response.object_id),
        request_id: request_id.to_string(),
        endpoint: "/v1/analyze-object".to_string(),
        object_id: response.object_id.clone(),
        generated_at_unix_seconds: now_unix_seconds(),
        horizon_hours: response.prediction.horizon_hours,
        base_epoch_unix_seconds: response.prediction.based_on_observation_epoch,
        propagation_model: response.prediction.propagation_model.clone(),
        model_version: response.assessment.model_version.clone(),
        evidence_bundle_hash: Some(response.evidence_bundle.bundle_hash.clone()),
        predictions: vec![response.prediction.clone()],
    }
}

fn prediction_snapshot_from_predict_response(
    request_id: &str,
    response: &sss_core::PredictResponse,
) -> PredictionSnapshot {
    let primary = response.predictions.first().cloned();
    PredictionSnapshot {
        snapshot_id: format!(
            "{request_id}:predict:{}:{}",
            response.object_id,
            response.predictions.len()
        ),
        request_id: request_id.to_string(),
        endpoint: "/v1/predict".to_string(),
        object_id: response.object_id.clone(),
        generated_at_unix_seconds: now_unix_seconds(),
        horizon_hours: primary
            .as_ref()
            .map_or(0.0, |prediction| prediction.horizon_hours),
        base_epoch_unix_seconds: primary
            .as_ref()
            .and_then(|prediction| prediction.based_on_observation_epoch),
        propagation_model: primary.as_ref().map_or_else(
            || "none".to_string(),
            |prediction| prediction.propagation_model.clone(),
        ),
        model_version: "sss-behavioral-mvp-v1".to_string(),
        evidence_bundle_hash: None,
        predictions: response.predictions.clone(),
    }
}

fn forecast_prediction_snapshot(
    request_id: &str,
    endpoint: &str,
    site_id: &str,
    horizon_hours: u32,
    explanation: &str,
    confidence: f64,
) -> PredictionSnapshot {
    PredictionSnapshot {
        snapshot_id: format!("{request_id}:forecast:{site_id}:{endpoint}"),
        request_id: request_id.to_string(),
        endpoint: endpoint.to_string(),
        object_id: site_id.to_string(),
        generated_at_unix_seconds: now_unix_seconds(),
        horizon_hours: f64::from(horizon_hours),
        base_epoch_unix_seconds: Some(now_unix_seconds()),
        propagation_model: "sss-forecast-modeled-v1".to_string(),
        model_version: "sss-forecast-v1".to_string(),
        evidence_bundle_hash: None,
        predictions: vec![Prediction {
            object_id: site_id.to_string(),
            horizon_hours: f64::from(horizon_hours),
            event: PredictedEvent::ContinuedAnomalousBehavior,
            probability: confidence.clamp(0.0, 0.99),
            explanation: explanation.to_string(),
            propagation_model: "sss-forecast-modeled-v1".to_string(),
            based_on_observation_epoch: Some(now_unix_seconds()),
            predicted_state: None,
            position_delta_km: None,
            velocity_delta_km_s: None,
        }],
    }
}

fn timeline_bucket_specs(horizon_hours: f64) -> Vec<(&'static str, f64, f64)> {
    let mut buckets = Vec::new();
    let capped_horizon = horizon_hours.max(1.0);
    let near_term_end = capped_horizon.min(6.0);
    buckets.push(("0-6h", 0.0, near_term_end));

    if capped_horizon > 6.0 {
        buckets.push(("6-24h", 6.0, capped_horizon.min(24.0)));
    }
    if capped_horizon > 24.0 {
        buckets.push(("24h+", 24.0, capped_horizon));
    }

    buckets
}

fn rounded_seconds_from_hours(hours: f64) -> i64 {
    Duration::from_secs_f64((hours.max(0.0) * 3_600.0).round())
        .as_secs()
        .try_into()
        .unwrap_or(i64::MAX)
}

fn validate_neows_date_range(start_date: &str, end_date: &str) -> Result<(), AppError> {
    if !is_iso_date(start_date) || !is_iso_date(end_date) {
        return Err(AppError::InvalidDateRange(
            "NeoWS dates must use YYYY-MM-DD".to_string(),
        ));
    }
    if start_date > end_date {
        return Err(AppError::InvalidDateRange(
            "NeoWS start_date must be before or equal to end_date".to_string(),
        ));
    }
    Ok(())
}

fn is_iso_date(value: &str) -> bool {
    value.len() == 10
        && value.chars().enumerate().all(|(index, ch)| match index {
            4 | 7 => ch == '-',
            _ => ch.is_ascii_digit(),
        })
}

fn build_overpass_query(request: &PassiveInfraDiscoverRequest) -> String {
    let bbox = format!(
        "({:.6},{:.6},{:.6},{:.6})",
        request.south, request.west, request.north, request.east
    );
    let site_types = requested_site_types(request);
    let mut selectors = Vec::new();

    if site_types.contains(&SiteType::SolarPlant) {
        selectors.push(format!(
            r#"nwr["power"="plant"]["plant:source"="solar"]{bbox};"#
        ));
        selectors.push(format!(
            r#"nwr["power"="generator"]["generator:source"="solar"]["generator:method"="photovoltaic"]{bbox};"#
        ));
    }
    if site_types.contains(&SiteType::Substation) {
        selectors.push(format!(r#"nwr["power"="substation"]{bbox};"#));
    }
    if site_types.contains(&SiteType::DataCenter) {
        selectors.push(format!(r#"nwr["telecom"="data_center"]{bbox};"#));
        selectors.push(format!(r#"nwr["building"="data_center"]{bbox};"#));
        selectors.push(format!(r#"nwr["building"="data_centre"]{bbox};"#));
    }

    format!(
        "[out:json][timeout:25];({});out center;",
        selectors.join("")
    )
}

fn requested_site_types(request: &PassiveInfraDiscoverRequest) -> Vec<SiteType> {
    let site_types = request.site_types.clone().unwrap_or_else(|| {
        vec![
            SiteType::SolarPlant,
            SiteType::DataCenter,
            SiteType::Substation,
        ]
    });
    let mut unique = Vec::new();
    for site_type in site_types {
        if !unique.contains(&site_type) {
            unique.push(site_type);
        }
    }
    unique
}

fn overpass_elements_to_seeds(
    request: &PassiveInfraDiscoverRequest,
    elements: Vec<OverpassElement>,
) -> Vec<OpenInfraSiteSeed> {
    let timezone = request
        .timezone
        .clone()
        .unwrap_or_else(|| "UTC".to_string());
    let country_code = request
        .country_code
        .clone()
        .unwrap_or_else(|| "XX".to_string());
    let default_operator_name = request
        .default_operator_name
        .clone()
        .unwrap_or_else(|| "OpenStreetMap".to_string());
    let observation_radius_km = request
        .observation_radius_km
        .unwrap_or(15.0)
        .clamp(1.0, 75.0);
    let default_criticality = request.default_criticality.unwrap_or(Criticality::High);
    let mut seen = std::collections::BTreeSet::new();
    let mut seeds = Vec::new();

    for element in elements {
        let Some(tags) = element.tags.as_ref() else {
            continue;
        };
        let Some(site_type) = overpass_site_type(tags) else {
            continue;
        };
        if matches!(site_type, SiteType::SolarPlant) && should_skip_small_rooftop_solar(tags) {
            continue;
        }
        let (latitude, longitude) = if let (Some(lat), Some(lon)) = (element.lat, element.lon) {
            (lat, lon)
        } else if let Some(center) = element.center.as_ref() {
            (center.lat, center.lon)
        } else {
            continue;
        };
        let name = overpass_name(tags, site_type, element.id);
        let operator_name = tags
            .get("operator")
            .cloned()
            .unwrap_or_else(|| default_operator_name.clone());
        let source_reference = format!("osm:{}:{}", element.element_type, element.id);
        let dedupe_key = format!(
            "{site_type:?}:{}:{}:{}:{}",
            name.to_lowercase(),
            rounded_coord_key(latitude),
            rounded_coord_key(longitude),
            source_reference
        );
        if !seen.insert(dedupe_key) {
            continue;
        }

        seeds.push(OpenInfraSiteSeed {
            name,
            site_type,
            latitude,
            longitude,
            elevation_m: 0.0,
            timezone: timezone.clone(),
            country_code: country_code.clone(),
            criticality: inferred_criticality(site_type, default_criticality, tags),
            operator_name,
            observation_radius_km,
            source_reference,
        });
    }

    seeds.sort_by(|left, right| {
        format!("{:?}", left.site_type)
            .cmp(&format!("{:?}", right.site_type))
            .then_with(|| left.name.cmp(&right.name))
    });
    seeds
}

fn overpass_site_type(tags: &std::collections::BTreeMap<String, String>) -> Option<SiteType> {
    if matches!(tags.get("power").map(String::as_str), Some("substation")) {
        return Some(SiteType::Substation);
    }
    if matches!(tags.get("plant:source").map(String::as_str), Some("solar"))
        || matches!(
            tags.get("generator:source").map(String::as_str),
            Some("solar")
        )
    {
        return Some(SiteType::SolarPlant);
    }
    if matches!(tags.get("telecom").map(String::as_str), Some("data_center"))
        || matches!(
            tags.get("building").map(String::as_str),
            Some("data_center" | "data_centre")
        )
    {
        return Some(SiteType::DataCenter);
    }
    None
}

fn should_skip_small_rooftop_solar(tags: &std::collections::BTreeMap<String, String>) -> bool {
    matches!(tags.get("location").map(String::as_str), Some("roof"))
        || matches!(
            tags.get("generator:place").map(String::as_str),
            Some("roof")
        )
        || matches!(
            tags.get("generator:output:electricity").map(String::as_str),
            Some("small_installation")
        )
}

fn overpass_name(
    tags: &std::collections::BTreeMap<String, String>,
    site_type: SiteType,
    id: i64,
) -> String {
    tags.get("name")
        .cloned()
        .unwrap_or_else(|| match site_type {
            SiteType::SolarPlant => format!("OSM Solar Site {id}"),
            SiteType::DataCenter => format!("OSM Data Center {id}"),
            SiteType::Substation => format!("OSM Substation {id}"),
        })
}

fn inferred_criticality(
    site_type: SiteType,
    default_criticality: Criticality,
    tags: &std::collections::BTreeMap<String, String>,
) -> Criticality {
    if tags.contains_key("voltage") || tags.contains_key("plant:output:electricity") {
        return Criticality::Critical;
    }
    match site_type {
        SiteType::DataCenter | SiteType::Substation => Criticality::High,
        SiteType::SolarPlant => default_criticality,
    }
}

fn rounded_coord_key(value: f64) -> String {
    format!("{value:.4}")
}

fn build_seed_record(
    seed: &OpenInfraSiteSeed,
    existing: Option<&PassiveSeedRecord>,
    scan: Option<&PassiveScanOutput>,
    observed_at_unix_seconds: i64,
) -> PassiveSeedRecord {
    let matched_site = scan.and_then(|output| {
        output
            .sites
            .iter()
            .find(|site| site.source_references.contains(&seed.source_reference))
    });
    let site_id = matched_site
        .map(|site| site.site.site_id.to_string())
        .or_else(|| existing.and_then(|record| record.site_id.as_ref()).cloned())
        .or_else(|| {
            Some(
                sss_passive_scanner::sources::infra_mapper::site_profile_from_seed(seed)
                    .site
                    .site_id
                    .to_string(),
            )
        });
    let site_events = site_id.as_ref().map_or_else(Vec::new, |site_id| {
        scan.map_or_else(Vec::new, |output| {
            output
                .events
                .iter()
                .filter(|event| event.site_id.to_string() == *site_id)
                .cloned()
                .collect::<Vec<_>>()
        })
    });
    let latest_risk = site_id.as_ref().and_then(|site_id| {
        scan.and_then(|output| {
            output
                .risk_history
                .iter()
                .filter(|record| record.site_id.to_string() == *site_id)
                .max_by(|left, right| {
                    left.window_end_unix_seconds
                        .cmp(&right.window_end_unix_seconds)
                })
                .cloned()
        })
    });
    let source_count = {
        let distinct_sources = site_events
            .iter()
            .map(|event| event.source_kind)
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        u32::try_from(distinct_sources.saturating_add(1)).unwrap_or(u32::MAX)
    };
    let last_event_at_unix_seconds = site_events
        .iter()
        .map(|event| event.observed_at_unix_seconds)
        .max()
        .or(existing.and_then(|record| record.last_event_at_unix_seconds));
    let last_scanned_at_unix_seconds = scan
        .map(|_| observed_at_unix_seconds)
        .or(existing.and_then(|record| record.last_scanned_at_unix_seconds));
    let confidence = seed_observation_confidence(
        seed,
        source_count,
        site_events.len(),
        latest_risk.as_ref().map(|record| record.peak_risk),
        existing.map_or(0.0, |record| record.confidence),
    );
    let scan_priority = seed_scan_priority(
        seed,
        latest_risk.as_ref().map(|record| record.peak_risk),
        site_events.len(),
        existing.map_or(0.0, |record| record.scan_priority),
    );
    let classification_status = classify_seed(
        scan.is_some(),
        site_events.len(),
        latest_risk.as_ref().map_or(0.0, |record| record.peak_risk),
    );
    let seed_status = seed_status(
        scan.is_some(),
        last_event_at_unix_seconds,
        latest_risk.as_ref().map(|record| record.peak_risk),
        observed_at_unix_seconds,
    );

    PassiveSeedRecord {
        seed_key: seed.source_reference.clone(),
        site_id,
        seed: seed.clone(),
        discovered_at_unix_seconds: existing.map_or(observed_at_unix_seconds, |record| {
            record.discovered_at_unix_seconds
        }),
        last_scanned_at_unix_seconds,
        last_event_at_unix_seconds,
        confidence,
        source_count: source_count.max(existing.map_or(0, |record| record.source_count)),
        classification_status,
        seed_status,
        scan_priority,
    }
}

fn seed_observation_confidence(
    seed: &OpenInfraSiteSeed,
    source_count: u32,
    event_count: usize,
    peak_risk: Option<f64>,
    previous_confidence: f64,
) -> f64 {
    let base: f64 = match seed.site_type {
        SiteType::Substation => 0.68,
        SiteType::DataCenter => 0.63,
        SiteType::SolarPlant => 0.58,
    };
    let criticality_bonus = match seed.criticality {
        Criticality::Low => 0.02,
        Criticality::Medium => 0.05,
        Criticality::High => 0.09,
        Criticality::Critical => 0.14,
    };
    let source_bonus = f64::from(source_count.saturating_sub(1)).min(4.0) * 0.04;
    let event_bonus = (f64::from(u32::try_from(event_count).unwrap_or(u32::MAX)) * 0.03).min(0.12);
    let risk_bonus = peak_risk.unwrap_or(0.0) * 0.12;
    (base.max(previous_confidence * 0.9)
        + criticality_bonus
        + source_bonus
        + event_bonus
        + risk_bonus)
        .clamp(0.0, 0.99)
}

fn seed_scan_priority(
    seed: &OpenInfraSiteSeed,
    peak_risk: Option<f64>,
    event_count: usize,
    previous_priority: f64,
) -> f64 {
    let site_weight = match seed.site_type {
        SiteType::Substation => 0.52,
        SiteType::DataCenter => 0.46,
        SiteType::SolarPlant => 0.38,
    };
    let criticality_weight = match seed.criticality {
        Criticality::Low => 0.05,
        Criticality::Medium => 0.10,
        Criticality::High => 0.16,
        Criticality::Critical => 0.22,
    };
    let event_weight = (f64::from(u32::try_from(event_count).unwrap_or(u32::MAX)) * 0.05).min(0.20);
    (site_weight + criticality_weight + peak_risk.unwrap_or(0.0) * 0.32 + event_weight)
        .max(previous_priority * 0.85)
        .clamp(0.0, 1.0)
}

fn classify_seed(
    scanned: bool,
    event_count: usize,
    peak_risk: f64,
) -> PassiveSeedClassificationStatus {
    if peak_risk >= 0.85 {
        PassiveSeedClassificationStatus::Strategic
    } else if peak_risk >= 0.55 || event_count >= 2 {
        PassiveSeedClassificationStatus::Elevated
    } else if scanned {
        PassiveSeedClassificationStatus::Observed
    } else {
        PassiveSeedClassificationStatus::Discovered
    }
}

fn seed_status(
    scanned: bool,
    last_event_at_unix_seconds: Option<i64>,
    peak_risk: Option<f64>,
    observed_at_unix_seconds: i64,
) -> PassiveSeedStatus {
    if peak_risk.unwrap_or(0.0) >= 0.8
        || last_event_at_unix_seconds == Some(observed_at_unix_seconds)
    {
        PassiveSeedStatus::Elevated
    } else if scanned {
        PassiveSeedStatus::Monitoring
    } else {
        PassiveSeedStatus::New
    }
}

fn build_execution_summary(
    site_overviews: &[PassiveSiteOverview],
    scan: &PassiveScanOutput,
) -> PassiveExecutionSummary {
    let highest_scan_priority = site_overviews
        .iter()
        .filter_map(|overview| {
            overview
                .seed_lifecycle
                .as_ref()
                .map(|record| record.scan_priority)
        })
        .fold(0.0, f64::max);
    let elevated_seed_count = site_overviews
        .iter()
        .filter(|overview| {
            overview.seed_lifecycle.as_ref().is_some_and(|record| {
                matches!(
                    record.classification_status,
                    PassiveSeedClassificationStatus::Elevated
                        | PassiveSeedClassificationStatus::Strategic
                )
            })
        })
        .count();
    let critical_event_count = scan
        .events
        .iter()
        .filter(|event| event.risk_score >= 0.8)
        .count();

    PassiveExecutionSummary {
        discovered_seed_count: site_overviews.len(),
        scanned_seed_count: scan.sites.len(),
        event_count: scan.events.len(),
        elevated_seed_count,
        critical_event_count,
        highest_scan_priority,
    }
}

fn passive_scan_cadence_seconds(record: &PassiveSeedRecord) -> i64 {
    if matches!(
        record.classification_status,
        PassiveSeedClassificationStatus::Strategic | PassiveSeedClassificationStatus::Elevated
    ) || matches!(record.seed_status, PassiveSeedStatus::Elevated)
    {
        return 3_600;
    }

    match (record.seed.site_type, record.seed.criticality) {
        (SiteType::Substation, Criticality::Critical) => 7_200,
        (SiteType::Substation, _)
        | (SiteType::DataCenter, Criticality::Critical | Criticality::High) => 10_800,
        (SiteType::DataCenter, _) | (SiteType::SolarPlant, Criticality::Critical) => 14_400,
        (SiteType::SolarPlant, Criticality::High) => 21_600,
        (SiteType::SolarPlant, _) => 43_200,
    }
}

fn passive_scan_overdue_seconds(
    record: &PassiveSeedRecord,
    now_unix_seconds: i64,
    cadence_seconds: i64,
) -> i64 {
    record
        .last_scanned_at_unix_seconds
        .map_or(cadence_seconds, |last_scanned| {
            now_unix_seconds
                .saturating_sub(last_scanned)
                .saturating_sub(cadence_seconds)
        })
}

fn passive_schedule_reason(
    record: &PassiveSeedRecord,
    force: bool,
    overdue_seconds: i64,
) -> String {
    if force {
        return "force requested by scheduler caller".to_string();
    }
    if record.last_scanned_at_unix_seconds.is_none() {
        return "seed has never been scanned".to_string();
    }
    if matches!(
        record.classification_status,
        PassiveSeedClassificationStatus::Strategic | PassiveSeedClassificationStatus::Elevated
    ) {
        return format!(
            "seed is {:?} and cadence is overdue by {} seconds",
            record.classification_status,
            overdue_seconds.max(0)
        );
    }
    format!("cadence overdue by {} seconds", overdue_seconds.max(0))
}

fn passive_region_discovery_request(target: &PassiveRegionTarget) -> PassiveInfraDiscoverRequest {
    PassiveInfraDiscoverRequest {
        south: target.south,
        west: target.west,
        north: target.north,
        east: target.east,
        site_types: target.site_types.clone(),
        timezone: target.timezone.clone(),
        country_code: target.country_code.clone(),
        default_operator_name: target.default_operator_name.clone(),
        default_criticality: target.default_criticality,
        observation_radius_km: target.observation_radius_km,
    }
}

fn passive_region_scheduler_request(
    request: &PassiveRegionRunRequest,
    scan_limit: usize,
    minimum_priority: Option<f64>,
    dry_run: bool,
) -> PassiveScheduleScanRequest {
    PassiveScheduleScanRequest {
        limit: Some(scan_limit),
        window_hours: request.window_hours,
        include_adsb: request.include_adsb,
        include_weather: request.include_weather,
        include_fire_smoke: request.include_fire_smoke,
        minimum_priority,
        force: None,
        dry_run: Some(dry_run),
    }
}

fn passive_region_run_status(
    request: &PassiveRegionRunRequest,
    response: &PassiveRegionRunResponse,
) -> PassiveRegionRunStatus {
    if request.dry_run.unwrap_or(false) {
        PassiveRegionRunStatus::DryRun
    } else if response.discovered_region_count == 0 {
        PassiveRegionRunStatus::CompletedWithNoDueRegions
    } else {
        PassiveRegionRunStatus::Completed
    }
}

fn passive_region_lease_diagnostic(
    lease: &PassiveRegionLease,
    now_unix_seconds: i64,
) -> PassiveRegionLeaseDiagnostic {
    PassiveRegionLeaseDiagnostic {
        region_id: lease.region_id.clone(),
        worker_id: lease.worker_id.clone(),
        run_id: lease.run_id.clone(),
        acquired_at_unix_seconds: lease.acquired_at_unix_seconds,
        heartbeat_at_unix_seconds: lease.heartbeat_at_unix_seconds,
        expires_at_unix_seconds: lease.expires_at_unix_seconds,
        heartbeat_lag_seconds: now_unix_seconds.saturating_sub(lease.heartbeat_at_unix_seconds),
        expires_in_seconds: lease
            .expires_at_unix_seconds
            .saturating_sub(now_unix_seconds),
        stale: lease.expires_at_unix_seconds <= now_unix_seconds,
    }
}

fn passive_worker_region_metrics(
    run_logs: &[PassiveRegionRunLog],
    leases: &[PassiveRegionLease],
    heartbeats: &[PassiveWorkerHeartbeat],
    now_unix_seconds: i64,
    stale_cutoff_unix_seconds: i64,
    limit: usize,
) -> Vec<PassiveWorkerRegionMetrics> {
    let mut metrics = BTreeMap::<String, PassiveWorkerRegionMetricsAccumulator>::new();

    for run in run_logs {
        for region_id in &run.region_ids {
            let entry = metrics.entry(region_id.clone()).or_default();
            entry.run_count += 1;
            match run.status {
                PassiveRegionRunStatus::Completed
                | PassiveRegionRunStatus::CompletedWithNoDueRegions
                | PassiveRegionRunStatus::DryRun => entry.completed_run_count += 1,
                PassiveRegionRunStatus::Partial => entry.partial_run_count += 1,
                PassiveRegionRunStatus::Failed => entry.failed_run_count += 1,
            }
            entry.discovered_seed_count += run.discovered_seed_count;
            entry.selected_seed_count += run.selected_seed_count;
            entry.event_count += run.event_count;
            entry.source_error_count += run.source_errors.len();
            if entry
                .last_finished_at_unix_seconds
                .is_none_or(|current| run.finished_at_unix_seconds > current)
            {
                entry.last_finished_at_unix_seconds = Some(run.finished_at_unix_seconds);
                entry.latest_run_status = Some(format!("{:?}", run.status));
            }
        }
    }

    for lease in leases {
        let entry = metrics.entry(lease.region_id.clone()).or_default();
        if lease.expires_at_unix_seconds > now_unix_seconds {
            entry.active_lease_count += 1;
        } else {
            entry.stale_lease_count += 1;
        }
    }

    for heartbeat in heartbeats {
        let Some(region_id) = heartbeat.current_region_id.as_ref() else {
            continue;
        };
        let entry = metrics.entry(region_id.clone()).or_default();
        if heartbeat.last_heartbeat_unix_seconds >= stale_cutoff_unix_seconds {
            entry.active_worker_count += 1;
        } else {
            entry.stale_worker_count += 1;
        }
    }

    let mut metrics = metrics
        .into_iter()
        .map(|(region_id, accumulator)| PassiveWorkerRegionMetrics {
            region_id,
            run_count: accumulator.run_count,
            completed_run_count: accumulator.completed_run_count,
            partial_run_count: accumulator.partial_run_count,
            failed_run_count: accumulator.failed_run_count,
            discovered_seed_count: accumulator.discovered_seed_count,
            selected_seed_count: accumulator.selected_seed_count,
            event_count: accumulator.event_count,
            source_error_count: accumulator.source_error_count,
            active_lease_count: accumulator.active_lease_count,
            stale_lease_count: accumulator.stale_lease_count,
            active_worker_count: accumulator.active_worker_count,
            stale_worker_count: accumulator.stale_worker_count,
            latest_run_status: accumulator.latest_run_status,
            last_finished_at_unix_seconds: accumulator.last_finished_at_unix_seconds,
        })
        .collect::<Vec<_>>();

    metrics.sort_by(|left, right| {
        right
            .failed_run_count
            .cmp(&left.failed_run_count)
            .then_with(|| right.stale_lease_count.cmp(&left.stale_lease_count))
            .then_with(|| right.event_count.cmp(&left.event_count))
            .then_with(|| {
                right
                    .last_finished_at_unix_seconds
                    .cmp(&left.last_finished_at_unix_seconds)
            })
            .then_with(|| left.region_id.cmp(&right.region_id))
    });
    metrics.truncate(limit);
    metrics
}

fn passive_worker_source_metrics(
    samples: &[PassiveSourceHealthSample],
    region_id: Option<&str>,
    limit: usize,
) -> Vec<PassiveWorkerSourceMetrics> {
    let now = now_unix_seconds();
    let mut metrics =
        BTreeMap::<(Option<String>, String), PassiveWorkerSourceMetricsAccumulator>::new();

    for sample in samples.iter().filter(|sample| {
        region_id.is_none_or(|expected| sample.region_id.as_deref() == Some(expected))
    }) {
        let key = (
            sample.region_id.clone(),
            format!("{:?}", sample.source_kind),
        );
        let entry = metrics.entry(key).or_default();
        if entry.sample_count == 0 {
            entry.failure_streak_open = true;
        }
        entry.sample_count += 1;
        if sample.fetched {
            entry.success_count += 1;
            entry.failure_streak_open = false;
        } else {
            entry.failure_count += 1;
            if entry.failure_streak_open {
                entry.consecutive_failure_count += 1;
            }
            if entry.last_error.is_none() {
                entry.last_error = Some(sample.detail.clone());
            }
        }
        if entry
            .latest_generated_at_unix_seconds
            .is_none_or(|current| sample.generated_at_unix_seconds > current)
        {
            entry.latest_generated_at_unix_seconds = Some(sample.generated_at_unix_seconds);
        }
    }

    let mut metrics = metrics
        .into_iter()
        .map(|((region_id, source), accumulator)| {
            let total = accumulator
                .success_count
                .saturating_add(accumulator.failure_count);
            let success_rate = if total == 0 {
                1.0
            } else {
                let success = u32::try_from(accumulator.success_count).unwrap_or(u32::MAX);
                let total = u32::try_from(total).unwrap_or(u32::MAX);
                f64::from(success) / f64::from(total)
            };
            let staleness_seconds = accumulator
                .latest_generated_at_unix_seconds
                .map(|latest| now.saturating_sub(latest));
            let reliability_score = source_reliability_score(success_rate, staleness_seconds);
            let health_status =
                source_health_status(success_rate, accumulator.failure_count, staleness_seconds);
            PassiveWorkerSourceMetrics {
                source,
                region_id,
                sample_count: accumulator.sample_count,
                success_count: accumulator.success_count,
                failure_count: accumulator.failure_count,
                consecutive_failure_count: accumulator.consecutive_failure_count,
                success_rate,
                reliability_score,
                health_status,
                latest_generated_at_unix_seconds: accumulator.latest_generated_at_unix_seconds,
                staleness_seconds,
                last_error: accumulator.last_error,
                recovery_hint: source_recovery_hint(
                    health_status,
                    accumulator.consecutive_failure_count,
                    staleness_seconds,
                ),
            }
        })
        .collect::<Vec<_>>();

    metrics.sort_by(|left, right| {
        right
            .failure_count
            .cmp(&left.failure_count)
            .then_with(|| left.success_rate.total_cmp(&right.success_rate))
            .then_with(|| {
                right
                    .latest_generated_at_unix_seconds
                    .cmp(&left.latest_generated_at_unix_seconds)
            })
            .then_with(|| left.source.cmp(&right.source))
            .then_with(|| left.region_id.cmp(&right.region_id))
    });
    metrics.truncate(limit);
    metrics
}

fn source_reliability_score(success_rate: f64, staleness_seconds: Option<i64>) -> f64 {
    let recency_factor = match staleness_seconds {
        Some(seconds) if seconds > 21_600 => 0.5,
        Some(seconds) if seconds > 3_600 => 0.8,
        Some(_) => 1.0,
        None => 0.5,
    };
    (success_rate * recency_factor).clamp(0.0, 1.0)
}

fn source_health_status(
    success_rate: f64,
    failure_count: usize,
    staleness_seconds: Option<i64>,
) -> PassiveSourceHealthStatus {
    if staleness_seconds.is_none_or(|seconds| seconds > 21_600) {
        PassiveSourceHealthStatus::Stale
    } else if failure_count > 0 || success_rate < 0.8 {
        PassiveSourceHealthStatus::Degraded
    } else if success_rate < 0.95 {
        PassiveSourceHealthStatus::Watch
    } else {
        PassiveSourceHealthStatus::Healthy
    }
}

fn readiness_from_samples<'a>(
    source_id: &str,
    label: &str,
    configured: bool,
    enabled: bool,
    samples: impl Iterator<Item = &'a PassiveSourceHealthSample>,
    missing_config_reason: &str,
) -> PassiveSourceReadiness {
    let collected = samples.collect::<Vec<_>>();
    let latest_sample = collected
        .iter()
        .max_by_key(|sample| sample.generated_at_unix_seconds)
        .copied();
    let latest_error = collected
        .iter()
        .map(|sample| sample.detail.clone())
        .find(|detail| !detail.trim().is_empty());
    let latest_staleness_seconds = latest_sample
        .map(|sample| now_unix_seconds().saturating_sub(sample.generated_at_unix_seconds));

    let readiness = if !enabled || !configured {
        PassiveSourceReadinessLevel::NeedsConfig
    } else if let Some(sample) = latest_sample {
        if !sample.fetched || latest_staleness_seconds.is_none_or(|seconds| seconds > 21_600) {
            PassiveSourceReadinessLevel::Degraded
        } else {
            PassiveSourceReadinessLevel::Ready
        }
    } else {
        PassiveSourceReadinessLevel::AwaitingData
    };

    let reason = if !enabled || !configured {
        missing_config_reason.to_string()
    } else if let Some(sample) = latest_sample {
        if sample.detail.trim().is_empty() {
            "Source is configured and has recent passive samples.".to_string()
        } else {
            sample.detail.clone()
        }
    } else {
        "Source is configured but no passive samples have been recorded yet.".to_string()
    };

    PassiveSourceReadiness {
        source_id: source_id.to_string(),
        label: label.to_string(),
        configured,
        enabled,
        readiness,
        reason,
        latest_sample_at_unix_seconds: latest_sample.map(|sample| sample.generated_at_unix_seconds),
        sample_count: collected.len(),
        last_error: latest_error,
    }
}

fn source_recovery_hint(
    health_status: PassiveSourceHealthStatus,
    consecutive_failure_count: usize,
    staleness_seconds: Option<i64>,
) -> String {
    if staleness_seconds.is_none_or(|seconds| seconds > 21_600) {
        return "No recent source sample; run a live scan or check scheduler cadence.".to_string();
    }
    if consecutive_failure_count >= 3 {
        return format!(
            "{consecutive_failure_count} consecutive failures; check credentials, quota, and upstream timeout."
        );
    }
    if consecutive_failure_count > 0 {
        return format!(
            "{consecutive_failure_count} latest failure; retry with backoff before escalating source health."
        );
    }
    match health_status {
        PassiveSourceHealthStatus::Healthy => {
            "Source is healthy; keep current cadence and retention.".to_string()
        }
        PassiveSourceHealthStatus::Watch => {
            "Source is usable but below target reliability; monitor next worker cycle.".to_string()
        }
        PassiveSourceHealthStatus::Degraded => {
            "Source is degraded; inspect recent samples and consider alternate feed fallback."
                .to_string()
        }
        PassiveSourceHealthStatus::Stale => {
            "Source is stale; run a fresh scan before trusting derived risk.".to_string()
        }
    }
}

fn passive_operations_recommendation(
    enabled_region_count: usize,
    due_region_count: usize,
    seed_count: usize,
    observed_seed_count: usize,
    elevated_seed_count: usize,
) -> String {
    if enabled_region_count == 0 {
        "No passive regions configured; create a regional watchlist before scheduling scans."
            .to_string()
    } else if due_region_count > 0 {
        format!("{due_region_count} passive regions are due; run the passive region scheduler.")
    } else if seed_count == 0 {
        "Passive regions are configured, but no seeds have been discovered yet.".to_string()
    } else if observed_seed_count == 0 {
        "Seeds exist but have not been scanned; execute a passive scheduler run.".to_string()
    } else if elevated_seed_count > 0 {
        format!("{elevated_seed_count} elevated seeds need operator review.")
    } else {
        "Passive observation is healthy; keep scheduler cadence running.".to_string()
    }
}

fn passive_seed_matches_region(record: &PassiveSeedRecord, region: &PassiveRegionTarget) -> bool {
    let seed = &record.seed;
    let within_bbox = seed.latitude >= region.south
        && seed.latitude <= region.north
        && seed.longitude >= region.west
        && seed.longitude <= region.east;
    let matches_type = region
        .site_types
        .as_ref()
        .is_none_or(|site_types| site_types.contains(&seed.site_type));
    within_bbox && matches_type
}

fn passive_seed_is_elevated(record: &PassiveSeedRecord) -> bool {
    matches!(
        record.classification_status,
        PassiveSeedClassificationStatus::Elevated | PassiveSeedClassificationStatus::Strategic
    ) || matches!(record.seed_status, PassiveSeedStatus::Elevated)
}

fn average_seed_confidence(records: &[PassiveSeedRecord]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }
    Some(
        records.iter().map(|record| record.confidence).sum::<f64>()
            / f64::from(u32::try_from(records.len()).unwrap_or(u32::MAX)),
    )
}

fn passive_region_site_summary(
    record: PassiveSeedRecord,
    overview: Option<&PassiveSiteOverview>,
) -> PassiveRegionSiteSummary {
    let recent_event_count = overview.map_or(0, |overview| overview.recent_events.len());
    let critical_event_count = overview.map_or(0, |overview| {
        overview
            .recent_events
            .iter()
            .filter(|event| event.risk_score >= 0.8)
            .count()
    });
    let latest_peak_risk =
        overview.and_then(|overview| overview.latest_risk.as_ref().map(|risk| risk.peak_risk));

    PassiveRegionSiteSummary {
        seed_key: record.seed_key,
        site_id: record.site_id,
        site_name: record.seed.name,
        site_type: record.seed.site_type,
        criticality: record.seed.criticality,
        scan_priority: record.scan_priority,
        observation_confidence: record.confidence,
        recent_event_count,
        critical_event_count,
        latest_peak_risk,
        risk_direction: overview.map_or_else(
            || "sem observacao".to_string(),
            |overview| overview.risk_history_narrative.risk_direction.clone(),
        ),
        narrative: overview.map_or_else(
            || "Seed descoberto; ainda sem historico operacional suficiente.".to_string(),
            |overview| overview.risk_history_narrative.narrative.clone(),
        ),
    }
}

fn region_site_sort_score(site: &PassiveRegionSiteSummary) -> f64 {
    let event_weight =
        (f64::from(u32::try_from(site.recent_event_count).unwrap_or(u32::MAX)) * 0.03).min(0.15);
    let critical_weight =
        (f64::from(u32::try_from(site.critical_event_count).unwrap_or(u32::MAX)) * 0.10).min(0.30);
    site.latest_peak_risk.unwrap_or(0.0) * 0.45
        + site.scan_priority * 0.35
        + site.observation_confidence * 0.10
        + event_weight
        + critical_weight
}

fn canonical_event_path(canonical_event_id: &str) -> String {
    format!("/v1/passive/canonical-events/{canonical_event_id}")
}

fn evidence_path(bundle_hash: &str) -> String {
    format!("/v1/evidence/{bundle_hash}")
}

fn replay_path(manifest_hash: &str) -> String {
    format!("/v1/replay/{manifest_hash}/execute")
}

fn passive_region_narrative_provenance(
    region_id: &str,
    top_sites: &[PassiveRegionSiteSummary],
) -> PassiveRegionNarrativeProvenance {
    let top_site_ids = top_sites
        .iter()
        .filter_map(|site| site.site_id.as_deref())
        .take(5)
        .collect::<Vec<_>>();

    PassiveRegionNarrativeProvenance {
        region_overview_path: format!("/v1/passive/regions/{region_id}/overview"),
        semantic_timeline_path: format!(
            "/v1/passive/regions/{region_id}/semantic-timeline?limit=6&window_hours=720"
        ),
        operational_timeline_path: format!(
            "/v1/passive/regions/{region_id}/operational-timeline?window_hours=72&bucket_hours=6&include_empty_buckets=false"
        ),
        remediation_path: format!("/v1/passive/regions/{region_id}/remediation"),
        runs_path: format!("/v1/passive/regions/runs?region_id={region_id}"),
        source_health_path: format!("/v1/passive/source-health/samples?limit=20&region_id={region_id}"),
        top_site_overview_paths: top_site_ids
            .iter()
            .map(|site_id| format!("/v1/passive/sites/{site_id}/overview"))
            .collect(),
        top_site_narrative_paths: top_site_ids
            .iter()
            .map(|site_id| format!("/v1/passive/sites/{site_id}/narrative?days=30"))
            .collect(),
    }
}

struct PassiveRegionNarrativeStats {
    days: u32,
    seed_count: usize,
    observed_seed_count: usize,
    recent_event_count: usize,
    critical_event_count: usize,
    highest_scan_priority: f64,
    average_observation_confidence: f64,
    discovery_due: bool,
}

fn passive_region_overview_narrative(
    region: &PassiveRegionTarget,
    stats: &PassiveRegionNarrativeStats,
) -> String {
    let discovery_status = if stats.discovery_due {
        "discovery vencido"
    } else {
        "discovery fresco"
    };
    format!(
        "Nos ultimos {} dias, a regiao {} tem {} seeds conhecidos, {} ja observados, {} eventos recentes e {} eventos criticos. Prioridade maxima {:.2}, confianca observacional media {:.2}; {}.",
        stats.days.max(1),
        region.name,
        stats.seed_count,
        stats.observed_seed_count,
        stats.recent_event_count,
        stats.critical_event_count,
        stats.highest_scan_priority,
        stats.average_observation_confidence,
        discovery_status
    )
}

fn passive_region_overdue_seconds(
    target: &PassiveRegionTarget,
    now_unix_seconds: i64,
    cadence_seconds: i64,
) -> i64 {
    target
        .last_discovered_at_unix_seconds
        .map_or(cadence_seconds, |last_discovered| {
            now_unix_seconds
                .saturating_sub(last_discovered)
                .saturating_sub(cadence_seconds)
        })
}

fn passive_region_id(name: &str, south: f64, west: f64) -> String {
    let mut slug = name
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }
    let slug = slug.trim_matches('-');
    let prefix = if slug.is_empty() {
        "passive-region"
    } else {
        slug
    };
    format!("{prefix}-{south:.3}-{west:.3}")
}

fn risk_direction_label(baseline_peak_risk: f64, latest_peak_risk: f64) -> &'static str {
    if latest_peak_risk > baseline_peak_risk + 0.10 {
        "a subir"
    } else if latest_peak_risk + 0.10 < baseline_peak_risk {
        "a descer"
    } else {
        "estavel"
    }
}

fn average_f64(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        let count = u32::try_from(values.len()).unwrap_or(u32::MAX);
        values.iter().sum::<f64>() / f64::from(count)
    }
}

#[allow(clippy::cast_possible_truncation)]
fn f64_to_f32(value: f64) -> f32 {
    if value.is_nan() {
        return 0.0;
    }
    if value > f64::from(f32::MAX) {
        return f32::MAX;
    }
    if value < f64::from(f32::MIN) {
        return f32::MIN;
    }
    value as f32
}

fn csv_index(headers: &csv::StringRecord, candidates: &[&str]) -> Result<usize, AppError> {
    headers
        .iter()
        .position(|header| {
            candidates
                .iter()
                .any(|candidate| header.eq_ignore_ascii_case(candidate))
        })
        .ok_or_else(|| {
            AppError::SourceUnavailable(format!(
                "CSV payload missing expected columns: {}",
                candidates.join(", ")
            ))
        })
}

fn bounding_box(latitude: f64, longitude: f64, radius_km: f64) -> (f64, f64, f64, f64) {
    let latitude_delta = radius_km / 111.0;
    let longitude_delta = radius_km / (111.320 * latitude.to_radians().cos().abs().max(0.1));
    (
        longitude - longitude_delta,
        latitude - latitude_delta,
        longitude + longitude_delta,
        latitude + latitude_delta,
    )
}

fn neows_priority_score(
    hazardous: bool,
    miss_distance_km: Option<f64>,
    relative_velocity_km_s: Option<f64>,
    estimated_diameter_max_m: Option<f64>,
    close_approach_date: Option<&str>,
    start_date: &str,
) -> f64 {
    let hazardous_score = if hazardous { 0.4 } else { 0.0 };
    let distance_score = miss_distance_km.map_or(0.0, |distance| {
        (1_000_000.0 / distance.max(1_000.0)).clamp(0.0, 0.3)
    });
    let velocity_score =
        relative_velocity_km_s.map_or(0.0, |velocity| (velocity / 40.0).clamp(0.0, 0.15));
    let size_score =
        estimated_diameter_max_m.map_or(0.0, |diameter| (diameter / 300.0).clamp(0.0, 0.2));
    let temporal_score =
        close_approach_date.map_or(0.0, |date| if date == start_date { 0.15 } else { 0.08 });
    (hazardous_score + distance_score + velocity_score + size_score + temporal_score)
        .clamp(0.0, 1.0)
}

fn neows_briefing_summary(hazardous: bool, close_approach_date: Option<&str>) -> String {
    match (hazardous, close_approach_date) {
        (true, Some(date)) => {
            format!(
                "Potentially hazardous NEO with close approach inside selected window on {date}."
            )
        }
        (false, Some(date)) => {
            format!("Non-hazardous NEO with relevant close approach on {date}.")
        }
        (true, None) => {
            "Potentially hazardous NEO without resolved close-approach date in feed.".to_string()
        }
        (false, None) => "NEO present in feed without close-approach timing details.".to_string(),
    }
}

fn close_approach_epochs(base_epoch_unix_seconds: i64, horizon_hours: f64) -> Vec<i64> {
    let rounded_horizon_hours = rounded_seconds_from_hours(horizon_hours.ceil()) / 3600;
    let mut epochs = (0..=rounded_horizon_hours)
        .map(|offset_hours| base_epoch_unix_seconds + offset_hours * 3600)
        .collect::<Vec<_>>();
    let precise_horizon_epoch = base_epoch_unix_seconds + rounded_seconds_from_hours(horizon_hours);
    if epochs.last().copied() != Some(precise_horizon_epoch) {
        epochs.push(precise_horizon_epoch);
    }
    epochs.sort_unstable();
    epochs.dedup();
    epochs
}

fn close_approach_priority_score(
    miss_distance_km: f64,
    hours_until_event: f64,
    criticality: f64,
    threshold_km: f64,
) -> f64 {
    let distance_factor = (1.0 - (miss_distance_km / threshold_km)).clamp(0.0, 1.0);
    let urgency_factor = (1.0 - (hours_until_event / 72.0)).clamp(0.0, 1.0);
    (distance_factor * 0.55 + urgency_factor * 0.25 + criticality * 0.20).clamp(0.0, 1.0)
}

fn seconds_to_hours(seconds: i64) -> f64 {
    Duration::from_secs(seconds.unsigned_abs()).as_secs_f64() / 3600.0
}

fn propagation_model_for_close_approach(
    primary_object: &SpaceObject,
    counterpart: &SpaceObject,
) -> &'static str {
    if primary_object.tle.is_some() && counterpart.tle.is_some() {
        "sgp4-pair"
    } else if primary_object.tle.is_some() || counterpart.tle.is_some() {
        "hybrid"
    } else {
        "linear"
    }
}

fn compare_close_approaches(
    left: &CloseApproachCandidate,
    right: &CloseApproachCandidate,
) -> Ordering {
    right
        .priority_score
        .total_cmp(&left.priority_score)
        .then_with(|| left.miss_distance_km.total_cmp(&right.miss_distance_km))
        .then_with(|| {
            left.target_epoch_unix_seconds
                .cmp(&right.target_epoch_unix_seconds)
        })
}

#[must_use]
pub fn now_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .try_into()
        .unwrap_or(i64::MAX)
}

fn default_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("serial-alice-sky-sss-api/0.1")
        .build()
        .expect("default SSS HTTP client should build")
}

fn demo_object() -> SpaceObject {
    SpaceObject {
        id: "SAT-001".to_string(),
        name: "Serial Alice Demo Object".to_string(),
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
            operator: Some("synthetic".to_string()),
            mission_class: MissionClass::Commercial,
            criticality: 0.55,
        },
    }
}

fn demo_observation(object: &SpaceObject) -> Observation {
    Observation {
        object_id: object.id.clone(),
        observed_state: OrbitalState {
            epoch_unix_seconds: 1_800_000_060,
            position_km: Vector3 {
                x: 6_940.0,
                y: 470.0,
                z: 0.0,
            },
            velocity_km_s: Vector3 {
                x: 0.05,
                y: 7.63,
                z: 0.0,
            },
        },
        rf_mhz: Some(2_241.1),
        maneuver_detected: true,
        source: ObservationSource::Synthetic,
    }
}
