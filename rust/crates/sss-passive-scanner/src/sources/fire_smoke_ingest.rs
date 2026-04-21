use serde_json::json;
use uuid::Uuid;

use crate::domain::{FireSmokeObservation, PassiveObservation, PassiveSourceKind};

#[must_use]
pub fn normalize_fire_smoke(record: &FireSmokeObservation) -> PassiveObservation {
    let severity_hint = f64::from(
        ((record.fire_radiative_power / 200.0).min(1.0) * 0.45)
            + ((record.smoke_density_index / 100.0).min(1.0) * 0.55),
    );

    PassiveObservation {
        observation_id: Uuid::new_v4(),
        source_kind: PassiveSourceKind::FireSmoke,
        observed_at_unix_seconds: record.observed_at_unix_seconds,
        latitude: record.latitude,
        longitude: record.longitude,
        altitude_m: 0.0,
        coverage_radius_km: 55.0,
        severity_hint,
        summary: "Fire or smoke plume context intersects monitored region".to_string(),
        payload: json!({
            "latitude": record.latitude,
            "longitude": record.longitude,
            "fire_radiative_power": record.fire_radiative_power,
            "smoke_density_index": record.smoke_density_index,
        }),
    }
}
