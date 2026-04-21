use serde_json::json;
use uuid::Uuid;

use crate::domain::{AdsbFlightObservation, PassiveObservation, PassiveSourceKind};

#[must_use]
pub fn normalize_adsb(record: &AdsbFlightObservation) -> PassiveObservation {
    PassiveObservation {
        observation_id: Uuid::new_v4(),
        source_kind: PassiveSourceKind::Adsb,
        observed_at_unix_seconds: record.observed_at_unix_seconds,
        latitude: record.latitude,
        longitude: record.longitude,
        altitude_m: record.altitude_m,
        coverage_radius_km: 8.0,
        severity_hint: if record.altitude_m < 1_000.0 {
            0.75
        } else {
            0.45
        },
        summary: format!(
            "ADS-B track {} over critical-infra corridor",
            record.flight_id
        ),
        payload: json!({
            "flight_id": record.flight_id,
            "latitude": record.latitude,
            "longitude": record.longitude,
            "altitude_m": record.altitude_m,
            "speed_kts": record.speed_kts,
            "vertical_rate_m_s": record.vertical_rate_m_s,
        }),
    }
}
