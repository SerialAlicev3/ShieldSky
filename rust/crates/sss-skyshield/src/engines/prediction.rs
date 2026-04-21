use crate::AerialTrack;

#[must_use]
pub fn predicted_path_summary(track: &AerialTrack) -> String {
    format!(
        "track {} projected heading ({:.1}, {:.1}, {:.1})",
        track.track_id, track.velocity_vector.x, track.velocity_vector.y, track.velocity_vector.z
    )
}
