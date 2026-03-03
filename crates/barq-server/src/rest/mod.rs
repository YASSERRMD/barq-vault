use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::limit::RequestBodyLimitLayer;

use crate::state::AppState;
use handlers::{
    fetch::fetch_handler,
    ingest::ingest_handler,
    search::search_handler,
};

pub mod handlers;
pub mod middleware;

pub fn build_rest_router(state: Arc<AppState>) -> Router {
    let max_payload = state.config.server.max_payload_bytes as usize;

    Router::new()
        .route("/api/v1/ingest", post(ingest_handler))
        .route("/api/v1/search", post(search_handler))
        .route("/api/v1/fetch/:id", get(fetch_handler))
        // Apply a high payload limit for media ingestion (e.g. 100MB by default if configured)
        .layer(RequestBodyLimitLayer::new(max_payload))
        // We skip axum auth middleware in MVP for simplicity, real app would .layer(middleware::from_fn(auth_middleware))
        .with_state(state)
}
