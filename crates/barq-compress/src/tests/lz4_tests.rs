use crate::lz4::{compress_lz4, decompress_lz4};
use barq_test_utils::assertions::{assert_decompressed_matches};
use std::time::Instant;
use crate::lzma::compress_lzma;

#[test]
fn test_lz4_empty_input() {
    let data = b"";
    let compressed = compress_lz4(data).unwrap();
    let decompressed = decompress_lz4(&compressed, 0).unwrap();
    assert!(decompressed.is_empty());
}

#[test]
fn test_lz4_round_trip_small() {
    let data = b"hello lz4 test data";
    let compressed = compress_lz4(data).unwrap();
    let decompressed = decompress_lz4(&compressed, data.len()).unwrap();
    assert_eq!(data, &decompressed[..]);
}

#[test]
fn test_lz4_large_data() {
    let data = vec![0u8; 1_000_000]; // 1MB
    let start = Instant::now();
    let compressed = compress_lz4(&data).unwrap();
    let duration = start.elapsed();
    
    let decompressed = decompress_lz4(&compressed, data.len()).unwrap();
    assert_decompressed_matches(&data, &decompressed);
    
    // LZ4 should be very fast
    assert!(duration.as_millis() < 100, "LZ4 compression took too long: {}ms", duration.as_millis());
}

#[test]
fn test_lz4_wrong_hint_too_small() {
    let data = b"some data here";
    let compressed = compress_lz4(data).unwrap();
    // Hinting 2 bytes for a 14 byte decompression
    let result = decompress_lz4(&compressed, 2);
    assert!(result.is_err());
}

#[test]
fn test_lz4_hint_too_large() {
    let data = b"some data here";
    let compressed = compress_lz4(data).unwrap();
    let decompressed = decompress_lz4(&compressed, 100).unwrap();
    assert_eq!(data, &decompressed[..]);
}

#[test]
fn test_lz4_vs_lzma_speed() {
    let data = b"repeated text data ".repeat(10000); // ~200KB
    
    let start_lz4 = Instant::now();
    let _ = compress_lz4(&data).unwrap();
    let dur_lz4 = start_lz4.elapsed();
    
    let start_lzma = Instant::now();
    let _ = compress_lzma(&data, 6).unwrap();
    let dur_lzma = start_lzma.elapsed();
    
    assert!(dur_lz4 < dur_lzma, "LZ4 ({:?}) should be faster than LZMA ({:?})", dur_lz4, dur_lzma);
}
