use crate::AerialTrack;

#[must_use]
pub fn behavior_summary(track: &AerialTrack) -> String {
    if track.velocity_vector.x.abs() < 1.0 && track.velocity_vector.y.abs() < 1.0 {
        "hover behavior over protected zone".to_string()
    } else {
        "transit behavior with persistent perimeter presence".to_string()
    }
}
