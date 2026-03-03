use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub mod commands;

#[derive(Parser, Debug)]
#[command(author, version, about = "CLI for BarqVault multimodal embedding database", long_about = None)]
pub struct Cli {
    /// gRPC endpoint URL for barq-server
    #[arg(short, long, default_value = "http://127.0.0.1:50051")]
    pub server: String,

    /// Transport provider (grpc or rest)
    #[arg(short, long, default_value = "grpc")]
    pub transport: String,

    /// Authentication token
    #[arg(short, long)]
    pub auth_token: Option<String>,

    /// Path to config file
    #[arg(short, long)]
    pub config_file: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Ingest a local file to the server
    Ingest {
        /// Path to the file to ingest
        path: PathBuf,
        
        #[arg(long, default_value = "hybrid_file")]
        storage_mode: String,
        
        #[arg(long, default_value = "openai")]
        llm_provider: String,
        
        #[arg(long)]
        llm_key: Option<String>,
        
        #[arg(long, default_value = "gpt-4o-mini")]
        llm_model: String,
        
        #[arg(long, default_value = "openai")]
        embed_provider: String,
        
        #[arg(long)]
        embed_key: Option<String>,
        
        #[arg(long, default_value = "text-embedding-3-small")]
        embed_model: String,
        
        #[arg(long)]
        stt_provider: Option<String>,
        
        #[arg(long)]
        vlm_provider: Option<String>,
    },
    
    /// Search the database for matching vectors using hybrid search
    Search {
        /// Search query text
        query: String,
        
        /// Top K results to return
        #[arg(short, long, default_value_t = 5)]
        top_k: usize,
        
        /// Weight of vector search (0.0 to 1.0)
        #[arg(short, long, default_value_t = 0.5)]
        vector_weight: f32,
        
        /// Filter by modality
        #[arg(short, long)]
        modality: Option<String>,
        
        #[arg(long)]
        embed_key: Option<String>,
        
        #[arg(long, default_value = "openai")]
        embed_provider: String,
    },
    
    /// Fetch a stored payload
    Fetch {
        /// UUID of the record to fetch
        id: String,
        
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Print payload to stdout
        #[arg(long, default_value_t = false)]
        stdout: bool,
    },
    
    /// Delete a record by UUID
    Delete {
        /// UUID of the record to delete
        id: String,
    },
    
    /// Ping the server to check connectivity and uptime
    Ping,
    
    /// Show server-wide statistics
    Stats,
    
    /// Show configuration
    Config {
        /// Print resolved config as TOML
        #[arg(short, long, default_value_t = true)]
        show: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match &cli.command {
        Commands::Ingest { .. } => commands::ingest::execute(&cli).await,
        Commands::Search { .. } => commands::search::execute(&cli).await,
        Commands::Fetch { .. } => commands::fetch::execute(&cli).await,
        Commands::Delete { id } => commands::delete::execute(&cli, id).await,
        Commands::Ping => commands::ping::execute(&cli).await,
        Commands::Stats => commands::stats::execute(&cli).await,
        Commands::Config { show } => commands::config_cmd::execute(&cli, *show).await,
    }
}
