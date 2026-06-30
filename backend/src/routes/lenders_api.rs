use crate::{
    error::{AppError, AppResult},
    models::lender::{LenderProfile, UpsertProfileRequest},
    models::metrics::LendingPolicy,
    routes::AuthUser,
    services::soroban,
    AppState,
};
use axum::{extract::State, Json};
use serde_json::{json, Value};

/// GET /lenders — public list of published profiles
pub async fn list_published(State(state): State<AppState>) -> AppResult<Json<Vec<Value>>> {
    let profiles: Vec<LenderProfile> = sqlx::query_as(
        "SELECT * FROM lender_profiles WHERE published = TRUE ORDER BY updated_at DESC",
    )
    .fetch_all(&state.db)
    .await?;

    let out: Vec<Value> = profiles
        .into_iter()
        .map(|p| {
            json!({
                "id": p.id,
                "display_name": p.display_name,
                "description": p.description,
                "policy": p.policy,
                "created_at": p.created_at,
            })
        })
        .collect();

    Ok(Json(out))
}

/// GET /lenders/me — lender's own profile
pub async fn get_my_profile(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Value>> {
    require_lender(&auth)?;

    let profile: Option<LenderProfile> =
        sqlx::query_as("SELECT * FROM lender_profiles WHERE user_id = $1")
            .bind(auth.id)
            .fetch_optional(&state.db)
            .await?;

    let profile = profile.ok_or_else(|| AppError::NotFound("Profile not found".into()))?;

    let stellar = profile.stellar_policy_tx.as_ref().map(|tx| {
        json!({
            "tx_hash": tx,
            "explorer_url": format!(
                "https://stellar.expert/explorer/{}/tx/{}",
                state.config.stellar_network, tx
            ),
            "contract_id": state.config.soroban_contract_id,
            "network": state.config.stellar_network,
        })
    });

    Ok(Json(json!({
        "id": profile.id,
        "display_name": profile.display_name,
        "description": profile.description,
        "policy": profile.policy,
        "published": profile.published,
        "stellar_policy_tx": profile.stellar_policy_tx,
        "loan_amount_stroops": profile.loan_amount_stroops,
        "stellar": stellar,
        "created_at": profile.created_at,
        "updated_at": profile.updated_at,
    })))
}

/// POST /lenders/me — create or update profile + policy
pub async fn upsert_profile(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpsertProfileRequest>,
) -> AppResult<Json<Value>> {
    require_lender(&auth)?;

    let publishing = req.published.unwrap_or(false);

    // When publishing: store policy on-chain AND set loan config on-chain
    let stellar_tx: Option<String> = if publishing {
        if let Some(policy_val) = &req.policy {
            if let Ok(lp) = serde_json::from_value::<LendingPolicy>(policy_val.clone()) {
                // Publish policy criteria on-chain
                let policy_tx = match soroban::publish_policy_on_chain(
                    &auth.id.to_string(),
                    &lp,
                    &state.config.soroban_contract_id,
                    &state.config.stellar_identity,
                    &state.config.stellar_network,
                ) {
                    Ok(tx) => Some(tx),
                    Err(e) => {
                        tracing::warn!("Soroban publish_policy failed (non-fatal): {e}");
                        None
                    }
                };

                // Store loan disbursement amount on-chain
                let loan_amount = req.loan_amount_stroops.unwrap_or(20_000_000); // default 2 XLM
                if let Err(e) = soroban::set_loan_config_on_chain(
                    &auth.id.to_string(),
                    loan_amount,
                    &state.config.soroban_contract_id,
                    &state.config.stellar_identity,
                    &state.config.stellar_network,
                ) {
                    tracing::warn!("Soroban set_loan_config failed (non-fatal): {e}");
                }

                policy_tx
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let loan_amount = req.loan_amount_stroops.unwrap_or(20_000_000);

    let profile: LenderProfile = sqlx::query_as(
        r#"INSERT INTO lender_profiles
               (user_id, display_name, description, policy, published, stellar_policy_tx, loan_amount_stroops)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           ON CONFLICT (user_id) DO UPDATE SET
               display_name       = COALESCE(EXCLUDED.display_name,      lender_profiles.display_name),
               description        = COALESCE(EXCLUDED.description,        lender_profiles.description),
               policy             = COALESCE(EXCLUDED.policy,             lender_profiles.policy),
               published          = COALESCE(EXCLUDED.published,          lender_profiles.published),
               stellar_policy_tx  = COALESCE(EXCLUDED.stellar_policy_tx,  lender_profiles.stellar_policy_tx),
               loan_amount_stroops = EXCLUDED.loan_amount_stroops,
               updated_at         = NOW()
           RETURNING *"#,
    )
    .bind(auth.id)
    .bind(req.display_name.as_deref().unwrap_or(""))
    .bind(req.description.as_deref().unwrap_or(""))
    .bind(req.policy.as_ref().unwrap_or(&serde_json::Value::Object(Default::default())))
    .bind(publishing)
    .bind(stellar_tx.as_deref())
    .bind(loan_amount)
    .fetch_one(&state.db)
    .await?;

    let stellar = profile.stellar_policy_tx.as_ref().map(|tx| {
        json!({
            "tx_hash": tx,
            "explorer_url": format!(
                "https://stellar.expert/explorer/{}/tx/{}",
                state.config.stellar_network, tx
            ),
            "contract_id": state.config.soroban_contract_id,
            "network": state.config.stellar_network,
        })
    });

    Ok(Json(json!({
        "id": profile.id,
        "display_name": profile.display_name,
        "description": profile.description,
        "policy": profile.policy,
        "published": profile.published,
        "stellar_policy_tx": profile.stellar_policy_tx,
        "loan_amount_stroops": profile.loan_amount_stroops,
        "stellar": stellar,
        "created_at": profile.created_at,
        "updated_at": profile.updated_at,
    })))
}

/// POST /lenders/me/publish — toggle published flag
pub async fn toggle_publish(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Value>> {
    require_lender(&auth)?;

    let published: bool = sqlx::query_scalar(
        "UPDATE lender_profiles
         SET published = NOT published, updated_at = NOW()
         WHERE user_id = $1
         RETURNING published",
    )
    .bind(auth.id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!({ "published": published })))
}

fn require_lender(auth: &AuthUser) -> AppResult<()> {
    if auth.role != "lender" {
        Err(AppError::Unauthorized("Lender role required".into()))
    } else {
        Ok(())
    }
}
