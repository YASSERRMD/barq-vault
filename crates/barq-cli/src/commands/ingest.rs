use crate::{Cli, Commands};
use barq_client::BarqClient;
use barq_types::{IngestRequest, StorageMode};
use tracing::{error, info};

pub async fn execute(cli: &Cli) -> anyhow::Result<()> {
    let Commands::Ingest {
        path,
        storage_mode,
        llm_provider,
        llm_key,
        llm_model,
        embed_provider,
        embed_key,
        embed_model,
        stt_provider,
        vlm_provider,
    } = &cli.command
    else {
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

    let sm = storage_mode.parse::<StorageMode>().unwrap_or(StorageMode::HybridFile);
    let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
    let modality = barq_ingest::detect_modality(&filename, None);

    info!("Reading file {:?} (modality: {})", path, modality);
    let bytes = tokio::fs::read(path).await.map_err(|e| {
        anyhow::anyhow!("Failed to read file: {}", e)
    })?;

    // Minimal config builder, passing keys to environment
    if let Some(k) = llm_key { std::env::set_var("BARQ__LLM__API_KEY", k); }
    if let Some(k) = embed_key { std::env::set_var("BARQ__EMBED__API_KEY", k); }
    std::env::set_var("BARQ__LLM__PROVIDER", llm_provider);
    std::env::set_var("BARQ__LLM__MODEL", llm_model);
    std::env::set_var("BARQ__EMBED__PROVIDER", embed_provider);
    std::env::set_var("BARQ__EMBED__MODEL", embed_model);

    if let Some(s) = stt_provider { std::env::set_var("BARQ__STT__PROVIDER", s); }
    if let Some(v) = vlm_provider { std::env::set_var("BARQ__VLM__PROVIDER", v); }

    let req = IngestRequest {
        summary: String::new(),
        embedding: Vec::new(),
        modality,
        storage_mode: sm,
        filename: Some(filename),
        raw_payload: Some(bytes),
        metadata: serde_json::Value::Object(Default::default()),
        chunk_index: 0,
        total_chunks: 1,
        parent_id: None,
    };

    match client.ingest(req).await {
        Ok(id) => {
            // Note: full table output would require parsing individual chunk records, 
            // but MVP just returns parent ID
            println!("Ingest successful!");
            println!("+--------------------------------------+");
            println!("| Record ID                            |");
            println!("+--------------------------------------+");
            println!("| {} |", id);
            println!("+--------------------------------------+");
        }
        Err(e) => {
            error!("Ingest failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
