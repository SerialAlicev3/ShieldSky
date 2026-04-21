use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreatType {
    UnauthorizedOverflight,
    PerimeterAirspaceViolation,
    HoverOverCriticalZone,
    RepeatPassPattern,
    RfLinkAnomaly,
    SuspectedPayloadRisk,
    MultiDronePattern,
    IrradianceDropExpected,
    StormCellApproach,
    HailRisk,
    SmokeIntrusionRisk,
    DustHazeDetected,
    CoolingAirQualityRisk,
    WindLoadRisk,
    LightningRisk,
    OrbitalReentryContext,
    SpaceWeatherContext,
    OverheadRiskContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentType {
    DroneDefense,
    Environmental,
    OrbitalContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentStatus {
    Detected,
    UnderAssessment,
    Confirmed,
    Escalated,
    ResponseAuthorized,
    ResponseInProgress,
    Contained,
    Closed,
    Replayed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkyThreatAssessment {
    pub assessment_id: Uuid,
    pub site_id: Uuid,
    pub threat_type: ThreatType,
    pub confidence: f32,
    pub impact_probability: f32,
    pub impact_window_seconds: i64,
    pub affected_zones: Vec<Uuid>,
    pub affected_assets: Vec<Uuid>,
    pub explanation: String,
    pub predicted_path_summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SiteIncident {
    pub incident_id: Uuid,
    pub site_id: Uuid,
    pub incident_type: IncidentType,
    pub status: IncidentStatus,
    pub created_at_unix_seconds: i64,
    pub updated_at_unix_seconds: i64,
    pub linked_track_ids: Vec<Uuid>,
    pub linked_event_ids: Vec<Uuid>,
    pub linked_bundle_hashes: Vec<String>,
    pub severity: Severity,
    pub current_owner: Option<String>,
}
