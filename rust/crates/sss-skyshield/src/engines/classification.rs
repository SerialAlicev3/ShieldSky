use sss_site_registry::SiteType;

use crate::{AerialTrack, ThreatType};

#[must_use]
pub fn classify_track(track: &AerialTrack, site_type: &SiteType) -> ThreatType {
    match (track.object_type, site_type) {
        (crate::ObjectType::Drone, SiteType::DataCenter) => ThreatType::HoverOverCriticalZone,
        (crate::ObjectType::Drone, SiteType::Substation) => ThreatType::PerimeterAirspaceViolation,
        (crate::ObjectType::Drone, SiteType::SolarPlant) => ThreatType::UnauthorizedOverflight,
        _ => ThreatType::OverheadRiskContext,
    }
}
