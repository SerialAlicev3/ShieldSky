use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Criticality, GeoPoint};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneType {
    PanelField,
    InverterArea,
    Cooling,
    Rooftop,
    Perimeter,
    Substation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AirspacePolicy {
    Open,
    Restricted,
    AuthorizedOnly,
    NoFly,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtectedZone {
    pub zone_id: Uuid,
    pub site_id: Uuid,
    pub name: String,
    pub zone_type: ZoneType,
    pub polygon: Vec<GeoPoint>,
    pub criticality: Criticality,
    pub allowed_airspace_policy: AirspacePolicy,
}
