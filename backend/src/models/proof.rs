use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Proof {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub metrics_id: Uuid,
    pub circuit_id: String,
    pub proof_hex: String,
    pub vk_hex: String,
    pub public_inputs: Value,
    pub predicates: Value,
    pub created_at: DateTime<Utc>,
}

/// A proof package shared from merchant to lender
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofPackage {
    pub proof_id: Uuid,
    pub merchant_id: Uuid,
    pub circuit_id: String,
    pub proof_hex: String,
    pub vk_hex: String,
    /// Raw public inputs bytes as hex (for bb verify -i)
    pub pub_inputs_hex: String,
    pub public_inputs: PublicInputs,
    pub predicates: Vec<ProvenPredicate>,
    pub created_at: DateTime<Utc>,
}

/// Public inputs to the ZK circuit (what the lender sees)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicInputs {
    pub required_monthly_revenue: u64,
    pub required_avg_balance: u64,
    pub required_positive_cash_flow_months: u64,
    pub max_revenue_volatility_bps: u64,
    pub max_customer_concentration_bps: u64,
    pub max_debt_ratio_bps: u64,
    pub require_no_missed_repayments: u64,
    pub required_account_age_months: u64,
}

/// Human-readable proven predicate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenPredicate {
    pub name: String,
    pub description: String,
    pub satisfied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanDecision {
    pub application_id: Uuid,
    pub decision: String,
    pub reason: String,
    pub proof_verified: bool,
    pub policy_met: bool,
    pub failed_predicates: Vec<String>,
}
