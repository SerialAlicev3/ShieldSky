use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseWebhookDelivery {
    pub delivery_id: Uuid,
    pub target: String,
    pub channel: String,
    pub payload_summary: String,
}
