use barq_types::{BarqError, BarqResult};
use uuid::Uuid;

use crate::{cf::CF_METADATA, store::BarqStore};

impl BarqStore {
    /// Store arbitrary JSON metadata for a record.
    pub fn put_metadata(&self, id: Uuid, meta: &serde_json::Value) -> BarqResult<()> {
        let key = id.as_bytes().to_vec();
        let value = serde_json::to_vec(meta)
            .map_err(|e| BarqError::Storage(format!("Serialize metadata: {}", e)))?;
        self.db
            .put_cf(self.cf(CF_METADATA), &key, &value)
            .map_err(|e| BarqError::Storage(format!("put_metadata: {}", e)))
    }

    /// Retrieve JSON metadata for a record.
    pub fn get_metadata(&self, id: Uuid) -> BarqResult<Option<serde_json::Value>> {
        let key = id.as_bytes().to_vec();
        match self.db.get_cf(self.cf(CF_METADATA), &key) {
            Ok(Some(bytes)) => {
                let meta: serde_json::Value = serde_json::from_slice(&bytes)
                    .map_err(|e| BarqError::Storage(format!("Deserialize metadata: {}", e)))?;
                Ok(Some(meta))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(BarqError::Storage(format!("get_metadata: {}", e))),
        }
    }

    /// Full-scan search for records whose metadata contains `key` = `value`.
    ///
    /// Note: This is O(N) and should only be used for bootstrap or rare admin queries.
    /// The in-memory MetadataIndex in barq-index handles high-frequency filter lookups.
    pub fn search_by_metadata_key(&self, key: &str, value: &str) -> BarqResult<Vec<Uuid>> {
        let mut results = Vec::new();

        for item in self
            .db
            .iterator_cf(self.cf(CF_METADATA), rocksdb::IteratorMode::Start)
        {
            let (raw_key, raw_val) = item.map_err(|e| {
                BarqError::Storage(format!("scan_metadata iter: {}", e))
            })?;

            if raw_key.len() == 16 {
                if let Ok(meta) = serde_json::from_slice::<serde_json::Value>(&raw_val) {
                    if meta.get(key).and_then(|v| v.as_str()) == Some(value) {
                        let id_bytes: [u8; 16] = raw_key.as_ref().try_into().unwrap();
                        results.push(Uuid::from_bytes(id_bytes));
                    }
                }
            }
        }

        Ok(results)
    }
}
