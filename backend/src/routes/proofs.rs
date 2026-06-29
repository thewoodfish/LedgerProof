use crate::{
    error::{AppError, AppResult},
    models::{
        metrics::{FinancialMetrics, LendingPolicy},
        proof::{Proof, ProofPackage},
    },
    services::proof_gen,
    AppState,
};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct GenerateProofRequest {
    pub metrics_id: Uuid,
    pub policy: LendingPolicy,
}

#[derive(Deserialize)]
pub struct VerifyProofRequest {
    pub proof_package: ProofPackage,
}

/// POST /generate-proof
pub async fn generate(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<GenerateProofRequest>,
) -> AppResult<Json<Value>> {
    let merchant_id = crate::routes::statements::merchant_id_from_headers(&headers);

    // Load metrics
    let metrics: FinancialMetrics = sqlx::query_as(
        "SELECT * FROM financial_metrics WHERE id = $1 AND merchant_id = $2",
    )
    .bind(req.metrics_id)
    .bind(merchant_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("metrics not found".into()))?;

    let proof_id = Uuid::new_v4();
    let circuits_dir = state.config.circuits_dir.clone();

    let package = tokio::task::spawn_blocking(move || {
        proof_gen::generate(proof_id, &metrics, &req.policy, &circuits_dir)
    })
    .await
    .map_err(|e| AppError::ProofGen(e.to_string()))?
    .map_err(|e| AppError::ProofGen(e.to_string()))?;

    // Persist proof
    sqlx::query(
        "INSERT INTO proofs
         (id, merchant_id, metrics_id, circuit_id, proof_hex, vk_hex, public_inputs, predicates)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(package.proof_id)
    .bind(package.merchant_id)
    .bind(req.metrics_id)
    .bind(&package.circuit_id)
    // Store combined "proof_hex|pub_inputs_hex" so it round-trips fully
    .bind(format!("{}|{}", package.proof_hex, package.pub_inputs_hex))
    .bind(&package.vk_hex)
    .bind(serde_json::to_value(&package.public_inputs).unwrap())
    .bind(serde_json::to_value(&package.predicates).unwrap())
    .execute(&state.db)
    .await?;

    Ok(Json(json!({
        "proof_id": package.proof_id,
        "circuit_id": package.circuit_id,
        "predicates": package.predicates,
        "package": package,
    })))
}

/// POST /verify-proof
pub async fn verify(
    State(state): State<AppState>,
    Json(req): Json<VerifyProofRequest>,
) -> AppResult<Json<Value>> {
    let circuits_dir = state.config.circuits_dir.clone();
    let package = req.proof_package.clone();

    let verified = tokio::task::spawn_blocking(move || {
        proof_gen::verify(&package, &circuits_dir)
    })
    .await
    .map_err(|e| AppError::ProofGen(e.to_string()))?
    .map_err(|e| AppError::ProofGen(e.to_string()))?;

    Ok(Json(json!({
        "verified": verified,
        "circuit_id": req.proof_package.circuit_id,
        "predicates": req.proof_package.predicates,
    })))
}

/// GET /proofs/:id
pub async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Proof>> {
    let row: Option<Proof> =
        sqlx::query_as("SELECT * FROM proofs WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.db)
            .await?;

    row.map(Json)
        .ok_or_else(|| AppError::NotFound("proof not found".into()))
}
