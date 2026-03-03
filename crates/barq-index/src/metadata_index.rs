use std::collections::HashMap;

use barq_types::BarqRecord;
use uuid::Uuid;

/// In-memory metadata index for fast filtering by modality, tags, and filename.
pub struct MetadataIndex {
    /// key → value → doc_ids
    key_value_map: HashMap<String, HashMap<String, Vec<Uuid>>>,
    /// modality string → doc_ids
    modality_map: HashMap<String, Vec<Uuid>>,
    /// original filename → doc_id
    filename_map: HashMap<String, Uuid>,
}

impl MetadataIndex {
    pub fn new() -> Self {
        Self {
            key_value_map: HashMap::new(),
            modality_map: HashMap::new(),
            filename_map: HashMap::new(),
        }
    }

    /// Index all indexable fields of a record.
    pub fn index_record(&mut self, record: &BarqRecord) {
        let id = record.id;

        // Index modality
        self.modality_map
            .entry(record.modality.to_string())
            .or_default()
            .push(id);

        // Index filename
        if let Some(ref fname) = record.filename {
            self.filename_map.insert(fname.clone(), id);
        }

        // Index flat string fields in metadata JSON
        if let serde_json::Value::Object(ref map) = record.metadata {
            for (k, v) in map {
                if let Some(val_str) = v.as_str() {
                    self.key_value_map
                        .entry(k.clone())
                        .or_default()
                        .entry(val_str.to_string())
                        .or_default()
                        .push(id);
                }
            }
        }
    }

    /// Remove all indexed entries for a record.
    pub fn remove_record(&mut self, id: Uuid, record: &BarqRecord) {
        let mod_str = record.modality.to_string();
        if let Some(list) = self.modality_map.get_mut(&mod_str) {
            list.retain(|&x| x != id);
        }

        if let Some(ref fname) = record.filename {
            self.filename_map.remove(fname);
        }

        if let serde_json::Value::Object(ref map) = record.metadata {
            for (k, v) in map {
                if let Some(val_str) = v.as_str() {
                    if let Some(kv) = self.key_value_map.get_mut(k) {
                        if let Some(list) = kv.get_mut(val_str) {
                            list.retain(|&x| x != id);
                        }
                    }
                }
            }
        }
    }

    /// Return all doc IDs with the given modality string.
    pub fn filter_by_modality(&self, modality: &str) -> Vec<Uuid> {
        self.modality_map
            .get(modality)
            .cloned()
            .unwrap_or_default()
    }

    /// Return all doc IDs where metadata[key] == value.
    pub fn filter_by_meta_key_value(&self, key: &str, value: &str) -> Vec<Uuid> {
        self.key_value_map
            .get(key)
            .and_then(|m| m.get(value))
            .cloned()
            .unwrap_or_default()
    }
}

impl Default for MetadataIndex {
    fn default() -> Self {
        Self::new()
    }
}
