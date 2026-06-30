use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LenderProfile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub display_name: String,
    pub description: String,
    pub policy: Value,
    pub published: bool,
    pub stellar_policy_tx: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertProfileRequest {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub policy: Option<Value>,
    pub published: Option<bool>,
}
