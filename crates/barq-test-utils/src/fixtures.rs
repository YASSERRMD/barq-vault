use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use std::io::Write;
use uuid::Uuid;

/// Returns a ~50-word Lorem Ipsum paragraph as bytes.
pub fn sample_text_bytes() -> Vec<u8> {
    b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod \
      tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, \
      quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo \
      consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse \
      cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat \
      non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."
        .to_vec()
}

/// Returns minimal valid PDF magic bytes with embedded text.
pub fn sample_pdf_bytes() -> Vec<u8> {
    let mut pdf = Vec::new();
    pdf.extend_from_slice(b"%PDF-1.4\n");
    pdf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    pdf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    pdf.extend_from_slice(b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R >>\nendobj\n");
    pdf.extend_from_slice(b"4 0 obj\n<< /Length 44 >>\nstream\n");
    pdf.extend_from_slice(b"BT /F1 12 Tf 100 700 Td (Hello, PDF Test!) Tj ET\n");
    pdf.extend_from_slice(b"endstream\nendobj\n");
    pdf.extend_from_slice(b"xref\n0 5\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n0000000203 00000 n \n");
    pdf.extend_from_slice(b"trailer\n<< /Size 5 /Root 1 0 R >>\n");
    pdf.extend_from_slice(b"startxref\n297\n%%EOF\n");
    pdf
}

/// Returns a minimal valid 1x1 transparent PNG.
pub fn sample_png_bytes() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // Magic bytes
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk start
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // Width 1, Height 1
        0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, // 8-bit truecolor+alpha
        0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, // IDAT chunk start
        0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, // Deflate encoded transparent pixel
        0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 
        0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, // IEND chunk
        0x42, 0x60, 0x82,
    ]
}

/// Returns a minimal valid WAV header with silence.
pub fn sample_wav_bytes() -> Vec<u8> {
    vec![
        b'R', b'I', b'F', b'F', // Chunk ID
        0x24, 0x00, 0x00, 0x00, // Chunk Size (36)
        b'W', b'A', b'V', b'E', // Format
        b'f', b'm', b't', b' ', // Subchunk1 ID
        0x10, 0x00, 0x00, 0x00, // Subchunk1 Size (16)
        0x01, 0x00,             // AudioFormat (PCM)
        0x01, 0x00,             // NumChannels (1)
        0x44, 0xAC, 0x00, 0x00, // SampleRate (44100)
        0x88, 0x58, 0x01, 0x00, // ByteRate
        0x02, 0x00,             // BlockAlign
        0x10, 0x00,             // BitsPerSample (16)
        b'd', b'a', b't', b'a', // Subchunk2 ID
        0x00, 0x00, 0x00, 0x00, // Subchunk2 Size (0)
    ]
}

/// Returns minimal valid MP4 header bytes (ftyp box).
pub fn sample_mp4_bytes() -> Vec<u8> {
    vec![
        0x00, 0x00, 0x00, 0x18, // size (24)
        b'f', b't', b'y', b'p', // box type
        b'i', b's', b'o', b'm', // major brand
        0x00, 0x00, 0x02, 0x00, // minor version
        b'i', b's', b'o', b'm', // compatible brand 1
        b'i', b's', b'o', b'2', // compatible brand 2
        b'a', b'v', b'c', b'1', // compatible brand 3
        b'm', b'p', b'4', b'1', // compatible brand 4
    ]
}

/// Writes content to a temp file with the given extension and returns it.
pub fn temp_file(content: &[u8], extension: &str) -> NamedTempFile {
    let mut ext = ".".to_string();
    ext.push_str(extension);
    
    let mut file = tempfile::Builder::new()
        .suffix(&ext)
        .tempfile()
        .expect("Failed to create temp file");
        
    file.write_all(content).expect("Failed to write to temp file");
    file.flush().unwrap();
    file
}

/// Returns a unique temporary directory path for tests.
pub fn temp_dir_path() -> PathBuf {
    std::env::temp_dir().join(format!("barq_test_{}", Uuid::new_v4()))
}
