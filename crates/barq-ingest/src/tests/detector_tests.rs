use crate::detector::{detect_modality, detect_mime_type};
use barq_types::Modality;
use barq_test_utils::fixtures::{sample_pdf_bytes, sample_png_bytes, sample_wav_bytes};
use rstest::rstest;

// Extension-based detection
#[rstest]
#[case("doc.txt", Modality::Text)]
#[case("notes.md", Modality::Text)]
#[case("data.json", Modality::Text)]
#[case("report.pdf", Modality::Document)]
#[case("slide.pptx", Modality::Document)]
#[case("photo.jpg", Modality::Image)]
#[case("image.png", Modality::Image)]
#[case("audio.mp3", Modality::Audio)]
#[case("sound.wav", Modality::Audio)]
#[case("clip.mp4", Modality::Video)]
#[case("movie.mkv", Modality::Video)]
fn test_detect_modality_by_extension(#[case] filename: &str, #[case] expected: Modality) {
    assert_eq!(detect_modality(filename, None), expected);
}

// Magic byte detection
#[test]
fn test_detect_modality_pdf_magic_bytes() {
    let bytes = sample_pdf_bytes();
    assert_eq!(detect_modality("file.unknown", Some(&bytes)), Modality::Document);
}

#[test]
fn test_detect_modality_png_magic_bytes() {
    let bytes = sample_png_bytes();
    assert_eq!(detect_modality("file.unknown", Some(&bytes)), Modality::Image);
}

#[test]
fn test_detect_modality_wav_magic_bytes() {
    let bytes = sample_wav_bytes();
    assert_eq!(detect_modality("file.unknown", Some(&bytes)), Modality::Audio);
}

#[test]
fn test_detect_modality_extension_wins_over_magic() {
    // Extension says text, magic says nothing different
    let pdf = sample_pdf_bytes();
    // extension .txt says Text; magic says Document -> extension should override
    assert_eq!(detect_modality("doc.txt", Some(&pdf)), Modality::Text);
}

#[test]
fn test_detect_modality_unknown_defaults_to_text() {
    assert_eq!(detect_modality("file.xyz", None), Modality::Text);
}

// MIME type detection
#[rstest]
#[case("doc.txt", "text/plain")]
#[case("notes.md", "text/markdown")]
#[case("report.pdf", "application/pdf")]
#[case("photo.jpg", "image/jpeg")]
#[case("clip.mp4", "video/mp4")]
fn test_detect_mime_type(#[case] filename: &str, #[case] expected: &str) {
    assert_eq!(detect_mime_type(filename), expected);
}
