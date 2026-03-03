use crate::{Cli, Commands};
use barq_client::BarqClient;
use barq_types::SearchRequest;
use tracing::{error, info};

pub async fn execute(cli: &Cli) -> anyhow::Result<()> {
    let Commands::Search {
        query,
        top_k,
        vector_weight,
        modality,
        embed_key,
        embed_provider,
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

    // Client-side embedding generation for CLI (since gRPC requires client to provide vector)
    info!("Generating query embedding (provider: {})...", embed_provider);
    if let Some(k) = embed_key { std::env::set_var("BARQ__EMBED__API_KEY", k); }
    std::env::set_var("BARQ__EMBED__PROVIDER", embed_provider);
    let mut embed_config = barq_ingest::EmbedConfig::default();
    embed_config.provider = match embed_provider.as_str() {
        "cohere" => barq_ingest::EmbedProvider::Cohere,
        "mistral" => barq_ingest::EmbedProvider::Mistral,
        "local" => barq_ingest::EmbedProvider::Local,
        _ => barq_ingest::EmbedProvider::OpenAi,
    };
    if let Some(k) = embed_key {
        embed_config.api_key = Some(k.clone());
    } else if let Ok(k) = std::env::var("BARQ__EMBED__API_KEY") {
        embed_config.api_key = Some(k);
    }

    let query_embedding = match barq_ingest::embed(query, &embed_config).await {
        Ok(emb) => emb,
        Err(e) => {
            info!("Embedding failed ({}), falling back to BM25 only (zero vector)", e);
            vec![0.0; embed_config.expected_dim]
        }
    };

    let mod_filter = modality.as_deref().and_then(|m| m.parse().ok());

    let req = SearchRequest {
        query_embedding,
        query_text: query.clone(),
        vector_weight: *vector_weight,
        top_k: *top_k,
        modality_filter: mod_filter,
        metadata_filters: serde_json::Value::Object(Default::default()),
    };

    info!("Executing hybrid search...");
    match client.search(req).await {
        Ok(results) => {
            println!("Found {} results:", results.len());
            println!("{:-<100}", "");
            println!("{:<4} | {:<8} | {:<20} | {:<50}", "Rank", "Score", "Filename", "Summary Preview");
            println!("{:-<100}", "");
            for (i, hit) in results.iter().enumerate() {
                let fname = hit.filename.as_deref().unwrap_or("unknown");
                let fname_trunc = if fname.len() > 18 { format!("{}..", &fname[..16]) } else { fname.to_string() };
                
                let summary_clean = hit.summary.replace("\n", " ");
                let summary_trunc = if summary_clean.len() > 47 { format!("{}...", &summary_clean[..47]) } else { summary_clean };
                
                println!("{:<4} | {:<8.4} | {:<20} | {:<50}", i+1, hit.score, fname_trunc, summary_trunc);
            }
            println!("{:-<100}", "");
        }
        Err(e) => {
            error!("Search failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
