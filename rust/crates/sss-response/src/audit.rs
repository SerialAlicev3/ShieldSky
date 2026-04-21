use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseAuditRecord {
    pub audit_id: Uuid,
    pub incident_id: Uuid,
    pub bundle_hash: String,
    pub summary: String,
}
