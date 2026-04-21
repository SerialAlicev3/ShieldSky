use std::time::Duration;

use sss_site_registry::{Criticality, ProtectedAsset, ProtectedZone, SiteExposureProfile};

#[must_use]
pub fn risk_score(
    track_confidence: f32,
    threat_probability: f32,
    zone_criticality: Criticality,
    asset_criticality: Criticality,
    exposure_sensitivity: f32,
    impact_window_seconds: i64,
) -> f32 {
    let urgency_factor = 1.0
        + (3600.0 / Duration::from_secs(impact_window_seconds.max(1).unsigned_abs()).as_secs_f32());
    track_confidence
        * threat_probability
        * criticality_weight(zone_criticality)
        * criticality_weight(asset_criticality)
        * exposure_sensitivity
        * urgency_factor
}

#[must_use]
pub fn zone_exposure_sensitivity(
    profile: &SiteExposureProfile,
    zone: &ProtectedZone,
    asset: &ProtectedAsset,
) -> f32 {
    let base = match zone.zone_type {
        sss_site_registry::ZoneType::PanelField => profile.irradiance_variability_sensitivity,
        sss_site_registry::ZoneType::Cooling => {
            profile.smoke_sensitivity.max(profile.wind_sensitivity)
        }
        sss_site_registry::ZoneType::Perimeter => profile.drone_risk_sensitivity,
        sss_site_registry::ZoneType::Substation => {
            profile.rf_sensitivity.max(profile.wind_sensitivity)
        }
        sss_site_registry::ZoneType::InverterArea | sss_site_registry::ZoneType::Rooftop => profile
            .drone_risk_sensitivity
            .max(profile.smoke_sensitivity),
    };
    (base + asset.operational_dependency_score).clamp(0.1, 2.0)
}

#[must_use]
pub fn criticality_weight(value: Criticality) -> f32 {
    match value {
        Criticality::Low => 0.35,
        Criticality::Medium => 0.55,
        Criticality::High => 0.8,
        Criticality::Critical => 1.0,
    }
}
