use uuid::Uuid;

use crate::{SignalSource, SignalType, SkySignal};

#[must_use]
pub fn airspace_notice(site_id: Uuid, observed_at_unix_seconds: i64) -> SkySignal {
    SkySignal {
        signal_id: Uuid::new_v4(),
        site_id,
        source: SignalSource::Airspace,
        signal_type: SignalType::RemoteId,
        observed_at_unix_seconds,
        confidence_hint: 0.55,
        payload: serde_json::json!({"authorized": false, "remote_id": null}),
    }
}
