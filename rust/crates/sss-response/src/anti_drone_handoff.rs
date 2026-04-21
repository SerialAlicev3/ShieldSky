use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AntiDroneHandoff {
    pub handoff_id: Uuid,
    pub provider_name: String,
    pub action_summary: String,
    pub defensive_only: bool,
}
