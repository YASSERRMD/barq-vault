use barq_types::{BarqError, BarqRecord, IngestRequest, Modality, SearchRequest, SearchResult};
use uuid::Uuid;

use crate::barqvault::v1::{
    ProtoIngestRequest, ProtoSearchRequest, ProtoSearchResult,
};

// ---- IngestRequest → ProtoIngestRequest ----

impl From<IngestRequest> for ProtoIngestRequest {
    fn from(req: IngestRequest) -> Self {
        // Serialize embedding as little-endian f32 bytes
        let emb_bytes: Vec<u8> = req
            .embedding
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();
        let emb_dim = req.embedding.len() as u32;

        Self {
            summary: req.summary,
            embedding: emb_bytes,
            embedding_dim: emb_dim,
            modality: req.modality.to_string(),
            storage_mode: req.storage_mode.to_string(),
            raw_payload: req.raw_payload.unwrap_or_default(),
            filename: req.filename.unwrap_or_default(),
            metadata_json: req.metadata.to_string(),
            chunk_index: req.chunk_index,
            total_chunks: req.total_chunks,
            parent_id: req.parent_id.map(|u| u.to_string()).unwrap_or_default(),
        }
    }
}

impl TryFrom<ProtoIngestRequest> for IngestRequest {
    type Error = BarqError;

    fn try_from(proto: ProtoIngestRequest) -> Result<Self, Self::Error> {
        // Validate embedding byte length is divisible by 4
        if proto.embedding.len() % 4 != 0 {
            return Err(BarqError::InvalidInput(
                "Embedding byte length must be divisible by 4".to_string(),
            ));
        }
        let expected_len = proto.embedding_dim as usize;
        let actual_len = proto.embedding.len() / 4;
        if actual_len != expected_len && expected_len != 0 {
            return Err(BarqError::InvalidInput(format!(
                "embedding_dim {} does not match byte len {}",
                expected_len, actual_len
            )));
        }

        let embedding: Vec<f32> = proto
            .embedding
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();

        let modality: Modality = proto.modality.parse().map_err(BarqError::InvalidInput)?;
        let storage_mode = proto.storage_mode.parse().map_err(BarqError::InvalidInput)?;

        let metadata: serde_json::Value = if proto.metadata_json.is_empty() {
            serde_json::Value::Object(Default::default())
        } else {
            serde_json::from_str(&proto.metadata_json)?
        };

        let parent_id = if proto.parent_id.is_empty() {
            None
        } else {
            Some(
                Uuid::parse_str(&proto.parent_id)
                    .map_err(|e| BarqError::InvalidInput(format!("parent_id: {}", e)))?,
            )
        };

        Ok(IngestRequest {
            summary: proto.summary,
            embedding,
            modality,
            storage_mode,
            filename: if proto.filename.is_empty() {
                None
            } else {
                Some(proto.filename)
            },
            raw_payload: if proto.raw_payload.is_empty() {
                None
            } else {
                Some(proto.raw_payload)
            },
            metadata,
            chunk_index: proto.chunk_index,
            total_chunks: proto.total_chunks,
            parent_id,
        })
    }
}

// ---- SearchRequest → ProtoSearchRequest ----

impl From<SearchRequest> for ProtoSearchRequest {
    fn from(req: SearchRequest) -> Self {
        let emb_bytes: Vec<u8> = req
            .query_embedding
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();
        let emb_dim = req.query_embedding.len() as u32;

        Self {
            query_embedding: emb_bytes,
            embedding_dim: emb_dim,
            query_text: req.query_text,
            vector_weight: req.vector_weight,
            top_k: req.top_k as u32,
            modality_filter: req
                .modality_filter
                .map(|m| m.to_string())
                .unwrap_or_default(),
            metadata_filter_json: req.metadata_filters.to_string(),
        }
    }
}

impl TryFrom<ProtoSearchRequest> for SearchRequest {
    type Error = BarqError;

    fn try_from(proto: ProtoSearchRequest) -> Result<Self, Self::Error> {
        if proto.query_embedding.len() % 4 != 0 {
            return Err(BarqError::InvalidInput(
                "Query embedding byte length must be divisible by 4".to_string(),
            ));
        }
        let query_embedding: Vec<f32> = proto
            .query_embedding
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();

        let modality_filter = if proto.modality_filter.is_empty() {
            None
        } else {
            Some(proto.modality_filter.parse().map_err(BarqError::InvalidInput)?)
        };

        let metadata_filters: serde_json::Value = if proto.metadata_filter_json.is_empty() {
            serde_json::Value::Object(Default::default())
        } else {
            serde_json::from_str(&proto.metadata_filter_json)?
        };

        Ok(SearchRequest {
            query_embedding,
            query_text: proto.query_text,
            vector_weight: proto.vector_weight,
            top_k: proto.top_k as usize,
            modality_filter,
            metadata_filters,
        })
    }
}

// ---- BarqRecord → ProtoSearchResult ----

impl From<BarqRecord> for ProtoSearchResult {
    fn from(record: BarqRecord) -> Self {
        Self {
            id: record.id.to_string(),
            summary: record.summary,
            filename: record.filename.unwrap_or_default(),
            modality: record.modality.to_string(),
            score: 0.0, // caller fills in actual score
            has_payload: record.compressed_payload.is_some(),
            metadata_json: record.metadata.to_string(),
        }
    }
}
