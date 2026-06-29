use crate::{
    error::{AppError, AppResult},
    models::{
        metrics::LendingPolicy,
        proof::ProofPackage,
    },
    services::loan_engine,
    AppState,
};
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct EvaluateLoanRequest {
    pub proof_package: ProofPackage,
    pub policy: Option<LendingPolicy>,
    pub lender_id: Option<Uuid>,
}

/// POST /loan/evaluate
pub async fn evaluate(
    State(state): State<AppState>,
    Json(req): Json<EvaluateLoanRequest>,
) -> AppResult<Json<Value>> {
    let lender_id = req.lender_id.unwrap_or_else(|| {
        Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap()
    });
    let policy = req.policy.unwrap_or_default();
    let policy_json = serde_json::to_value(&policy).unwrap();
    let application_id = Uuid::new_v4();
    let circuits_dir = state.config.circuits_dir.clone();
    let package = req.proof_package.clone();

    let decision = tokio::task::spawn_blocking(move || {
        loan_engine::evaluate(application_id, &package, &policy, &circuits_dir)
    })
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?
    .map_err(|e| AppError::Internal(e))?;

    // Persist application
    let decided_at = if decision.decision != "pending" {
        Some(chrono::Utc::now())
    } else {
        None
    };

    sqlx::query(
        "INSERT INTO loan_applications
         (id, merchant_id, lender_id, proof_id, policy, decision, reason, decided_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(decision.application_id)
    .bind(req.proof_package.merchant_id)
    .bind(lender_id)
    .bind(req.proof_package.proof_id)
    .bind(policy_json)
    .bind(&decision.decision)
    .bind(&decision.reason)
    .bind(decided_at)
    .execute(&state.db)
    .await?;

    Ok(Json(json!({
        "application_id": decision.application_id,
        "decision": decision.decision,
        "reason": decision.reason,
        "proof_verified": decision.proof_verified,
        "policy_met": decision.policy_met,
        "failed_predicates": decision.failed_predicates,
    })))
}

/// GET /loan/:id
pub async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let row: Option<(Uuid, Uuid, Uuid, Uuid, serde_json::Value, Option<String>, Option<String>)> =
        sqlx::query_as(
            "SELECT id, merchant_id, lender_id, proof_id, policy, decision, reason
             FROM loan_applications WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&state.db)
        .await?;

    let (id, merchant_id, lender_id, proof_id, policy, decision, reason) =
        row.ok_or_else(|| AppError::NotFound("loan application not found".into()))?;

    Ok(Json(json!({
        "id": id,
        "merchant_id": merchant_id,
        "lender_id": lender_id,
        "proof_id": proof_id,
        "policy": policy,
        "decision": decision,
        "reason": reason,
    })))
}
