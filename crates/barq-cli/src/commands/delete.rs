use crate::{Cli, Commands};
use barq_client::BarqClient;
use tracing::{error, info};

pub async fn execute(cli: &Cli, id: &str) -> anyhow::Result<()> {
    info!("Connecting to BarqVault...");
    let mut client = match BarqClient::connect(&cli.server).await {
        Ok(c) => c,
        Err(e) => {
            error!("Connection failed: {}", e);
            std::process::exit(1);
        }
    };

    let uuid = match uuid::Uuid::parse_str(id) {
        Ok(u) => u,
        Err(_) => {
            error!("Invalid UUID provided: {}", id);
            std::process::exit(1);
        }
    };

    match client.delete(uuid).await {
        Ok(_) => println!("Successfully deleted record: {}", id),
        Err(e) => {
            error!("Delete failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
