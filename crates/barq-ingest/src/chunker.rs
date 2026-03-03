/// Configuration for the text chunker.
#[derive(Debug, Clone)]
pub struct ChunkConfig {
    /// Approximate word count per chunk.
    pub chunk_size_tokens: usize,
    /// Number of words from the previous chunk to prepend as overlap.
    pub overlap_tokens: usize,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            chunk_size_tokens: 512,
            overlap_tokens: 64,
        }
    }
}

/// Whether the text is long enough to require chunking.
pub fn should_chunk(text: &str, threshold: usize) -> bool {
    text.split_whitespace().count() > threshold
}

/// Split `text` into overlapping chunks on sentence boundaries.
pub fn chunk_text(text: &str, config: &ChunkConfig) -> Vec<String> {
    if !should_chunk(text, config.chunk_size_tokens) {
        return vec![text.to_string()];
    }

    // Split into sentences on '.', '?', '!' followed by whitespace/newline
    let sentence_endings: &[char] = &['.', '?', '!'];
    let mut sentences: Vec<String> = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        current.push(ch);
        if sentence_endings.contains(&ch) {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                sentences.push(trimmed);
            }
            current.clear();
        }
    }
    if !current.trim().is_empty() {
        sentences.push(current.trim().to_string());
    }

    // Accumulate sentences into chunks with overlap
    let mut chunks: Vec<String> = Vec::new();
    let mut acc: Vec<String> = Vec::new();
    let mut word_count = 0usize;

    for sentence in &sentences {
        let wc = sentence.split_whitespace().count();
        if word_count + wc > config.chunk_size_tokens && !acc.is_empty() {
            chunks.push(acc.join(" "));

            // Collect overlap words into owned strings before reassigning acc
            let all_words: Vec<String> = acc
                .iter()
                .flat_map(|s| s.split_whitespace().map(|w| w.to_string()))
                .collect();
            let overlap_words: Vec<String> = all_words
                .iter()
                .rev()
                .take(config.overlap_tokens)
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            let overlap_str = overlap_words.join(" ");
            let overlap_len = overlap_words.len();
            acc = vec![overlap_str];
            word_count = overlap_len;
        }
        acc.push(sentence.clone());
        word_count += wc;
    }

    if !acc.is_empty() {
        chunks.push(acc.join(" "));
    }

    if chunks.is_empty() {
        vec![text.to_string()]
    } else {
        chunks
    }
}
