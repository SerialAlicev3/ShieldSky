use serde_json::json;
use uuid::Uuid;

use crate::domain::{PassiveObservation, PassiveSourceKind, WeatherObservation};

#[must_use]
pub fn normalize_weather(record: &WeatherObservation) -> PassiveObservation {
    let severity_hint = f64::from(
        (record.hail_probability * 0.55)
            + (record.irradiance_drop_ratio * 0.25)
            + ((record.wind_kph / 120.0).min(1.0) * 0.20),
    );

    PassiveObservation {
        observation_id: Uuid::new_v4(),
        source_kind: PassiveSourceKind::Weather,
        observed_at_unix_seconds: record.observed_at_unix_seconds,
        latitude: record.latitude,
        longitude: record.longitude,
        altitude_m: 0.0,
        coverage_radius_km: 35.0,
        severity_hint,
        summary: "Weather cell intersects monitored infrastructure region".to_string(),
        payload: json!({
            "latitude": record.latitude,
            "longitude": record.longitude,
            "wind_kph": record.wind_kph,
            "hail_probability": record.hail_probability,
            "irradiance_drop_ratio": record.irradiance_drop_ratio,
        }),
    }
}
