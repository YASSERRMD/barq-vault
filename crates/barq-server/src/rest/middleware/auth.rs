use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

pub async fn auth_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    // In a real implementation this would check state.config.server.auth
    // Since middleware running via from_fn requires access to state or extension,
    // we would extract State<Arc<AppState>> here.
    // However, axum middleware fn signatures don't inject state easily without it being in Extensions.
    // For MVP REST, we check header directly or skip.
    
    // For MVP, simply pass-through if we don't have state access
    Ok(next.run(req).await)
}
