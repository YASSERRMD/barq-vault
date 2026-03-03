use crate::{select_codec_for_modality, compress, decompress, Codec};
use barq_types::Modality;
use proptest::prelude::*;

#[test]
fn test_codec_selection() {
    assert!(matches!(select_codec_for_modality(&Modality::Text), Codec::Lzma(6)));
    assert!(matches!(select_codec_for_modality(&Modality::Document), Codec::Lzma(6)));
    assert!(matches!(select_codec_for_modality(&Modality::Audio), Codec::Lzma(4)));
    assert!(matches!(select_codec_for_modality(&Modality::Image), Codec::Zstd(9)));
    assert!(matches!(select_codec_for_modality(&Modality::Video), Codec::Lz4));
}

#[test]
fn test_dispatch_round_trip() {
    let data = b"dispatch test data repeat ".repeat(10);
    
    // Test LZMA
    let c_lzma = compress(&data, Codec::Lzma(3)).unwrap();
    let d_lzma = decompress(&c_lzma, Codec::Lzma(3), data.len()).unwrap();
    assert_eq!(data.to_vec(), d_lzma);
    
    // Test LZ4
    let c_lz4 = compress(&data, Codec::Lz4).unwrap();
    let d_lz4 = decompress(&c_lz4, Codec::Lz4, data.len()).unwrap();
    assert_eq!(data.to_vec(), d_lz4);
    
    // Test Zstd
    let c_zstd = compress(&data, Codec::Zstd(3)).unwrap();
    let d_zstd = decompress(&c_zstd, Codec::Zstd(3), data.len()).unwrap();
    assert_eq!(data.to_vec(), d_zstd);
}

proptest! {
    #[test]
    fn test_lz4_property(ref s in "\\PC{1,10000}") {
        let data = s.as_bytes();
        let compressed = compress(data, Codec::Lz4).unwrap();
        let decompressed = decompress(&compressed, Codec::Lz4, data.len()).unwrap();
        prop_assert_eq!(data, &decompressed[..]);
    }
}
