use uuid::Uuid;

use crate::{SignalSource, SignalType, SkySignal};

#[must_use]
pub fn radar_contact(site_id: Uuid, observed_at_unix_seconds: i64) -> SkySignal {
    SkySignal {
        signal_id: Uuid::new_v4(),
        site_id,
        source: SignalSource::Radar,
        signal_type: SignalType::TrackDetection,
        observed_at_unix_seconds,
        confidence_hint: 0.88,
        payload: serde_json::json!({"range_m": 420.0, "bearing_deg": 18.0}),
    }
}
