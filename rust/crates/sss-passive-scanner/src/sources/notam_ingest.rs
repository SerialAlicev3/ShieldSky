use serde_json::json;
use uuid::Uuid;

use crate::domain::{NotamObservation, PassiveObservation, PassiveSourceKind};

#[must_use]
pub fn normalize_notams(record: &NotamObservation) -> PassiveObservation {
    PassiveObservation {
        observation_id: Uuid::new_v4(),
        source_kind: PassiveSourceKind::Notam,
        observed_at_unix_seconds: record.observed_at_unix_seconds,
        latitude: record.latitude,
        longitude: record.longitude,
        altitude_m: 0.0,
        coverage_radius_km: record.radius_km,
        severity_hint: f64::from(record.severity_hint).min(1.0),
        summary: format!("NOTAM {} overlaps monitored area", record.notam_id),
        payload: json!({
            "notam_id": record.notam_id,
            "latitude": record.latitude,
            "longitude": record.longitude,
            "radius_km": record.radius_km,
            "restriction_summary": record.restriction_summary,
            "severity_hint": record.severity_hint,
        }),
    }
}
