use crate::{
    error::{AppError, AppResult},
    models::{
        application::CreateApplicationRequest,
        lender::LenderProfile,
        metrics::FinancialMetrics,
    },
    routes::AuthUser,
    services::{loan_engine, proof_gen},
    AppState,
};
use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

/// POST /applications — borrower submits an application
pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateApplicationRequest>,
) -> AppResult<Json<Value>> {
    if auth.role != "borrower" {
        return Err(AppError::Unauthorized("Borrower role required".into()));
    }

    let metrics_ok: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM financial_metrics WHERE id = $1 AND merchant_id = $2)",
    )
    .bind(req.metrics_id)
    .bind(auth.id)
    .fetch_one(&state.db)
    .await?;

    if !metrics_ok {
        return Err(AppError::NotFound("Metrics not found".into()));
    }

    let profile_ok: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM lender_profiles WHERE id = $1 AND published = TRUE)",
    )
    .bind(req.lender_profile_id)
    .fetch_one(&state.db)
    .await?;

    if !profile_ok {
        return Err(AppError::NotFound(
            "Lender not found or not accepting applications".into(),
        ));
    }

    let dup: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM loan_applications
         WHERE borrower_id = $1 AND lender_profile_id = $2 AND status = 'pending')",
    )
    .bind(auth.id)
    .bind(req.lender_profile_id)
    .fetch_one(&state.db)
    .await?;

    if dup {
        return Err(AppError::BadRequest(
            "You already have a pending application with this lender".into(),
        ));
    }

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO loan_applications
         (borrower_id, lender_profile_id, metrics_id, amount_requested)
         VALUES ($1, $2, $3, $4)
         RETURNING id",
    )
    .bind(auth.id)
    .bind(req.lender_profile_id)
    .bind(req.metrics_id)
    .bind(req.amount_requested)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!({ "application_id": id, "status": "pending" })))
}

