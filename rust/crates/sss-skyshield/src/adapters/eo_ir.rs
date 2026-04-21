use uuid::Uuid;

use crate::{SignalSource, SignalType, SkySignal};

#[must_use]
pub fn thermal_signature(site_id: Uuid, observed_at_unix_seconds: i64) -> SkySignal {
    SkySignal {
        signal_id: Uuid::new_v4(),
        site_id,
        source: SignalSource::EoIr,
        signal_type: SignalType::TrackDetection,
        observed_at_unix_seconds,
        confidence_hint: 0.81,
        payload: serde_json::json!({"thermal_contrast": "medium", "visual_shape": "quadrotor-like"}),
    }
}
