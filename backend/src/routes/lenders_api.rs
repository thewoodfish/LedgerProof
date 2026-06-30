use crate::{
    error::{AppError, AppResult},
    models::lender::{LenderProfile, UpsertProfileRequest},
    routes::AuthUser,
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
) -> AppResult<Json<LenderProfile>> {
    require_lender(&auth)?;

    let profile: Option<LenderProfile> =
        sqlx::query_as("SELECT * FROM lender_profiles WHERE user_id = $1")
            .bind(auth.id)
            .fetch_optional(&state.db)
            .await?;

    profile
        .map(Json)
        .ok_or_else(|| AppError::NotFound("Profile not found".into()))
}

/// POST /lenders/me — create or update profile + policy
pub async fn upsert_profile(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpsertProfileRequest>,
) -> AppResult<Json<LenderProfile>> {
    require_lender(&auth)?;

    let profile: LenderProfile = sqlx::query_as(
        r#"INSERT INTO lender_profiles (user_id, display_name, description, policy, published)
           VALUES ($1, $2, $3, $4, $5)
           ON CONFLICT (user_id) DO UPDATE SET
               display_name = COALESCE(EXCLUDED.display_name, lender_profiles.display_name),
               description  = COALESCE(EXCLUDED.description,  lender_profiles.description),
               policy       = COALESCE(EXCLUDED.policy,       lender_profiles.policy),
               published    = COALESCE(EXCLUDED.published,    lender_profiles.published),
               updated_at   = NOW()
           RETURNING *"#,
    )
    .bind(auth.id)
    .bind(req.display_name.as_deref().unwrap_or(""))
    .bind(req.description.as_deref().unwrap_or(""))
    .bind(req.policy.as_ref().unwrap_or(&serde_json::Value::Object(Default::default())))
    .bind(req.published.unwrap_or(false))
    .fetch_one(&state.db)
    .await?;

    Ok(Json(profile))
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
