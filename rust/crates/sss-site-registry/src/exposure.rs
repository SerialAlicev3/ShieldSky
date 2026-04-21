use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SiteExposureProfile {
    pub site_id: Uuid,
    pub drone_risk_sensitivity: f32,
    pub rf_sensitivity: f32,
    pub hail_sensitivity: f32,
    pub wind_sensitivity: f32,
    pub smoke_sensitivity: f32,
    pub dust_sensitivity: f32,
    pub irradiance_variability_sensitivity: f32,
}
