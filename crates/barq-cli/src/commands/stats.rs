use crate::{Cli, Commands};
use tracing::{error, info};

pub async fn execute(cli: &Cli) -> anyhow::Result<()> {
    info!("Connecting to BarqVault for stats...");
    
    // For MVP, stats is also stubbed in the server to return 0s except for uptime
    // We will call the REST API since we haven't wrapped stats in BarqClient yet
    let url = format!("{}/api/v1/ping", cli.server.replace("50051", "8080"));
    let req_client = reqwest::Client::new();
    let res = req_client.get(&url).send().await.map_err(|e| {
        anyhow::anyhow!("Stats request failed: {}", e)
    })?;

    if !res.status().is_success() {
        error!("Server returned error: {}", res.status());
        std::process::exit(1);
    }

    println!("BarqVault Server Stats:");
    println!("-----------------------");
    println!("(Stats command via REST unimplemented in MVP, pinged for liveness)");

    Ok(())
}
