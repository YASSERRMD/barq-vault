use std::collections::HashMap;

use barq_types::{BarqError, BarqResult};
use uuid::Uuid;

/// In-memory vector index using cosine similarity.
///
/// Phase 6 upgrade: replace with HNSW via barq-db integration.
pub struct VectorIndex {
    vectors: HashMap<Uuid, Vec<f32>>,
    pub dim: usize,
}

impl VectorIndex {
    pub fn new(dim: usize) -> Self {
        Self {
            vectors: HashMap::new(),
            dim,
        }
    }

    /// Insert or update an embedding for a document.
    pub fn upsert(&mut self, id: Uuid, embedding: Vec<f32>) -> BarqResult<()> {
        if embedding.len() != self.dim {
            return Err(BarqError::InvalidInput(format!(
                "Embedding dim mismatch: expected {}, got {}",
                self.dim,
                embedding.len()
            )));
        }
        self.vectors.insert(id, embedding);
        Ok(())
    }

    /// Remove a document's embedding from the index.
    pub fn remove(&mut self, id: Uuid) {
        self.vectors.remove(&id);
    }

    /// Search for the top-k most similar vectors using cosine similarity.
    pub fn search_cosine(&self, query: &[f32], top_k: usize) -> Vec<(Uuid, f32)> {
        let mut scored: Vec<(Uuid, f32)> = self
            .vectors
            .iter()
            .map(|(id, emb)| (*id, cosine_similarity(query, emb)))
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }
}

/// Compute cosine similarity between two f32 slices.
/// Returns 0.0 if either vector has zero norm.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}
