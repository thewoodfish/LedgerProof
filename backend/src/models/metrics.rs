use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FinancialMetrics {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub computed_at: DateTime<Utc>,

    pub monthly_revenue: Value,         // {"2024-01": 500000000}
    pub avg_monthly_revenue: i64,       // kobo
    pub revenue_volatility_bps: i32,    // basis points (100 bps = 1%)

    pub monthly_cash_flow: Value,
    pub positive_cash_flow_months: i32,

    pub avg_monthly_balance: i64,
    pub min_balance: i64,

    pub revenue_growth_months: i32,

    pub avg_monthly_tx_count: i32,

    pub customer_concentration_bps: i32,
    pub supplier_concentration_bps: i32,

    pub expense_variance_bps: i32,

    pub debt_ratio_bps: i32,

    pub has_missed_repayments: bool,

    pub account_age_months: i32,
}

/// Lender-configured underwriting policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LendingPolicy {
    /// Minimum average monthly revenue in kobo
    pub required_monthly_revenue: Option<i64>,
    /// Minimum average balance in kobo
    pub required_avg_balance: Option<i64>,
    /// Minimum number of months with positive cash flow (out of 6)
    pub required_positive_cash_flow_months: Option<i32>,
    /// Maximum revenue volatility in basis points (e.g. 1500 = 15%)
    pub max_revenue_volatility_bps: Option<i32>,
    /// Maximum customer concentration in basis points (e.g. 2500 = 25%)
    pub max_customer_concentration_bps: Option<i32>,
    /// Maximum debt ratio in basis points (e.g. 2500 = 25%)
    pub max_debt_ratio_bps: Option<i32>,
    /// Require no missed repayments
    pub require_no_missed_repayments: Option<bool>,
    /// Minimum account age in months
    pub required_account_age_months: Option<i32>,
}

impl Default for LendingPolicy {
    fn default() -> Self {
        Self {
            required_monthly_revenue: Some(500_000_000),   // ₦5M in kobo
            required_avg_balance: Some(50_000_000),        // ₦500k in kobo
            required_positive_cash_flow_months: Some(4),
            max_revenue_volatility_bps: Some(1500),        // 15%
            max_customer_concentration_bps: Some(2500),    // 25%
            max_debt_ratio_bps: Some(2500),                // 25%
            require_no_missed_repayments: Some(true),
            required_account_age_months: Some(12),
        }
    }
}
