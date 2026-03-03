use crate::{Cli, Commands};
use tracing::info;

pub async fn execute(_cli: &Cli, show: bool) -> anyhow::Result<()> {
    if show {
        info!("Loading server configuration...");
        match barq_server::config::ServerConfig::load() {
            Ok(config) => {
                let toml_string = toml::to_string_pretty(&config)?;
                println!("--- Current BarqVault Configuration ---");
                println!("{}", toml_string);
                println!("---------------------------------------");
            }
            Err(e) => {
                tracing::error!("Failed to load configuration: {}", e);
                std::process::exit(1);
            }
        }
    }
    
    Ok(())
}
