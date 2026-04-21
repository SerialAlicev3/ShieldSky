use uuid::Uuid;

use crate::{SignalSource, SignalType, SkySignal};

#[must_use]
pub fn orbital_context(site_id: Uuid, observed_at_unix_seconds: i64) -> SkySignal {
    SkySignal {
        signal_id: Uuid::new_v4(),
        site_id,
        source: SignalSource::OrbitalContext,
        signal_type: SignalType::OverheadContext,
        observed_at_unix_seconds,
        confidence_hint: 0.61,
        payload: serde_json::json!({"context": "space-weather-watch", "severity": "medium"}),
    }
}
