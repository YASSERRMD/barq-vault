use std::time::SystemTime;

use barq_types::{BarqError, BarqRecord, BarqResult, IngestRequest, Modality, StorageMode};
use blake3::Hasher;
use tracing::debug;
use uuid::Uuid;

use barq_compress::{compress, select_codec_for_modality, Codec};
use barq_compress::{compress_embedding};

use crate::{
    chunker::{chunk_text, should_chunk, ChunkConfig},
    detector::{detect_mime_type, detect_modality},
    embedder::{embed, EmbedConfig},
    extraction::{
        document::DocumentExtractor, ocr::OcrExtractor, stt::SttExtractor, text::PlainTextExtractor,
        vlm::VlmExtractor, TextExtractor,
    },
    summarizer::{summarize, LlmConfig},
};
use crate::extraction::stt::SttConfig;
use crate::extraction::vlm::VlmConfig;

/// Combined ingestion configuration.
#[derive(Debug, Clone, Default)]
pub struct IngestConfig {
    pub llm: LlmConfig,
    pub embed: EmbedConfig,
    pub stt: SttConfig,
    pub vlm: VlmConfig,
    pub chunk: ChunkConfig,
}

/// Orchestrates the full 10-step ingestion pipeline.
pub struct IngestPipeline {
    pub config: IngestConfig,
}

impl IngestPipeline {
    pub fn new(config: IngestConfig) -> Self {
        Self { config }
    }

    /// Run the full ingestion pipeline for a single file and return one or more `BarqRecord`s.
    pub async fn run(&self, request: IngestRequest) -> BarqResult<Vec<BarqRecord>> {
        // Step 1: Detect modality
        let raw_ref = request.raw_payload.as_deref();
        let fname = request.filename.as_deref().unwrap_or("unknown");
        let modality = detect_modality(fname, raw_ref);
        let mime_type = detect_mime_type(fname);
        debug!("Step 1: modality={}", modality);

        // Step 2: Extract text
        let raw_bytes = request.raw_payload.as_deref().unwrap_or(&[]);
        let extracted_text = self.extract_text(&modality, raw_bytes, fname).await?;
        debug!("Step 2: extracted {} chars", extracted_text.len());

        // Step 3: Chunk if needed
        let chunks = if should_chunk(&extracted_text, self.config.chunk.chunk_size_tokens) {
            chunk_text(&extracted_text, &self.config.chunk)
        } else {
            vec![extracted_text.clone()]
        };
        let total_chunks = chunks.len() as u32;
        debug!("Step 3: {} chunk(s)", total_chunks);

        // Step 8: Compute blake3 checksum of original bytes (done early for reuse)
        let checksum = {
            let mut h = Hasher::new();
            h.update(raw_bytes);
            let hash = h.finalize();
            let mut arr = [0u8; 32];
            arr.copy_from_slice(hash.as_bytes());
            arr
        };

        // Step 7: Compress raw payload if storage mode requires it
        let (compressed_payload, original_size, compressed_size, compression_ratio) =
            self.compress_payload(raw_bytes, &modality, &request.storage_mode)?;

        let now_secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let parent_id = if total_chunks > 1 {
            Some(Uuid::new_v4())
        } else {
            request.parent_id
        };

        let mut records = Vec::with_capacity(chunks.len());

        for (chunk_index, chunk_text_content) in chunks.iter().enumerate() {
            // Step 4: LLM summarization
            let summary = summarize(chunk_text_content, &modality, &self.config.llm).await?;
            debug!("Step 4: chunk {} summarized", chunk_index);

            // Step 5: Embedding
            let embedding = embed(&summary, &self.config.embed).await?;
            debug!("Step 5: chunk {} embedded (dim={})", chunk_index, embedding.len());

            // Step 6: BM25 tokenization
            let bm25_tokens = barq_index::tokenize(&summary);

            // Compress embedding (delta-f32 + Zstd)
            let compressed_embed = compress_embedding(&embedding).unwrap_or_default();

            // Step 9: Assemble BarqRecord
            let record = BarqRecord {
                id: Uuid::new_v4(),
                parent_id,
                chunk_index: chunk_index as u32,
                total_chunks,
                modality: modality.clone(),
                storage_mode: request.storage_mode.clone(),
                codec: codec_to_domain(select_codec_for_modality(&modality)),
                filename: request.filename.clone(),
                mime_type: Some(mime_type.clone()),
                summary,
                embedding,
                compressed_embed,
                embedding_dim: self.config.embed.expected_dim as u32,
                bm25_tokens,
                metadata: request.metadata.clone(),
                compressed_payload: compressed_payload.clone(),
                original_size,
                compressed_size,
                compression_ratio,
                created_at: now_secs,
                updated_at: now_secs,
                checksum,
            };
            records.push(record);
        }

        debug!("Step 10: assembled {} record(s)", records.len());
        Ok(records)
    }

    async fn extract_text(
        &self,
        modality: &Modality,
        raw: &[u8],
        filename: &str,
    ) -> BarqResult<String> {
        match modality {
            Modality::Text => PlainTextExtractor.extract(raw, filename).await,
            Modality::Document => DocumentExtractor.extract(raw, filename).await,
            Modality::Image => {
                // Try OCR first; use VLM as additional enrichment
                let ocr_text = OcrExtractor.extract(raw, filename).await.unwrap_or_default();
                let vlm_extractor = VlmExtractor::new(self.config.vlm.clone(), self.config.stt.clone());
                let vlm_text = vlm_extractor.extract(raw, filename).await.unwrap_or_default();
                Ok(format!("{} {}", ocr_text, vlm_text).trim().to_string())
            }
            Modality::Audio => {
                SttExtractor::new(self.config.stt.clone())
                    .extract(raw, filename)
                    .await
            }
            Modality::Video => {
                VlmExtractor::new(self.config.vlm.clone(), self.config.stt.clone())
                    .extract(raw, filename)
                    .await
            }
        }
    }

    fn compress_payload(
        &self,
        raw: &[u8],
        modality: &Modality,
        storage_mode: &StorageMode,
    ) -> BarqResult<(Option<Vec<u8>>, u64, u64, f32)> {
        let original_size = raw.len() as u64;

        if matches!(storage_mode, StorageMode::TextOnly) || raw.is_empty() {
            return Ok((None, original_size, 0, 0.0));
        }

        let codec = select_codec_for_modality(modality);
        let compressed = compress(raw, codec)
            .map_err(|e| BarqError::Compression(format!("Pipeline compress: {}", e)))?;
        let compressed_size = compressed.len() as u64;
        let ratio = if compressed_size == 0 {
            1.0
        } else {
            original_size as f32 / compressed_size as f32
        };

        Ok((Some(compressed), original_size, compressed_size, ratio))
    }
}

fn codec_to_domain(codec: barq_compress::Codec) -> barq_types::CodecType {
    match codec {
        barq_compress::Codec::Lzma(l) => barq_types::CodecType::Lzma(l),
        barq_compress::Codec::Lz4 => barq_types::CodecType::Lz4,
        barq_compress::Codec::Zstd(l) => barq_types::CodecType::Zstd(l),
    }
}
