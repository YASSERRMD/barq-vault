use std::sync::Arc;
use std::time::Instant;

use barq_types::BarqResult;
use tokio::sync::RwLock;
use tracing::info;

use barq_compress::select_codec_for_modality;
use barq_index::{HybridEngine, IndexManager};
use barq_ingest::{IngestConfig, IngestPipeline};
use barq_store::BarqStore;

use crate::config::ServerConfig;

/// Shared application state passed to every gRPC and REST handler.
pub struct AppState {
    pub store: Arc<BarqStore>,
    pub index: Arc<RwLock<IndexManager>>,
    pub ingest_pipeline: Arc<IngestPipeline>,
    pub config: Arc<ServerConfig>,
    pub started_at: Instant,
}

impl AppState {
    /// Initialize app state: open store, bootstrap indexes, prepare pipeline.
    pub async fn init(config: ServerConfig) -> BarqResult<Arc<Self>> {
        info!("Opening store at: {}", config.server.store_path);
        let store = Arc::new(BarqStore::open(&config.server.store_path)?);

        let engine = HybridEngine::new(
            config.index.bm25_k1,
            config.index.bm25_b,
            config.index.vector_dim,
        );
        let mut index_manager = IndexManager::new(engine, Arc::clone(&store));

        info!("Bootstrapping in-memory indexes...");
        let t0 = Instant::now();
        index_manager.bootstrap()?;
        info!("Bootstrap took {:?}", t0.elapsed());

        let ingest_pipeline = Arc::new(IngestPipeline::new(IngestConfig::default()));
        let index = Arc::new(RwLock::new(index_manager));

        Ok(Arc::new(Self {
            store,
            index,
            ingest_pipeline,
            config: Arc::new(config),
            started_at: Instant::now(),
        }))
    }
}