/// GET /applications/mine — borrower's own applications with lender name
pub async fn list_mine(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<Value>>> {
    if auth.role != "borrower" {
        return Err(AppError::Unauthorized("Borrower role required".into()));
    }

    // (id, status, decision_reason, amount_requested, created_at, decided_at, lender_name)
    let rows: Vec<(Uuid, String, Option<String>, Option<i64>, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>, String)> =
        sqlx::query_as(
            "SELECT la.id, la.status, la.decision_reason, la.amount_requested,
                    la.created_at, la.decided_at, lp.display_name
             FROM loan_applications la
             JOIN lender_profiles lp ON lp.id = la.lender_profile_id
             WHERE la.borrower_id = $1
             ORDER BY la.created_at DESC",
        )
        .bind(auth.id)
        .fetch_all(&state.db)
        .await?;

    let out = rows
        .into_iter()
        .map(|(id, status, reason, amount, created_at, decided_at, lender_name)| {
            json!({
                "id": id,
                "status": status,
                "decision_reason": reason,
                "amount_requested": amount,
                "created_at": created_at,
                "decided_at": decided_at,
                "lender": { "display_name": lender_name },
            })
        })
        .collect();

    Ok(Json(out))
}

/// GET /applications/lender — lender's incoming applications (no financial data)
pub async fn list_for_lender(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<Value>>> {
    if auth.role != "lender" {
        return Err(AppError::Unauthorized("Lender role required".into()));
    }

    // (id, status, decision_reason, amount_requested, created_at, decided_at, borrower_id)
    let rows: Vec<(Uuid, String, Option<String>, Option<i64>, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>, Uuid)> =
        sqlx::query_as(
            "SELECT la.id, la.status, la.decision_reason, la.amount_requested,
                    la.created_at, la.decided_at, la.borrower_id
             FROM loan_applications la
             JOIN lender_profiles lp ON lp.id = la.lender_profile_id
             WHERE lp.user_id = $1
             ORDER BY la.created_at DESC",
        )
        .bind(auth.id)
        .fetch_all(&state.db)
        .await?;

    let out = rows
        .into_iter()
        .map(|(id, status, reason, amount, created_at, decided_at, borrower_id)| {
            let anon = &borrower_id.to_string()[..8];
            json!({
                "id": id,
                "borrower_ref": format!("Applicant #{}", anon),
                "status": status,
                "decision_reason": reason,
                "amount_requested": amount,
                "created_at": created_at,
                "decided_at": decided_at,
            })
        })
        .collect();

    Ok(Json(out))
}

/// POST /applications/:id/verify — lender triggers ZK proof + loan decision
pub async fn verify(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(app_id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    if auth.role != "lender" {
        return Err(AppError::Unauthorized("Lender role required".into()));
    }

    // (app_id, borrower_id, metrics_id, lender_profile_id, status, borrower_stellar_address)
    let row: Option<(Uuid, Uuid, Uuid, Uuid, String, Option<String>)> = sqlx::query_as(
        "SELECT la.id, la.borrower_id, la.metrics_id, la.lender_profile_id, la.status,
                u.stellar_address
         FROM loan_applications la
         JOIN lender_profiles lp ON lp.id = la.lender_profile_id
         JOIN users u ON u.id = la.borrower_id
         WHERE la.id = $1 AND lp.user_id = $2",
    )
    .bind(app_id)
    .bind(auth.id)
    .fetch_optional(&state.db)
    .await?;

    let (_, borrower_id, metrics_id, lender_profile_id, status, borrower_stellar_address) =
        row.ok_or_else(|| AppError::NotFound("Application not found".into()))?;

    if status != "pending" {
        return Err(AppError::BadRequest("Application already processed".into()));
    }

    let metrics: FinancialMetrics =
        sqlx::query_as("SELECT * FROM financial_metrics WHERE id = $1")
            .bind(metrics_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Metrics not found".into()))?;

    let lender_profile: LenderProfile =
        sqlx::query_as("SELECT * FROM lender_profiles WHERE id = $1")
            .bind(lender_profile_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Lender profile not found".into()))?;

    let policy: crate::models::metrics::LendingPolicy =
        serde_json::from_value(lender_profile.policy).unwrap_or_default();

    let _guard = state.proof_lock.lock().await;

    let circuits_dir = state.config.circuits_dir.clone();
    let proof_id = Uuid::new_v4();
    let metrics_clone = metrics.clone();
    let policy_clone = policy.clone();

    let package = tokio::task::spawn_blocking(move || {
        proof_gen::generate(proof_id, &metrics_clone, &policy_clone, &circuits_dir)
    })
    .await
    .map_err(|e| AppError::ProofGen(e.to_string()))?
    .map_err(|e| AppError::ProofGen(e.to_string()))?;

    sqlx::query(
        "INSERT INTO proofs
         (id, merchant_id, metrics_id, circuit_id, proof_hex, vk_hex, public_inputs, predicates)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(package.proof_id)
    .bind(borrower_id)
    .bind(metrics_id)
    .bind(&package.circuit_id)
    .bind(format!("{}|{}", package.proof_hex, package.pub_inputs_hex))
    .bind(&package.vk_hex)
    .bind(serde_json::to_value(&package.public_inputs).unwrap())
    .bind(serde_json::to_value(&package.predicates).unwrap())
    .execute(&state.db)
    .await?;

    let circuits_dir2 = state.config.circuits_dir.clone();
    let package_clone = package.clone();
    let policy_clone2 = policy.clone();
    let soroban_contract = state.config.soroban_contract_id.clone();
    let stellar_identity = state.config.stellar_identity.clone();
    let stellar_network = state.config.stellar_network.clone();
    let lender_id_str = lender_profile_id.to_string();
    let borrower_addr = borrower_stellar_address.clone();

    let decision = tokio::task::spawn_blocking(move || {
        loan_engine::evaluate(
            app_id,
            &package_clone,
            &policy_clone2,
            &circuits_dir2,
            &lender_id_str,
            borrower_addr.as_deref(),
            &loan_engine::SorobanConfig {
                contract_id: &soroban_contract,
                identity: &stellar_identity,
                network: &stellar_network,
            },
        )
    })
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?
    .map_err(AppError::Internal)?;

    sqlx::query(
        "UPDATE loan_applications
         SET status = $1, proof_id = $2, decision_reason = $3,
             decided_at = NOW(), disbursement_tx_hash = $5
         WHERE id = $4",
    )
    .bind(&decision.decision)
    .bind(package.proof_id)
    .bind(&decision.reason)
    .bind(app_id)
    .bind(&decision.disbursement_tx_hash)
    .execute(&state.db)
    .await?;

    let proof_bytes_len = package.proof_hex.len() / 2;
    let proof_hash_preview = &package.proof_hex[..package.proof_hex.len().min(64)];
    let vk_hash_preview = &package.vk_hex[..package.vk_hex.len().min(32)];

    let stellar_explorer = decision.stellar_tx_hash.as_deref().map(|h| {
        format!("https://stellar.expert/explorer/{}/tx/{}", state.config.stellar_network, h)
    });

    let disbursement = if decision.disbursement_tx_hash.is_some() {
        let dtx = decision.disbursement_tx_hash.as_deref().unwrap_or("");
        json!({
            "tx_hash": dtx,
            "explorer_url": format!("https://stellar.expert/explorer/{}/tx/{}", state.config.stellar_network, dtx),
            "recipient": borrower_stellar_address,
        })
    } else {
        json!(null)
    };

    Ok(Json(json!({
        "application_id": app_id,
        "status": decision.decision,
        "decision_reason": decision.reason,
        "proof_verified": decision.proof_verified,
        "predicates": package.predicates,
        "stellar": {
            "tx_hash": decision.stellar_tx_hash,
            "explorer_url": stellar_explorer,
            "contract_id": state.config.soroban_contract_id,
            "network": state.config.stellar_network,
        },
        "disbursement": disbursement,
        "proof": {
            "id": package.proof_id,
            "circuit_id": package.circuit_id,
            "proof_hash": proof_hash_preview,
            "vk_hash": vk_hash_preview,
            "proof_size_bytes": proof_bytes_len,
            "pub_inputs_hex": package.pub_inputs_hex,
            "public_inputs": package.public_inputs,
        }
    })))
}
