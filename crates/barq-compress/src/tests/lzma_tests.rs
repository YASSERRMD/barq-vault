use crate::lzma::{compress_lzma, decompress_lzma};
use barq_test_utils::assertions::{assert_compressed_smaller, assert_decompressed_matches};
use rstest::rstest;

#[test]
fn test_lzma_empty_input() {
    let data = b"";
    let compressed = compress_lzma(data, 6).unwrap();
    let decompressed = decompress_lzma(&compressed, 0).unwrap();
    assert!(decompressed.is_empty());
}

#[test]
fn test_lzma_single_byte() {
    let data = b"a";
    let compressed = compress_lzma(data, 6).unwrap();
    let decompressed = decompress_lzma(&compressed, 1).unwrap();
    assert_eq!(decompressed, data);
}

#[test]
fn test_lzma_round_trip_1kb() {
    let data = vec![0x42u8; 1024];
    let compressed = compress_lzma(&data, 6).unwrap();
    let decompressed = decompress_lzma(&compressed, 1024).unwrap();
    assert_decompressed_matches(&data, &decompressed);
    assert_compressed_smaller(&data, &compressed);
}

#[rstest]
#[case(0)]
#[case(1)]
#[case(6)]
#[case(9)]
fn test_lzma_levels(#[case] level: u32) {
    let data = b"test data repeated test data repeated".repeat(10);
    let compressed = compress_lzma(&data, level).unwrap();
    let decompressed = decompress_lzma(&compressed, data.len()).unwrap();
    assert_eq!(data, decompressed);
}

#[test]
fn test_lzma_corrupted() {
    let data = b"some data to compress";
    let mut compressed = compress_lzma(data, 6).unwrap();
    
    // Corrupt a byte in the middle of the payload (the first 8 bytes are size)
    if compressed.len() > 10 {
        compressed[9] ^= 0xFF;
    }
    
    let result = decompress_lzma(&compressed, data.len());
    assert!(result.is_err());
}

#[test]
fn test_lzma_empty_compressed_input() {
    let result = decompress_lzma(b"", 10);
    assert!(result.is_err());
}
