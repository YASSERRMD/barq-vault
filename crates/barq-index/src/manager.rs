use std::sync::Arc;

use barq_types::{BarqRecord, BarqResult};
use tracing::{info, debug};
use uuid::Uuid;

use barq_store::BarqStore;

use crate::hybrid::{HybridEngine, SearchParams};

/// Manages the in-memory hybrid index and coordinates with the store.
pub struct IndexManager {
    engine: HybridEngine,
    store: Arc<BarqStore>,
}

impl IndexManager {
    pub fn new(engine: HybridEngine, store: Arc<BarqStore>) -> Self {
        Self { engine, store }
    }

    /// Rebuild all in-memory indexes from RocksDB on server startup.
    pub fn bootstrap(&mut self) -> BarqResult<()> {
        info!("Bootstrapping index from RocksDB...");

        // Collect records first to avoid simultaneous mutable + immutable borrow of self
        let records: Vec<BarqRecord> = self.store.iter_all_records().collect();
        let total = records.len();

        for (i, record) in records.into_iter().enumerate() {
            self.index_new_internal(&record);
            if (i + 1) % 1000 == 0 {
                debug!("Bootstrapped {} records...", i + 1);
            }
        }

        info!("Bootstrap complete: indexed {} records", total);
        Ok(())
    }

    /// Index a newly ingested record into all three sub-indexes.
    pub fn index_new(&mut self, record: &BarqRecord) {
        self.index_new_internal(record);
    }

    /// Remove a record from all three sub-indexes.
    pub fn remove(&mut self, id: Uuid, record: &BarqRecord) {
        self.engine.bm25.remove_document(id);
        self.engine.vector.remove(id);
        self.engine.meta.remove_record(id, record);
    }

    /// Run a hybrid search.
    pub fn search(&self, params: SearchParams) -> BarqResult<Vec<(Uuid, f32)>> {
        self.engine.search(params)
    }

    fn index_new_internal(&mut self, record: &BarqRecord) {
        self.engine
            .bm25
            .index_document(record.id, &record.bm25_tokens);

        if !record.embedding.is_empty() {
            let _ = self.engine.vector.upsert(record.id, record.embedding.clone());
        }

        self.engine.meta.index_record(record);
    }
}
