use serde_json::json;
use uuid::Uuid;

use crate::domain::{OrbitalPassObservation, PassiveObservation, PassiveSourceKind};

#[must_use]
pub fn normalize_orbital(record: &OrbitalPassObservation) -> PassiveObservation {
    PassiveObservation {
        observation_id: Uuid::new_v4(),
        source_kind: PassiveSourceKind::Orbital,
        observed_at_unix_seconds: record.observed_at_unix_seconds,
        latitude: record.latitude,
        longitude: record.longitude,
        altitude_m: 120_000.0,
        coverage_radius_km: record.footprint_radius_km,
        severity_hint: f64::from(record.reentry_probability).min(1.0),
        summary: format!("Orbital context for object {}", record.object_id),
        payload: json!({
            "object_id": record.object_id,
            "latitude": record.latitude,
            "longitude": record.longitude,
            "reentry_probability": record.reentry_probability,
            "footprint_radius_km": record.footprint_radius_km,
            "altitude_m": 120_000.0,
        }),
    }
}
