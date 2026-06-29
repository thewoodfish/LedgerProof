use crate::{
    error::{AppError, AppResult},
    models::{
        metrics::FinancialMetrics,
        transaction::Transaction,
    },
    services::metrics as metrics_svc,
    AppState,
};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

/// POST /metrics
/// Computes financial metrics for the authenticated merchant.
pub async fn compute(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Json<Value>> {
    let merchant_id = crate::routes::statements::merchant_id_from_headers(&headers);

    let transactions: Vec<Transaction> = sqlx::query_as(
        "SELECT * FROM transactions WHERE merchant_id = $1 ORDER BY date ASC",
    )
    .bind(merchant_id)
    .fetch_all(&state.db)
    .await?;

    if transactions.is_empty() {
        return Err(AppError::BadRequest(
            "No transactions found. Upload and parse statements first.".to_string(),
        ));
    }

    let metrics = metrics_svc::compute(merchant_id, &transactions)
        .map_err(|e| AppError::Internal(e))?;

    // Persist metrics
    let id: Uuid = sqlx::query_scalar(
        r#"INSERT INTO financial_metrics (
            id, merchant_id, computed_at,
            monthly_revenue, avg_monthly_revenue, revenue_volatility_bps,
            monthly_cash_flow, positive_cash_flow_months,
            avg_monthly_balance, min_balance,
            revenue_growth_months,
            avg_monthly_tx_count,
            customer_concentration_bps, supplier_concentration_bps,
            expense_variance_bps, debt_ratio_bps,
            has_missed_repayments, account_age_months
        ) VALUES (
            $1, $2, $3,
            $4, $5, $6,
            $7, $8,
            $9, $10,
            $11,
            $12,
            $13, $14,
            $15, $16,
            $17, $18
        ) RETURNING id"#,
    )
    .bind(metrics.id)
    .bind(metrics.merchant_id)
    .bind(metrics.computed_at)
    .bind(&metrics.monthly_revenue)
    .bind(metrics.avg_monthly_revenue)
    .bind(metrics.revenue_volatility_bps)
    .bind(&metrics.monthly_cash_flow)
    .bind(metrics.positive_cash_flow_months)
    .bind(metrics.avg_monthly_balance)
    .bind(metrics.min_balance)
    .bind(metrics.revenue_growth_months)
    .bind(metrics.avg_monthly_tx_count)
    .bind(metrics.customer_concentration_bps)
    .bind(metrics.supplier_concentration_bps)
    .bind(metrics.expense_variance_bps)
    .bind(metrics.debt_ratio_bps)
    .bind(metrics.has_missed_repayments)
    .bind(metrics.account_age_months)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!({
        "metrics_id": id,
        "merchant_id": merchant_id,
        "summary": {
            "avg_monthly_revenue_naira": metrics.avg_monthly_revenue as f64 / 100.0,
            "avg_monthly_balance_naira": metrics.avg_monthly_balance as f64 / 100.0,
            "positive_cash_flow_months": metrics.positive_cash_flow_months,
            "revenue_volatility_pct": metrics.revenue_volatility_bps as f64 / 100.0,
            "debt_ratio_pct": metrics.debt_ratio_bps as f64 / 100.0,
            "customer_concentration_pct": metrics.customer_concentration_bps as f64 / 100.0,
            "has_missed_repayments": metrics.has_missed_repayments,
            "account_age_months": metrics.account_age_months,
            "revenue_growth_months": metrics.revenue_growth_months,
            "avg_monthly_tx_count": metrics.avg_monthly_tx_count,
        },
        "monthly_revenue": metrics.monthly_revenue,
        "monthly_cash_flow": metrics.monthly_cash_flow,
    })))
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<FinancialMetrics>> {
    let row: Option<FinancialMetrics> = sqlx::query_as(
        "SELECT * FROM financial_metrics WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    row.map(Json)
        .ok_or_else(|| AppError::NotFound("metrics not found".into()))
}

pub async fn latest_for_merchant(
    State(state): State<AppState>,
    Path(merchant_id): Path<Uuid>,
) -> AppResult<Json<FinancialMetrics>> {
    let row: Option<FinancialMetrics> = sqlx::query_as(
        "SELECT * FROM financial_metrics
         WHERE merchant_id = $1
         ORDER BY computed_at DESC
         LIMIT 1",
    )
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await?;

    row.map(Json)
        .ok_or_else(|| AppError::NotFound("no metrics found for merchant".into()))
}
