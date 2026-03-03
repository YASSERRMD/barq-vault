use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{info, error};

use barq_client::BarqClient;
use barq_types::{IngestRequest, Modality, SearchRequest, StorageMode};

#[derive(Parser, Debug)]
#[command(author, version, about = "CLI for BarqVault multimodal embedding database", long_about = None)]
struct Cli {
    /// gRPC endpoint URL for barq-server
    #[arg(short, long, default_value = "http://127.0.0.1:50051")]
    url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Ping the server to check connectivity and uptime
    Ping,
    
    /// Delete a record by UUID
    Delete {
        /// UUID of the record to delete
        id: String,
    },
    
    /// Ingest a local file to the server
    Ingest {
        /// Path to the file to ingest
        file: PathBuf,
        
        /// Data modality: text, image, audio, video, document
        #[arg(short, long, default_value = "document")]
        modality: String,
    },
    
    /// Search the database for matching vectors using hybrid search
    Search {
        /// Search query text
        query: String,
        
        /// Top K results to return
        #[arg(short, long, default_value_t = 5)]
        top_k: usize,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    let mut client = match BarqClient::connect(&cli.url).await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to connect to BarqVault server: {}", e);
            std::process::exit(1);
        }
    };

    match cli.command {
        Commands::Ping => {
            match client.ping().await {
                Ok((version, uptime)) => {
                    println!("Pong! version: {}, uptime: {}s", version, uptime);
                }
                Err(e) => error!("Ping failed: {}", e),
            }
        }
        Commands::Delete { id } => {
            let uuid = uuid::Uuid::parse_str(&id)?;
            match client.delete(uuid).await {
                Ok(_) => println!("Successfully deleted record: {}", id),
                Err(e) => error!("Delete failed: {}", e),
            }
        }
        Commands::Ingest { file, modality } => {
            info!("Reading file: {:?}", file);
            let bytes = tokio::fs::read(&file).await?;
            let filename = file.file_name().unwrap_or_default().to_string_lossy().to_string();
            
            let req = IngestRequest {
                summary: String::new(),
                embedding: Vec::new(),
                modality: modality.parse().unwrap_or(Modality::Document),
                storage_mode: StorageMode::HybridFile,
                filename: Some(filename),
                raw_payload: Some(bytes),
                metadata: serde_json::Value::Object(Default::default()),
                chunk_index: 0,
                total_chunks: 1,
                parent_id: None,
            };

            info!("Uploading {} bytes...", req.raw_payload.as_ref().unwrap().len());
            match client.ingest(req).await {
                Ok(id) => println!("Ingest complete! Parent record ID: {}", id),
                Err(e) => error!("Ingest failed: {}", e),
            }
        }
        Commands::Search { query, top_k } => {
            // Note: In MVP, the server REST handler generates query embeddings, but gRPC currently requires the client to send the query embedding.
            // Let's use barq-ingest locally to embed the query before sending via gRPC.
            info!("Generating embedding for query...");
            let embed_config = barq_ingest::EmbedConfig::default();
            let query_embedding = match barq_ingest::embed(&query, &embed_config).await {
                Ok(emb) => emb,
                Err(_) => {
                    info!("Embedding failed, falling back to zero vector (BM25 only)");
                    vec![0.0; embed_config.expected_dim]
                }
            };
            
            let req = SearchRequest {
                query_embedding,
                query_text: query.clone(),
                vector_weight: 0.5,
                top_k,
                modality_filter: None,
                metadata_filters: serde_json::Value::Object(Default::default()),
            };

            info!("Searching index...");
            match client.search(req).await {
                Ok(results) => {
                    println!("Found {} results:", results.len());
                    for (i, hit) in results.iter().enumerate() {
                        println!("[{}] {} (score: {:.4})", i+1, hit.filename.as_deref().unwrap_or("unknown"), hit.score);
                        println!("     ID: {}", hit.id);
                        println!("     Summary: {}", hit.summary);
                        println!("---");
                    }
                }
                Err(e) => error!("Search failed: {}", e),
            }
        }
    }

    Ok(())
}
