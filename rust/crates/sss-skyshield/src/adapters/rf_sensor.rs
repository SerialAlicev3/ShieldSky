use uuid::Uuid;

use crate::{SignalSource, SignalType, SkySignal};

#[must_use]
pub fn rf_burst(site_id: Uuid, observed_at_unix_seconds: i64) -> SkySignal {
    SkySignal {
        signal_id: Uuid::new_v4(),
        site_id,
        source: SignalSource::Rf,
        signal_type: SignalType::RfBurst,
        observed_at_unix_seconds,
        confidence_hint: 0.72,
        payload: serde_json::json!({"band": "2.4GHz", "summary": "command link detected"}),
    }
}
