mod applications;
mod auth;
mod lenders_api;
mod loans;
mod metrics;
mod proofs;
mod statements;
mod transactions;

use crate::{
    error::AppError,
    models::user::AuthClaims,
    AppState,
};
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, header::AUTHORIZATION},
    routing::{get, post},
    Router,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use uuid::Uuid;

/// Authenticated user extracted from JWT Bearer token.
pub struct AuthUser {
    pub id: Uuid,
    pub role: String,
    pub username: String,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".into()))?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("Use Bearer token".into()))?;

        let claims = decode::<AuthClaims>(
            token,
            &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| AppError::Unauthorized("Invalid or expired token".into()))?
        .claims;

        let id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Unauthorized("Malformed token sub".into()))?;

        Ok(AuthUser {
            id,
            role: claims.role,
            username: claims.username,
        })
    }
}

pub fn build(state: AppState) -> Router {
    Router::new()
        // ── Auth (public) ──────────────────────────────────────────────────
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        // ── Lender profiles ────────────────────────────────────────────────
        .route("/lenders", get(lenders_api::list_published))
        .route("/lenders/me", get(lenders_api::get_my_profile).post(lenders_api::upsert_profile))
        .route("/lenders/me/publish", post(lenders_api::toggle_publish))
        // ── Applications ───────────────────────────────────────────────────
        .route("/applications", post(applications::create))
        .route("/applications/mine", get(applications::list_mine))
        .route("/applications/lender", get(applications::list_for_lender))
        .route("/applications/{id}/verify", post(applications::verify))
        // ── Statements ─────────────────────────────────────────────────────
        .route("/upload-statement", post(statements::upload))
        .route("/parse/{statement_id}", post(statements::parse))
        // ── Transactions ───────────────────────────────────────────────────
        .route("/transactions", get(transactions::list))
        .route("/transactions/{id}", get(transactions::get_one))
        // ── Metrics (/metrics/latest before /metrics/{id} — static beats dynamic) ──
        .route("/metrics/latest", get(metrics::latest_for_current_user))
        .route("/metrics", post(metrics::compute))
        .route("/metrics/{id}", get(metrics::get_one))
        // ── Proofs ─────────────────────────────────────────────────────────
        .route("/generate-proof", post(proofs::generate))
        .route("/verify-proof", post(proofs::verify))
        .route("/proofs/{id}", get(proofs::get_one))
        // ── Loans (legacy evaluate endpoint kept for testing) ──────────────
        .route("/loan/evaluate", post(loans::evaluate))
        .route("/loan/{id}", get(loans::get_one))
        // ── Health ─────────────────────────────────────────────────────────
        .route("/health", get(|| async { "ok" }))
        .with_state(state)
}
