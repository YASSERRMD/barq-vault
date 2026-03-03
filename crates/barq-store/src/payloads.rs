use barq_types::{BarqError, BarqResult};
use uuid::Uuid;

use crate::{cf::CF_PAYLOADS, store::BarqStore};

impl BarqStore {
    /// Store compressed payload bytes for a record.
    pub fn put_payload(&self, id: Uuid, data: &[u8]) -> BarqResult<()> {
        let key = id.as_bytes().to_vec();
        self.db
            .put_cf(self.cf(CF_PAYLOADS), &key, data)
            .map_err(|e| BarqError::Storage(format!("put_payload: {}", e)))
    }

    /// Retrieve compressed payload bytes for a record.
    pub fn get_payload(&self, id: Uuid) -> BarqResult<Option<Vec<u8>>> {
        let key = id.as_bytes().to_vec();
        self.db
            .get_cf(self.cf(CF_PAYLOADS), &key)
            .map(|opt| opt.map(|v| v.to_vec()))
            .map_err(|e| BarqError::Storage(format!("get_payload: {}", e)))
    }

    /// Delete payload bytes for a record.
    pub fn delete_payload(&self, id: Uuid) -> BarqResult<()> {
        let key = id.as_bytes().to_vec();
        self.db
            .delete_cf(self.cf(CF_PAYLOADS), &key)
            .map_err(|e| BarqError::Storage(format!("delete_payload: {}", e)))
    }

    /// Return the size of the stored payload without reading its full content.
    pub fn get_payload_size(&self, id: Uuid) -> BarqResult<Option<u64>> {
        let key = id.as_bytes().to_vec();
        match self.db.get_cf(self.cf(CF_PAYLOADS), &key) {
            Ok(Some(bytes)) => Ok(Some(bytes.len() as u64)),
            Ok(None) => Ok(None),
            Err(e) => Err(BarqError::Storage(format!("get_payload_size: {}", e))),
        }
    }
}
