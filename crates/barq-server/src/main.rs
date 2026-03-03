use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpListener;
use tonic::transport::Server;
use tracing::{error, info};

use barq_proto::BarqVaultServer;
use barq_server::{config::ServerConfig, grpc::BarqVaultService, rest::build_rest_router, AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load server config
    info!("Loading barq-vault configuration...");
    let config = ServerConfig::load().unwrap_or_else(|e| {
        error!("Failed to load config: {}", e);
        std::process::exit(1);
    });

    // Initialize application state (store, index, pipeline)
    info!("Initializing BarqVault application state...");
    let state = AppState::init(config.clone()).await.unwrap_or_else(|e| {
        error!("Failed to initialize app state: {}", e);
        std::process::exit(1);
    });

    let grpc_addr: SocketAddr = config.server.grpc_addr.parse()?;
    let rest_addr: SocketAddr = config.server.rest_addr.parse()?;

    // Build gRPC service
    let grpc_service = BarqVaultServer::new(BarqVaultService::new(Arc::clone(&state)));

    // Build REST router
    let rest_app = build_rest_router(Arc::clone(&state));

    info!("Starting BarqVault gRPC server on {}", grpc_addr);
    info!("Starting BarqVault REST API on {}", rest_addr);

    // Spawn REST server
    let rest_listener = TcpListener::bind(rest_addr).await?;
    let rest_task = tokio::spawn(async move {
        axum::serve(rest_listener, rest_app).await.unwrap();
    });

    // Spawn gRPC server
    let grpc_task = tokio::spawn(async move {
        Server::builder()
            .add_service(grpc_service)
            .serve(grpc_addr)
            .await
            .unwrap();
    });

    // Wait for both to finish (in reality, run indefinitely)
    let _ = tokio::try_join!(rest_task, grpc_task);

    Ok(())
}
