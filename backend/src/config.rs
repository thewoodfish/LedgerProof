use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub openai_api_key: String,
    pub circuits_dir: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Arc<Self>> {
        dotenvy::dotenv().ok();
        Ok(Arc::new(Self {
            database_url: required("DATABASE_URL")?,
            openai_api_key: required("OPENAI_API_KEY")?,
            circuits_dir: std::env::var("CIRCUITS_DIR")
                .unwrap_or_else(|_| "../circuits/lending".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3001".to_string())
                .parse()?,
        }))
    }
}

fn required(key: &str) -> anyhow::Result<String> {
    std::env::var(key).map_err(|_| anyhow::anyhow!("missing env var: {key}"))
}
