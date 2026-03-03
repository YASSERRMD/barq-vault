// barq-vault: barq-ingest

pub mod chunker;
pub mod detector;
pub mod embedder;
pub mod extraction;
pub mod pipeline;
pub mod summarizer;

pub use chunker::{chunk_text, should_chunk, ChunkConfig};
pub use detector::{detect_mime_type, detect_modality};
pub use embedder::{embed, EmbedConfig, EmbedProvider};
pub use pipeline::{IngestConfig, IngestPipeline};
pub use summarizer::{summarize, LlmConfig, LlmProvider};
