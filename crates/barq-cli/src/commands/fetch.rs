use crate::{Cli, Commands};
use barq_client::BarqClient;
use tracing::{error, info};

pub async fn execute(cli: &Cli) -> anyhow::Result<()> {
    let Commands::Fetch { id, output, stdout } = &cli.command else {
        return Ok(());
    };

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

    info!("Fetching payload for record {}...", uuid);
    // In our BarqClient, fetch isn't implemented as a high level wrapper yet, 
    // so we will make a direct REST call since fetch returns a stream.
    // For MVP, if we use the REST endpoint we can just download it:
    
    let url = format!("{}/api/v1/fetch/{}", cli.server.replace("50051", "8080"), uuid);
    let req_client = reqwest::Client::new();
    let res = req_client.get(&url).send().await.map_err(|e| {
        anyhow::anyhow!("Fetch request failed: {}", e)
    })?;

    if !res.status().is_success() {
        error!("Server returned error: {}", res.status());
        std::process::exit(1);
    }

    let bytes = res.bytes().await.map_err(|e| {
        anyhow::anyhow!("Failed to read response bytes: {}", e)
    })?;

    if *stdout {
        use std::io::Write;
        std::io::stdout().write_all(&bytes)?;
    } else if let Some(path) = output {
        tokio::fs::write(path, &bytes).await?;
        println!("Successfully wrote {} bytes to {:?}", bytes.len(), path);
    } else {
        println!("Fetched {} bytes. Use --output or --stdout to save/view.", bytes.len());
    }

    Ok(())
}
