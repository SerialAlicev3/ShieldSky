use uuid::Uuid;

use crate::{SignalSource, SignalType, SkySignal};

#[must_use]
pub fn smoke_intrusion(site_id: Uuid, observed_at_unix_seconds: i64) -> SkySignal {
    SkySignal {
        signal_id: Uuid::new_v4(),
        site_id,
        source: SignalSource::Smoke,
        signal_type: SignalType::SmokePlume,
        observed_at_unix_seconds,
        confidence_hint: 0.79,
        payload: serde_json::json!({"pm25_index": 143, "direction": "west-east"}),
    }
}
