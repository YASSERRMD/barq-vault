use std::sync::Arc;
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::error;
use uuid::Uuid;
use serde_json::json;

use barq_types::{IngestRequest, Modality, StorageMode};
use crate::state::AppState;

pub async fn ingest_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut raw_payload = None;
    let mut filename = None;
    let mut modality_str = String::from("text");
    let mut storage_mode_str = state.config.ingest.default_storage_mode.clone();
    let mut metadata_str = String::from("{}");
    let mut summary_str = String::new();
    let mut chunk_index = 0u32;
    let mut total_chunks = 1u32;
    let mut parent_id_str = String::new();

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "file" {
            filename = field.file_name().map(|s| s.to_string());
            if let Ok(bytes) = field.bytes().await {
                raw_payload = Some(bytes.to_vec());
            }
        } else if let Ok(text) = field.text().await {
            match name.as_str() {
                "modality" => modality_str = text,
                "storage_mode" => storage_mode_str = text,
                "metadata" => metadata_str = text,
                "summary" => summary_str = text,
                "chunk_index" => chunk_index = text.parse().unwrap_or(0),
                "total_chunks" => total_chunks = text.parse().unwrap_or(1),
                "parent_id" => parent_id_str = text,
                _ => {}
            }
        }
    }

    let modality = match modality_str.parse::<Modality>() {
        Ok(m) => m,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid modality"}))),
    };

    let storage_mode = match storage_mode_str.parse::<StorageMode>() {
        Ok(s) => s,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid storage_mode"}))),
    };

    let metadata = match serde_json::from_str(&metadata_str) {
        Ok(v) => v,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid metadata JSON"}))),
    };

    let parent_id = if parent_id_str.is_empty() {
        None
    } else {
        match Uuid::parse_str(&parent_id_str) {
            Ok(id) => Some(id),
            Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid parent_id UUID"}))),
        }
    };

    let req = IngestRequest {
        summary: summary_str,
        embedding: Vec::new(), // Embeddings are generated in the pipeline
        modality,
        storage_mode,
        filename,
        raw_payload,
        metadata,
        chunk_index,
        total_chunks,
        parent_id,
    };

    match state.ingest_pipeline.run(req).await {
        Ok(records) => {
            let mut index = state.index.write().await;
            let mut first_id = Uuid::nil();
            let mut total_compressed = 0u64;

            for record in records {
                if first_id.is_nil() {
                    first_id = record.id;
                }

                if let Some(payload) = &record.compressed_payload {
                    if let Err(e) = state.store.put_payload(record.id, payload) {
                        error!("Failed to store payload: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Storage error"})));
                    }
                    total_compressed += payload.len() as u64;
                }

                if let Err(e) = state.store.put_metadata(record.id, &record.metadata) {
                    error!("Failed to store metadata: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Storage error"})));
                }

                if let Err(e) = state.store.put_record(&record) {
                    error!("Failed to store record: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Storage error"})));
                }

                index.index_new(&record);
            }

            (StatusCode::OK, Json(json!({
                "id": first_id.to_string(),
                "success": true,
                "compressed_size": total_compressed,
            })))
        }
        Err(e) => {
            error!("Pipeline run failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
        }
    }
}
