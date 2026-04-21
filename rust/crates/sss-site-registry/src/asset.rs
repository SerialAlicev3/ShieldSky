use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Criticality;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetType {
    SolarArray,
    Inverter,
    Transformer,
    CoolingUnit,
    RoofHvac,
    FiberHub,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtectedAsset {
    pub asset_id: Uuid,
    pub site_id: Uuid,
    pub zone_id: Uuid,
    pub asset_type: AssetType,
    pub criticality: Criticality,
    pub operational_dependency_score: f32,
}
