use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Criticality {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SiteType {
    SolarPlant,
    DataCenter,
    Substation,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeoPoint {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude_m: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Site {
    pub site_id: Uuid,
    pub name: String,
    pub site_type: SiteType,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation_m: f32,
    pub timezone: String,
    pub country_code: String,
    pub criticality: Criticality,
    pub operator_name: String,
}
