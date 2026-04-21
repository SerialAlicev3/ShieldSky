use serde_json::json;
use uuid::Uuid;

use crate::domain::{PassiveObservation, PassiveSourceKind, SatelliteSceneObservation};

#[must_use]
pub fn normalize_satellite(record: &SatelliteSceneObservation) -> PassiveObservation {
    PassiveObservation {
        observation_id: Uuid::new_v4(),
        source_kind: PassiveSourceKind::Satellite,
        observed_at_unix_seconds: record.observed_at_unix_seconds,
        latitude: record.latitude,
        longitude: record.longitude,
        altitude_m: 0.0,
        coverage_radius_km: 6.0,
        severity_hint: f64::from(record.change_score).min(1.0),
        summary: format!(
            "Satellite scene {} indicates surface change",
            record.scene_id
        ),
        payload: json!({
            "scene_id": record.scene_id,
            "latitude": record.latitude,
            "longitude": record.longitude,
            "change_score": record.change_score,
            "cloud_cover_ratio": record.cloud_cover_ratio,
        }),
    }
}
