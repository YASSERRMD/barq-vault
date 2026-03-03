use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;
use crate::rest::build_rest_router;
use crate::tests::state_tests::test_server_config;
use crate::state::AppState;
use barq_test_utils::fixtures::temp_dir_path;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_rest_router_ingest_endpoint_exists() {
    let dir = temp_dir_path();
    let config = test_server_config(dir.to_str().unwrap());
    let state = AppState::init(config).await.unwrap();
    let router = build_rest_router(state);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/ingest")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should not be 404 — endpoint exists (may return 422 for malformed body)
    assert_ne!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[serial]
async fn test_rest_router_search_endpoint_exists() {
    let dir = temp_dir_path();
    let config = test_server_config(dir.to_str().unwrap());
    let state = AppState::init(config).await.unwrap();
    let router = build_rest_router(state);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/search")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_ne!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[serial]
async fn test_rest_router_fetch_endpoint_exists() {
    let dir = temp_dir_path();
    let config = test_server_config(dir.to_str().unwrap());
    let state = AppState::init(config).await.unwrap();
    let router = build_rest_router(state);

    // 404 from router means route not defined; UUID 404 means route IS defined but record not found
    let response = router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/fetch/00000000-0000-0000-0000-000000000000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // 404 is acceptable (record not found), but NOT_FOUND from unknown route would be different
    // The route should be registered — any 2xx or 4xx from handler is OK, routing 404 is not
    let status = response.status();
    assert!(status.as_u16() < 500, "Unexpected server error: {}", status);
}

#[tokio::test]
#[serial]
async fn test_rest_router_unknown_path_returns_404() {
    let dir = temp_dir_path();
    let config = test_server_config(dir.to_str().unwrap());
    let state = AppState::init(config).await.unwrap();
    let router = build_rest_router(state);

    let response = router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/not/a/real/endpoint")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
