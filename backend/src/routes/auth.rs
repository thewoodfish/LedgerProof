use crate::{
    error::{AppError, AppResult},
    models::user::{AuthClaims, LoginRequest, RegisterRequest, User},
    AppState,
};
use axum::{extract::State, Json};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<Value>> {
    if req.username.trim().is_empty() || req.password.len() < 6 {
        return Err(AppError::BadRequest(
            "Username required; password must be at least 6 characters".into(),
        ));
    }
    if !matches!(req.role.as_str(), "borrower" | "lender") {
        return Err(AppError::BadRequest("Role must be 'borrower' or 'lender'".into()));
    }

    let password = req.password.clone();
    let hash = tokio::task::spawn_blocking(move || bcrypt::hash(password, 12))
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;

    let user: User = sqlx::query_as(
        "INSERT INTO users (username, password_hash, role, full_name)
         VALUES ($1, $2, $3, $4)
         RETURNING *",
    )
    .bind(req.username.trim())
    .bind(&hash)
    .bind(&req.role)
    .bind(req.full_name.as_deref().map(str::trim))
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
            AppError::BadRequest("Username already taken".into())
        } else {
            AppError::Database(e)
        }
    })?;

    // Auto-create lender profile with sensible default policy
    if user.role == "lender" {
        let default_policy = serde_json::json!({
            "required_monthly_revenue": 80_000_000,
            "required_avg_balance": 50_000,
            "required_positive_cash_flow_months": 0,
            "max_revenue_volatility_bps": 12_000,
            "max_customer_concentration_bps": 6_000,
            "max_debt_ratio_bps": 9_000,
            "require_no_missed_repayments": false,
            "required_account_age_months": 1
        });
        sqlx::query(
            "INSERT INTO lender_profiles (user_id, policy)
             VALUES ($1, $2)
             ON CONFLICT (user_id) DO NOTHING",
        )
        .bind(user.id)
        .bind(default_policy)
        .execute(&state.db)
        .await?;
    }

    let token = make_token(&user, &state.config.jwt_secret)?;
    Ok(Json(json!({ "token": token, "user": public_user(&user) })))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<Value>> {
    let user: Option<User> =
        sqlx::query_as("SELECT * FROM users WHERE username = $1")
            .bind(req.username.trim())
            .fetch_optional(&state.db)
            .await?;

    let user = user.ok_or_else(|| AppError::Unauthorized("Invalid credentials".into()))?;

    let hash = user.password_hash.clone();
    let password = req.password.clone();
    let valid = tokio::task::spawn_blocking(move || bcrypt::verify(password, &hash))
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;

    if !valid {
        return Err(AppError::Unauthorized("Invalid credentials".into()));
    }

    let token = make_token(&user, &state.config.jwt_secret)?;
    Ok(Json(json!({ "token": token, "user": public_user(&user) })))
}

fn make_token(user: &User, secret: &str) -> AppResult<String> {
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?
        .as_secs() as usize
        + 7 * 86_400; // 7 days

    let claims = AuthClaims {
        sub: user.id.to_string(),
        role: user.role.clone(),
        username: user.username.clone(),
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("JWT encode: {e}")))
}

fn public_user(u: &User) -> serde_json::Value {
    json!({
        "id": u.id,
        "username": u.username,
        "role": u.role,
        "full_name": u.full_name,
    })
}
