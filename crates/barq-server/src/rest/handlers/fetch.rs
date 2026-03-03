use std::sync::Arc;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::state::AppState;

pub async fn fetch_handler(
    State(state): State<Arc<AppState>>,
    Path(id_str): Path<String>,
) -> impl IntoResponse {
    let id = match Uuid::parse_str(&id_str) {
        Ok(uuid) => uuid,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid UUID").into_response(),
    };

    if !state.store.record_exists(id) {
        return (StatusCode::NOT_FOUND, "Record not found").into_response();
    }

    let payload = match state.store.get_payload(id) {
        Ok(Some(p)) => p,
        Ok(None) => return (StatusCode::NOT_FOUND, "Payload deleted or not stored").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // Note: Decompression is left to the client in REST MVP
    // Stream response
    let body = Body::from(payload);

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/octet-stream")],
        body,
    ).into_response()
}
