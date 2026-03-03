use std::collections::{HashMap, HashSet};

use barq_types::{BarqError, BarqResult};
use uuid::Uuid;

use crate::{
    bm25::Bm25Index,
    metadata_index::MetadataIndex,
    tokenizer::tokenize_query,
    vector::VectorIndex,
};

/// Search parameters for the hybrid engine.
pub struct SearchParams {
    pub query_embedding: Vec<f32>,
    pub query_text: String,
    /// Weight of vector score in RRF (0.0 = pure BM25, 1.0 = pure vector).
    pub vector_weight: f32,
    pub top_k: usize,
    pub modality_filter: Option<String>,
    pub metadata_filters: serde_json::Value,
}

/// Combines BM25, vector, and metadata indexes with Reciprocal Rank Fusion.
pub struct HybridEngine {
    pub bm25: Bm25Index,
    pub vector: VectorIndex,
    pub meta: MetadataIndex,
}

impl HybridEngine {
    pub fn new(k1: f64, b: f64, vector_dim: usize) -> Self {
        Self {
            bm25: Bm25Index::new(k1, b),
            vector: VectorIndex::new(vector_dim),
            meta: MetadataIndex::new(),
        }
    }

    /// Run a hybrid search and return (doc_id, rrf_score) sorted descending.
    pub fn search(&self, params: SearchParams) -> BarqResult<Vec<(Uuid, f32)>> {
        // --- Candidate filtering from metadata ---
        let candidate_set: Option<HashSet<Uuid>> = self.build_candidate_set(&params);

        // --- BM25 scoring ---
        let query_tokens = tokenize_query(&params.query_text);
        let bm25_results = self.bm25.score(&query_tokens, params.top_k * 4);

        // --- Vector scoring ---
        let vector_results = self
            .vector
            .search_cosine(&params.query_embedding, params.top_k * 4);

        // --- RRF fusion (k=60) ---
        let k = 60_f32;
        let w_v = params.vector_weight;
        let w_b = 1.0 - w_v;

        let mut rrf_scores: HashMap<Uuid, f32> = HashMap::new();

        for (rank, (id, _)) in vector_results.iter().enumerate() {
            *rrf_scores.entry(*id).or_default() += w_v / (k + rank as f32 + 1.0);
        }
        for (rank, (id, _)) in bm25_results.iter().enumerate() {
            *rrf_scores.entry(*id).or_default() += w_b / (k + rank as f32 + 1.0);
        }

        // --- Apply candidate filter ---
        let mut ranked: Vec<(Uuid, f32)> = rrf_scores
            .into_iter()
            .filter(|(id, _)| {
                candidate_set
                    .as_ref()
                    .map(|s| s.contains(id))
                    .unwrap_or(true)
            })
            .collect();

        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked.truncate(params.top_k);
        Ok(ranked)
    }

    fn build_candidate_set(&self, params: &SearchParams) -> Option<HashSet<Uuid>> {
        let mut sets: Vec<HashSet<Uuid>> = Vec::new();

        if let Some(ref modality) = params.modality_filter {
            let ids: HashSet<Uuid> = self.meta.filter_by_modality(modality).into_iter().collect();
            sets.push(ids);
        }

        if let serde_json::Value::Object(ref map) = params.metadata_filters {
            for (k, v) in map {
                if let Some(val_str) = v.as_str() {
                    let ids: HashSet<Uuid> = self
                        .meta
                        .filter_by_meta_key_value(k, val_str)
                        .into_iter()
                        .collect();
                    sets.push(ids);
                }
            }
        }

        if sets.is_empty() {
            return None;
        }

        // Intersect all filter sets
        let mut result = sets.remove(0);
        for set in sets {
            result = result.intersection(&set).copied().collect();
        }
        Some(result)
    }
}
