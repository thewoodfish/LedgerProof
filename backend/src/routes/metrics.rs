use crate::{
    error::{AppError, AppResult},
    models::{metrics::FinancialMetrics, transaction::Transaction},
    routes::AuthUser,
    services::metrics as metrics_svc,
    AppState,
};
use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

/// POST /metrics — compute + persist metrics for current user
pub async fn compute(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Value>> {
    let transactions: Vec<Transaction> = sqlx::query_as(
        "SELECT * FROM transactions WHERE merchant_id = $1 ORDER BY date ASC",
    )
    .bind(auth.id)
    .fetch_all(&state.db)
    .await?;

    if transactions.is_empty() {
        return Err(AppError::BadRequest(
            "No transactions found. Upload and parse your statement first.".into(),
        ));
    }

    let metrics = metrics_svc::compute(auth.id, &transactions)
        .map_err(AppError::Internal)?;

    let id: Uuid = sqlx::query_scalar(
        r#"INSERT INTO financial_metrics (
            id, merchant_id, computed_at,
            monthly_revenue, avg_monthly_revenue, revenue_volatility_bps,
            monthly_cash_flow, positive_cash_flow_months,
            avg_monthly_balance, min_balance, revenue_growth_months,
            avg_monthly_tx_count, customer_concentration_bps,
            supplier_concentration_bps, expense_variance_bps,
            debt_ratio_bps, has_missed_repayments, account_age_months
        ) VALUES (
            $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18
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
        "merchant_id": auth.id,
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

/// GET /metrics/latest — most recent metrics for the current user
pub async fn latest_for_current_user(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Value>> {
    let metrics: Option<FinancialMetrics> = sqlx::query_as(
        "SELECT * FROM financial_metrics WHERE merchant_id = $1 ORDER BY computed_at DESC LIMIT 1",
    )
    .bind(auth.id)
    .fetch_optional(&state.db)
    .await?;

    match metrics {
        None => Err(AppError::NotFound("No metrics computed yet".into())),
        Some(m) => Ok(Json(json!({
            "metrics_id": m.id,
            "merchant_id": m.merchant_id,
            "summary": {
                "avg_monthly_revenue_naira": m.avg_monthly_revenue as f64 / 100.0,
                "avg_monthly_balance_naira": m.avg_monthly_balance as f64 / 100.0,
                "positive_cash_flow_months": m.positive_cash_flow_months,
                "revenue_volatility_pct": m.revenue_volatility_bps as f64 / 100.0,
                "debt_ratio_pct": m.debt_ratio_bps as f64 / 100.0,
                "customer_concentration_pct": m.customer_concentration_bps as f64 / 100.0,
                "has_missed_repayments": m.has_missed_repayments,
                "account_age_months": m.account_age_months,
                "revenue_growth_months": m.revenue_growth_months,
                "avg_monthly_tx_count": m.avg_monthly_tx_count,
            },
            "monthly_revenue": m.monthly_revenue,
            "monthly_cash_flow": m.monthly_cash_flow,
        }))),
    }
}

/// GET /metrics/:id
pub async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<FinancialMetrics>> {
    let row: Option<FinancialMetrics> =
        sqlx::query_as("SELECT * FROM financial_metrics WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.db)
            .await?;

    row.map(Json)
        .ok_or_else(|| AppError::NotFound("metrics not found".into()))
}
