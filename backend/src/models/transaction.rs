use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub statement_id: Uuid,
    pub merchant_id: Uuid,
    pub date: NaiveDate,
    pub description: String,
    pub credit: i64,   // kobo
    pub debit: i64,    // kobo
    pub balance: i64,  // kobo
    pub category: String,
    pub created_at: DateTime<Utc>,
}

/// What the LLM returns for each parsed transaction row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTransaction {
    pub date: String,
    pub description: String,
    pub credit: f64,
    pub debit: f64,
    pub balance: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Revenue,
    Expense,
    LoanRepayment,
    Transfer,
    CashWithdrawal,
    Tax,
    Unknown,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Revenue => write!(f, "revenue"),
            Category::Expense => write!(f, "expense"),
            Category::LoanRepayment => write!(f, "loan_repayment"),
            Category::Transfer => write!(f, "transfer"),
            Category::CashWithdrawal => write!(f, "cash_withdrawal"),
            Category::Tax => write!(f, "tax"),
            Category::Unknown => write!(f, "unknown"),
        }
    }
}
