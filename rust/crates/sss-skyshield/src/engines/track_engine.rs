use sss_site_registry::{GeoPoint, Site};
use uuid::Uuid;

use crate::{AerialTrack, ObjectType, SkySignal, Vec3};

#[must_use]
pub fn track_observation(site: &Site, signals: &[SkySignal]) -> Option<AerialTrack> {
    let first = signals.first()?;
    let last_seen = signals
        .iter()
        .map(|signal| signal.observed_at_unix_seconds)
        .max()
        .unwrap_or(first.observed_at_unix_seconds);
    Some(AerialTrack {
        track_id: Uuid::new_v4(),
        site_id: site.site_id,
        object_type: ObjectType::Drone,
        first_seen_unix_seconds: first.observed_at_unix_seconds,
        last_seen_unix_seconds: last_seen,
        current_position: GeoPoint {
            latitude: site.latitude + 0.0008,
            longitude: site.longitude + 0.0007,
            altitude_m: site.elevation_m + 120.0,
        },
        velocity_vector: Vec3 {
            x: 4.0,
            y: 1.5,
            z: 0.0,
        },
        altitude_m: 120.0,
        rf_signature: Some("2.4GHz-link".to_string()),
        remote_id: None,
        source_fusion_summary: crate::engines::signal_fusion::fusion_summary(signals),
    })
}
