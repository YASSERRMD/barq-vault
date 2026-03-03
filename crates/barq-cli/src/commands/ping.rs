use crate::Cli;
use barq_client::BarqClient;
use tracing::{error, info};

pub async fn execute(cli: &Cli) -> anyhow::Result<()> {
    info!("Connecting to BarqVault...");
    let mut client = match BarqClient::connect(&cli.server).await {
        Ok(c) => c,
        Err(e) => {
            error!("Connection failed: {}", e);
            std::process::exit(1);
        }
    };

    match client.ping().await {
        Ok((version, uptime)) => {
            println!("Pong! version: {}, uptime: {}s", version, uptime);
        }
        Err(e) => {
            error!("Ping failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
