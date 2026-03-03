use barq_types::BarqRecord;
use uuid::Uuid;

/// Asserts that an embedding is valid: correct dimension, no NaN/inf, and non-zero.
pub fn assert_embedding_valid(embedding: &[f32], expected_dim: usize) {
    assert_eq!(embedding.len(), expected_dim, "Embedding dimension mismatch: expected {}, got {}", expected_dim, embedding.len());
    let mut all_zero = true;
    for &val in embedding {
        assert!(val.is_finite(), "Embedding contains non-finite value: {}", val);
        if val != 0.0 {
            all_zero = false;
        }
    }
    assert!(!all_zero, "Embedding is all zeros");
}

/// Helper for cosine similarity
fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn magnitude(a: &[f32]) -> f32 {
    a.iter().map(|x| x * x).sum::<f32>().sqrt()
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot = dot_product(a, b);
    let mag_a = magnitude(a);
    let mag_b = magnitude(b);
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a * mag_b)
}

/// Asserts cosine similarity >= threshold.
pub fn assert_embedding_similar(a: &[f32], b: &[f32], threshold: f32) {
    let sim = cosine_similarity(a, b);
    assert!(sim >= threshold, "Embeddings not similar enough: {} < {}", sim, threshold);
}

/// Asserts cosine similarity <= threshold.
pub fn assert_embedding_different(a: &[f32], b: &[f32], threshold: f32) {
    let sim = cosine_similarity(a, b);
    assert!(sim <= threshold, "Embeddings too similar: {} > {}", sim, threshold);
}

/// Asserts compressed size is strictly smaller than original.
pub fn assert_compressed_smaller(original: &[u8], compressed: &[u8]) {
    assert!(compressed.len() < original.len(), "Compression failed to reduce size: {} >= {}", compressed.len(), original.len());
}

/// Asserts byte-exact equality with descriptive message.
pub fn assert_decompressed_matches(original: &[u8], decompressed: &[u8]) {
    assert_eq!(original.len(), decompressed.len(), "Decompressed length mismatch");
    assert_eq!(original, decompressed, "Decompressed bytes do not match original");
}

/// Asserts all core fields of a BarqRecord are present.
pub fn assert_record_complete(record: &BarqRecord) {
    assert!(!record.id.is_nil(), "Record ID is nil");
    assert!(!record.summary.is_empty(), "Record summary is empty");
    assert!(!record.embedding.is_empty(), "Record embedding is empty");
    assert!(record.embedding_dim > 0, "Record embedding_dim is 0");
}

/// Asserts search results are in non-increasing score order.
pub fn assert_search_results_ordered(results: &[(Uuid, f32)]) {
    for i in 0..results.len().saturating_sub(1) {
        assert!(results[i].1 >= results[i+1].1, "Search results not ordered: {} < {} at index {}", results[i].1, results[i+1].1, i);
    }
}

/// Asserts RRF score is in range (0.0, 1.0).
pub fn assert_rrf_score_valid(score: f32) {
    assert!(score > 0.0 && score < 1.0, "RRF score out of range: {}", score);
}
