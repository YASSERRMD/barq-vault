use std::sync::Arc;

use rocksdb::{ColumnFamily, DB};

use barq_types::{BarqError, BarqResult};

use crate::cf::{build_cf_descriptors, build_db_options};

/// barq-vault RocksDB storage handle, cheaply cloneable via Arc.
#[derive(Clone)]
pub struct BarqStore {
    pub(crate) db: Arc<DB>,
}

impl BarqStore {
    /// Open (or create) the database at `path` with all column families.
    pub fn open(path: &str) -> BarqResult<Self> {
        let opts = build_db_options();
        let cf_descriptors = build_cf_descriptors();

        let db = DB::open_cf_descriptors(&opts, path, cf_descriptors)
            .map_err(|e| BarqError::Storage(format!("Failed to open RocksDB at {}: {}", path, e)))?;

        Ok(Self { db: Arc::new(db) })
    }

    /// Retrieve a handle to a named column family.
    ///
    /// # Panics
    /// Panics if the column family name is unknown (misconfiguration).
    pub fn cf(&self, name: &str) -> &ColumnFamily {
        self.db
            .cf_handle(name)
            .unwrap_or_else(|| panic!("Column family '{}' not found in BarqStore", name))
    }
}
