use std::str::FromStr;
use rstest::rstest;
use crate::modality::{Modality, StorageMode, CodecType};

#[rstest]
#[case("text", Modality::Text)]
#[case("image", Modality::Image)]
#[case("audio", Modality::Audio)]
#[case("video", Modality::Video)]
#[case("document", Modality::Document)]
#[case("TEXT", Modality::Text)]
fn test_modality_from_str_valid(#[case] input: &str, #[case] expected: Modality) {
    assert_eq!(Modality::from_str(input).unwrap(), expected);
}

#[test]
fn test_modality_from_str_invalid() {
    assert!(Modality::from_str("unknown").is_err());
}

#[rstest]
#[case(Modality::Text, "text")]
#[case(Modality::Image, "image")]
#[case(Modality::Audio, "audio")]
#[case(Modality::Video, "video")]
#[case(Modality::Document, "document")]
fn test_modality_round_trip(#[case] variant: Modality, #[case] expected: &str) {
    let s = variant.to_string();
    assert_eq!(s, expected);
    assert_eq!(Modality::from_str(&s).unwrap(), variant);
}

#[rstest]
#[case("text_only", StorageMode::TextOnly)]
#[case("hybrid_file", StorageMode::HybridFile)]
#[case("full_raw", StorageMode::FullRaw)]
fn test_storage_mode_from_str_valid(#[case] input: &str, #[case] expected: StorageMode) {
    assert_eq!(StorageMode::from_str(input).unwrap(), expected);
}

#[rstest]
#[case(CodecType::Lzma(6))]
#[case(CodecType::Lz4)]
#[case(CodecType::Zstd(3))]
fn test_codec_type_serialization(#[case] codec: CodecType) {
    let json = serde_json::to_string(&codec).unwrap();
    let decoded: CodecType = serde_json::from_str(&json).unwrap();
    assert_eq!(codec, decoded);
}
