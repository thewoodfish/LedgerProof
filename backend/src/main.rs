mod config;
mod db;
mod error;
mod models;
mod routes;
mod services;

use config::Config;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<Config>,
    /// Prevents concurrent ZK proof generation from corrupting shared circuit output files.
    pub proof_lock: Arc<Mutex<()>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ledgerproof_backend=debug,tower_http=info".into()),
        )
        .init();

    let config = Config::from_env()?;
    tracing::info!("Connecting to database...");
    let pool = db::create_pool(&config.database_url).await?;

    tracing::info!("Running migrations...");
    db::run_migrations(&pool).await?;

    let state = AppState {
        db: pool,
        config: config.clone(),
        proof_lock: Arc::new(Mutex::new(())),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = routes::build(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("LedgerProof backend listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
