use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LoanApplication {
    pub id: Uuid,
    pub borrower_id: Uuid,
    pub lender_profile_id: Uuid,
    pub metrics_id: Uuid,
    pub proof_id: Option<Uuid>,
    pub status: String,
    pub decision_reason: Option<String>,
    pub amount_requested: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub decided_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateApplicationRequest {
    pub lender_profile_id: Uuid,
    pub metrics_id: Uuid,
    pub amount_requested: Option<i64>,
}
