use crate::embedding::{compress_embedding, decompress_embedding};
use barq_test_utils::builders::random_embedding;
use approx::assert_relative_eq;

#[test]
fn test_embedding_round_trip_zeros() {
    let vec = vec![0.0f32; 128];
    let compressed = compress_embedding(&vec).unwrap();
    let decompressed = decompress_embedding(&compressed, 128).unwrap();
    assert_eq!(vec, decompressed);
}

#[test]
fn test_embedding_round_trip_random_384() {
    let vec = random_embedding(384);
    let compressed = compress_embedding(&vec).unwrap();
    let decompressed = decompress_embedding(&compressed, 384).unwrap();
    
    for (a, b) in vec.iter().zip(decompressed.iter()) {
        assert_relative_eq!(a, b, epsilon = 1e-5);
    }
}

#[test]
fn test_embedding_round_trip_1536() {
    let vec = random_embedding(1536);
    let compressed = compress_embedding(&vec).unwrap();
    let decompressed = decompress_embedding(&compressed, 1536).unwrap();
    
    for (a, b) in vec.iter().zip(decompressed.iter()) {
        assert_relative_eq!(a, b, epsilon = 1e-5);
    }
}

#[test]
fn test_embedding_wrong_dim() {
    let vec = random_embedding(128);
    let compressed = compress_embedding(&vec).unwrap();
    // Hinting 64 instead of 128
    let result = decompress_embedding(&compressed, 64);
    assert!(result.is_err());
}

#[test]
fn test_embedding_compression_ratio() {
    let vec = random_embedding(1536);
    let raw_size = vec.len() * 4;
    let compressed = compress_embedding(&vec).unwrap();
    assert!(compressed.len() < raw_size, "Embedding compression failed to reduce size: {} >= {}", compressed.len(), raw_size);
}
