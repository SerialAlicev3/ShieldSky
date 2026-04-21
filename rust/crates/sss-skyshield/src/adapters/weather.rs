use uuid::Uuid;

use crate::{SignalSource, SignalType, SkySignal};

#[must_use]
pub fn irradiance_drop(site_id: Uuid, observed_at_unix_seconds: i64) -> SkySignal {
    SkySignal {
        signal_id: Uuid::new_v4(),
        site_id,
        source: SignalSource::Weather,
        signal_type: SignalType::IrradianceDrop,
        observed_at_unix_seconds,
        confidence_hint: 0.76,
        payload: serde_json::json!({"irradiance_drop_pct": 31.0, "storm_cell_eta_min": 18}),
    }
}
