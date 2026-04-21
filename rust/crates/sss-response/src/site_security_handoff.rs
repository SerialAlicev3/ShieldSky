use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SiteSecurityHandoff {
    pub handoff_id: Uuid,
    pub team_name: String,
    pub response_summary: String,
}
