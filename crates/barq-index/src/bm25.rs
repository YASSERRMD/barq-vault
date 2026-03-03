use std::collections::HashMap;

use uuid::Uuid;

/// BM25 inverted index with configurable k1 and b parameters.
pub struct Bm25Index {
    /// token → list of (doc_id, term_frequency)
    inverted: HashMap<String, Vec<(Uuid, u32)>>,
    /// per-document token count
    doc_lens: HashMap<Uuid, usize>,
    doc_count: usize,
    avg_len: f64,
    /// Term saturation parameter (default 1.5)
    pub k1: f64,
    /// Length normalization parameter (default 0.75)
    pub b: f64,
}

impl Bm25Index {
    pub fn new(k1: f64, b: f64) -> Self {
        Self {
            inverted: HashMap::new(),
            doc_lens: HashMap::new(),
            doc_count: 0,
            avg_len: 0.0,
            k1,
            b,
        }
    }

    /// Add a document to the index.
    pub fn index_document(&mut self, id: Uuid, tokens: &[String]) {
        // Count term frequencies
        let mut tf: HashMap<&str, u32> = HashMap::new();
        for token in tokens {
            *tf.entry(token.as_str()).or_insert(0) += 1;
        }

        // Insert postings into inverted index
        for (term, freq) in &tf {
            self.inverted
                .entry((*term).to_string())
                .or_default()
                .push((id, *freq));
        }

        self.doc_lens.insert(id, tokens.len());
        self.doc_count += 1;
        self.recompute_avg_len();
    }

    /// Remove a document from the index.
    pub fn remove_document(&mut self, id: Uuid) {
        self.doc_lens.remove(&id);
        if self.doc_count > 0 {
            self.doc_count -= 1;
        }

        // Prune postings for this document
        self.inverted.values_mut().for_each(|postings| {
            postings.retain(|(doc_id, _)| *doc_id != id);
        });
        // Remove empty posting lists
        self.inverted.retain(|_, v| !v.is_empty());

        self.recompute_avg_len();
    }

    /// Score documents for `query_tokens` and return top-k (doc_id, score).
    pub fn score(&self, query_tokens: &[String], top_k: usize) -> Vec<(Uuid, f64)> {
        let n = self.doc_count as f64;
        let mut doc_scores: HashMap<Uuid, f64> = HashMap::new();

        for token in query_tokens {
            if let Some(postings) = self.inverted.get(token) {
                let df = postings.len() as f64;
                let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();

                for (doc_id, tf) in postings {
                    let doc_len = *self.doc_lens.get(doc_id).unwrap_or(&1) as f64;
                    let tf_f = *tf as f64;
                    let tf_norm = tf_f * (self.k1 + 1.0)
                        / (tf_f + self.k1 * (1.0 - self.b + self.b * doc_len / self.avg_len));
                    *doc_scores.entry(*doc_id).or_default() += idf * tf_norm;
                }
            }
        }

        let mut ranked: Vec<(Uuid, f64)> = doc_scores.into_iter().collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked.truncate(top_k);
        ranked
    }

    fn recompute_avg_len(&mut self) {
        if self.doc_count == 0 {
            self.avg_len = 0.0;
        } else {
            let total: usize = self.doc_lens.values().sum();
            self.avg_len = total as f64 / self.doc_count as f64;
        }
    }
}
