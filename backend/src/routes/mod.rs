mod loans;
mod metrics;
mod proofs;
mod statements;
mod transactions;

use crate::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub fn build(state: AppState) -> Router {
    Router::new()
        // Statement ingestion
        .route("/upload-statement", post(statements::upload))
        .route("/parse/{statement_id}", post(statements::parse))
        // Transactions
        .route("/transactions", get(transactions::list))
        .route("/transactions/{id}", get(transactions::get_one))
        // Metrics
        .route("/metrics", post(metrics::compute))
        .route("/metrics/{id}", get(metrics::get_one))
        .route("/metrics/merchant/{merchant_id}", get(metrics::latest_for_merchant))
        // Proofs
        .route("/generate-proof", post(proofs::generate))
        .route("/verify-proof", post(proofs::verify))
        .route("/proofs/{id}", get(proofs::get_one))
        // Loans
        .route("/loan/evaluate", post(loans::evaluate))
        .route("/loan/{id}", get(loans::get_one))
        // Health
        .route("/health", get(|| async { "ok" }))
        .with_state(state)
}
