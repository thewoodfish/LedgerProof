use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub circuits_dir: String,
    pub jwt_secret: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Arc<Self>> {
        dotenvy::dotenv().ok();
        Ok(Arc::new(Self {
            database_url: required("DATABASE_URL")?,
            circuits_dir: std::env::var("CIRCUITS_DIR")
                .unwrap_or_else(|_| "../circuits/lending".to_string()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "ledgerproof-dev-secret-2026-change-in-prod".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3001".to_string())
                .parse()?,
        }))
    }
}

fn required(key: &str) -> anyhow::Result<String> {
    std::env::var(key).map_err(|_| anyhow::anyhow!("missing env var: {key}"))
}
