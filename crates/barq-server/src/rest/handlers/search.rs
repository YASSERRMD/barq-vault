use std::sync::Arc;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use barq_types::Modality;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub query_text: String,
    pub top_k: usize,
    pub modality_filter: Option<Modality>,
    #[serde(default)]
    pub metadata_filters: serde_json::Value,
    #[serde(default = "default_vector_weight")]
    pub vector_weight: f32,
}

fn default_vector_weight() -> f32 {
    0.5
}

#[derive(Serialize)]
pub struct SearchHit {
    pub id: String,
    pub summary: String,
    pub filename: String,
    pub modality: String,
    pub score: f32,
    pub has_payload: bool,
    pub metadata: serde_json::Value,
}

#[derive(Serialize)]
pub struct SearchResponseJson {
    pub total_found: u32,
    pub results: Vec<SearchHit>,
    pub took_ms: u64,
}

pub async fn search_handler(
    State(state): State<Arc<AppState>>,
    Json(query): Json<SearchQuery>,
) -> impl IntoResponse {
    let t0 = std::time::Instant::now();

    // In a real implementation this would generate the embedding.
    // Assuming vector generation is part of the search engine or we proxy to one here.
    // For MVP REST, we will just use dummy vector or empty since it's an embedding generation step.
    // Better: use the embedder from ingest pipeline
    let embed_config = &state.config.ingest.embed;
    let query_embedding = match barq_ingest::embed(&query.query_text, embed_config).await {
        Ok(emb) => emb,
        Err(_) => vec![0.0; embed_config.expected_dim], // Fallback to BM25 if embed fails
    };

    let search_params = barq_index::SearchParams {
        query_embedding,
        query_text: query.query_text,
        vector_weight: query.vector_weight,
        top_k: query.top_k,
        modality_filter: query.modality_filter.map(|m| m.to_string()),
        metadata_filters: query.metadata_filters,
    };

    let index = state.index.read().await;
    let results = match index.search(search_params) {
        Ok(res) => res,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    };

    let mut out_results = Vec::new();
    for (id, score) in results {
        if let Ok(Some(record)) = state.store.get_record(id) {
            out_results.push(SearchHit {
                id: record.id.to_string(),
                summary: record.summary,
                filename: record.filename.unwrap_or_default(),
                modality: record.modality.to_string(),
                score,
                has_payload: record.compressed_payload.is_some(),
                metadata: record.metadata,
            });
        }
    }

    let took_ms = t0.elapsed().as_millis() as u64;

    (StatusCode::OK, Json(SearchResponseJson {
        total_found: out_results.len() as u32,
        results: out_results,
        took_ms,
    })).into_response()
}
