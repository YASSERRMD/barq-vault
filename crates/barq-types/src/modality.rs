use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Represents the modality (content type) of an ingested record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modality {
    Text,
    Image,
    Audio,
    Video,
    Document,
}

impl fmt::Display for Modality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Modality::Text => "text",
            Modality::Image => "image",
            Modality::Audio => "audio",
            Modality::Video => "video",
            Modality::Document => "document",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Modality {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(Modality::Text),
            "image" => Ok(Modality::Image),
            "audio" => Ok(Modality::Audio),
            "video" => Ok(Modality::Video),
            "document" => Ok(Modality::Document),
            other => Err(format!("Unknown modality: {}", other)),
        }
    }
}

/// Determines how (or whether) raw bytes are stored alongside the text summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageMode {
    /// Only the text summary and embedding are stored; no raw bytes.
    TextOnly,
    /// Summary + embedding + compressed original file.
    HybridFile,
    /// Full raw bytes stored without modification (use only for debug).
    FullRaw,
}

impl fmt::Display for StorageMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            StorageMode::TextOnly => "text_only",
            StorageMode::HybridFile => "hybrid_file",
            StorageMode::FullRaw => "full_raw",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for StorageMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text_only" => Ok(StorageMode::TextOnly),
            "hybrid_file" => Ok(StorageMode::HybridFile),
            "full_raw" => Ok(StorageMode::FullRaw),
            other => Err(format!("Unknown storage mode: {}", other)),
        }
    }
}

/// The compression codec used for stored payloads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodecType {
    /// LZMA compression at a given level (1–9).
    Lzma(u32),
    /// LZ4 fast compression.
    Lz4,
    /// Zstd compression at a given level.
    Zstd(i32),
}

impl fmt::Display for CodecType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodecType::Lzma(level) => write!(f, "lzma({})", level),
            CodecType::Lz4 => write!(f, "lz4"),
            CodecType::Zstd(level) => write!(f, "zstd({})", level),
        }
    }
}
