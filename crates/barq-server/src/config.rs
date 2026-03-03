use barq_types::{BarqError, BarqResult};
use config::{Config, File, Environment};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub enabled: bool,
    pub cert_path: String,
    pub key_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub enabled: bool,
    pub token: String,
    pub skip_ping: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    pub vector_dim: usize,
    pub hnsw_ef_construction: usize,
    pub hnsw_m: usize,
    pub bm25_k1: f64,
    pub bm25_b: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestCfg {
    pub chunk_size_tokens: usize,
    pub chunk_overlap_tokens: usize,
    pub default_storage_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEndpointConfig {
    pub grpc_addr: String,
    pub rest_addr: String,
    pub store_path: String,
    pub max_payload_bytes: u64,
    pub tls: TlsConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub server: ServerEndpointConfig,
    pub index: IndexConfig,
    pub ingest: IngestCfg,
}

impl ServerConfig {
    /// Load config from default.toml → production.toml (if BARQ_ENV=production) → env vars.
    pub fn load() -> BarqResult<Self> {
        let barq_env = std::env::var("BARQ_ENV").unwrap_or_else(|_| "development".to_string());

        let cfg = Config::builder()
            .add_source(File::with_name("config/default").required(true))
            .add_source(
                File::with_name("config/production")
                    .required(barq_env == "production"),
            )
            .add_source(Environment::with_prefix("BARQ").separator("__"))
            .build()
            .map_err(|e| BarqError::InvalidInput(format!("Config load error: {}", e)))?;

        cfg.try_deserialize::<ServerConfig>()
            .map_err(|e| BarqError::InvalidInput(format!("Config deserialize error: {}", e)))
    }
}
