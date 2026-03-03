use barq_types::Modality;

/// Detect a file's modality from its name and optional magic bytes.
pub fn detect_modality(filename: &str, raw_bytes: Option<&[u8]>) -> Modality {
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    if let Some(ref ext) = ext {
        match ext.as_str() {
            // Text
            "txt" | "md" | "json" | "csv" | "xml" | "yaml" | "yml" | "toml" | "rst" => {
                return Modality::Text
            }
            // Document
            "pdf" | "docx" | "xlsx" | "pptx" | "odt" | "ods" => return Modality::Document,
            // Image
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "tiff" | "tif" | "avif" => {
                return Modality::Image
            }
            // Audio
            "mp3" | "wav" | "flac" | "ogg" | "aac" | "opus" | "m4a" => {
                return Modality::Audio
            }
            // Video
            "mp4" | "mkv" | "avi" | "mov" | "webm" | "m4v" | "ts" => return Modality::Video,
            _ => {}
        }
    }

    // Magic byte sniffing fallback
    if let Some(bytes) = raw_bytes {
        if bytes.starts_with(b"\xFF\xD8\xFF") {
            return Modality::Image; // JPEG
        }
        if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            return Modality::Image; // PNG
        }
        if bytes.starts_with(b"%PDF") {
            return Modality::Document; // PDF
        }
        // MP4: check for 'ftyp' at offset 4
        if bytes.len() > 8 && &bytes[4..8] == b"ftyp" {
            return Modality::Video;
        }
        // RIFF/WAV
        if bytes.starts_with(b"RIFF") && bytes.len() > 8 && &bytes[8..12] == b"WAVE" {
            return Modality::Audio;
        }
        // OGG
        if bytes.starts_with(b"OggS") {
            return Modality::Audio;
        }
        // GIF
        if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
            return Modality::Image;
        }
    }

    // Default fallback
    Modality::Text
}

/// Map a filename to a MIME type string.
pub fn detect_mime_type(filename: &str) -> String {
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match ext.as_deref() {
        Some("txt") | Some("text") => "text/plain",
        Some("md") => "text/markdown",
        Some("json") => "application/json",
        Some("csv") => "text/csv",
        Some("xml") => "application/xml",
        Some("yaml") | Some("yml") => "application/yaml",
        Some("pdf") => "application/pdf",
        Some("docx") => {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        }
        Some("xlsx") => {
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        }
        Some("pptx") => {
            "application/vnd.openxmlformats-officedocument.presentationml.presentation"
        }
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        Some("tiff") | Some("tif") => "image/tiff",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("flac") => "audio/flac",
        Some("ogg") => "audio/ogg",
        Some("aac") => "audio/aac",
        Some("opus") => "audio/opus",
        Some("mp4") => "video/mp4",
        Some("mkv") => "video/x-matroska",
        Some("avi") => "video/x-msvideo",
        Some("mov") => "video/quicktime",
        Some("webm") => "video/webm",
        _ => "application/octet-stream",
    }
    .to_string()
}
