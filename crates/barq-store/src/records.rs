use barq_types::{BarqError, BarqRecord, BarqResult};
use uuid::Uuid;

use crate::{
    cf::{CF_PAYLOADS, CF_RECORDS},
    store::BarqStore,
};

impl BarqStore {
    /// Persist a record's metadata (without payload bytes or raw embedding).
    pub fn put_record(&self, record: &BarqRecord) -> BarqResult<()> {
        // Strip heap-heavy fields before serialization
        let mut stripped = record.clone();
        stripped.compressed_payload = None;
        stripped.embedding = Vec::new();

        let key = record.id.as_bytes().to_vec();
        let value = serde_json::to_vec(&stripped)
            .map_err(|e| BarqError::Storage(format!("Serialize record: {}", e)))?;

        self.db
            .put_cf(self.cf(CF_RECORDS), &key, &value)
            .map_err(|e| BarqError::Storage(format!("put_record: {}", e)))
    }

    /// Retrieve a record by ID.
    pub fn get_record(&self, id: Uuid) -> BarqResult<Option<BarqRecord>> {
        let key = id.as_bytes().to_vec();
        match self.db.get_cf(self.cf(CF_RECORDS), &key) {
            Ok(Some(bytes)) => {
                let record: BarqRecord = serde_json::from_slice(&bytes)
                    .map_err(|e| BarqError::Storage(format!("Deserialize record: {}", e)))?;
                Ok(Some(record))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(BarqError::Storage(format!("get_record: {}", e))),
        }
    }

    /// Delete a record and its associated payload.
    pub fn delete_record(&self, id: Uuid) -> BarqResult<()> {
        let key = id.as_bytes().to_vec();
        self.db
            .delete_cf(self.cf(CF_RECORDS), &key)
            .map_err(|e| BarqError::Storage(format!("delete_record: {}", e)))?;
        // Also remove payload if present
        let _ = self
            .db
            .delete_cf(self.cf(CF_PAYLOADS), &key);
        Ok(())
    }

    /// Iterate over all records in CF_RECORDS for index bootstrap.
    pub fn iter_all_records(&self) -> impl Iterator<Item = BarqRecord> + '_ {
        self.db
            .iterator_cf(self.cf(CF_RECORDS), rocksdb::IteratorMode::Start)
            .filter_map(|result| {
                result.ok().and_then(|(_k, v)| {
                    serde_json::from_slice::<BarqRecord>(&v).ok()
                })
            })
    }

    /// Check if a record with the given ID exists.
    pub fn record_exists(&self, id: Uuid) -> bool {
        let key = id.as_bytes().to_vec();
        self.db
            .get_cf(self.cf(CF_RECORDS), &key)
            .map(|v| v.is_some())
            .unwrap_or(false)
    }
}
