use serde::{Deserialize, Serialize};
use uuid::Uuid;

use sss_site_registry::GeoPoint;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalSource {
    Radar,
    Rf,
    EoIr,
    Weather,
    Smoke,
    Airspace,
    OrbitalContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalType {
    TrackDetection,
    RemoteId,
    WeatherCell,
    SmokePlume,
    RfBurst,
    IrradianceDrop,
    OverheadContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectType {
    Drone,
    Aircraft,
    Unknown,
    Balloon,
    BirdLike,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkySignal {
    pub signal_id: Uuid,
    pub site_id: Uuid,
    pub source: SignalSource,
    pub signal_type: SignalType,
    pub observed_at_unix_seconds: i64,
    pub confidence_hint: f32,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AerialTrack {
    pub track_id: Uuid,
    pub site_id: Uuid,
    pub object_type: ObjectType,
    pub first_seen_unix_seconds: i64,
    pub last_seen_unix_seconds: i64,
    pub current_position: GeoPoint,
    pub velocity_vector: Vec3,
    pub altitude_m: f32,
    pub rf_signature: Option<String>,
    pub remote_id: Option<String>,
    pub source_fusion_summary: String,
}
