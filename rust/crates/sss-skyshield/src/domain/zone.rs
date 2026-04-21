use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AffectedZoneImpact {
    pub zone_id: Uuid,
    pub impact_summary: String,
    pub impact_window_seconds: i64,
}
