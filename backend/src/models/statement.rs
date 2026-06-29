use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Statement {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub filename: String,
    pub month: String,
    pub raw_text: Option<String>,
    pub status: String,
    pub error_msg: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UploadStatementQuery {
    pub month: String,
}
