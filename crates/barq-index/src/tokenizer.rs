/// A fixed set of ~50 English stopwords to filter from BM25 indexes.
pub const STOPWORDS: &[&str] = &[
    "a", "an", "the", "and", "or", "but", "in", "on", "at", "to", "for",
    "of", "with", "by", "from", "as", "is", "was", "are", "were", "be",
    "been", "being", "have", "has", "had", "do", "does", "did", "will",
    "would", "could", "should", "may", "might", "shall", "can", "not",
    "no", "nor", "so", "yet", "both", "either", "neither", "this", "that",
    "these", "those", "it", "its", "they", "them", "their",
];

/// Tokenize `text` for BM25 indexing.
///
/// - Lowercases the input
/// - Splits on whitespace and punctuation
/// - Removes tokens shorter than 3 characters
/// - Trims non-alphanumeric characters from edges
/// - Deduplicates while preserving first-occurrence order
/// - Filters English stopwords
pub fn tokenize(text: &str) -> Vec<String> {
    let stopwords: std::collections::HashSet<&str> = STOPWORDS.iter().copied().collect();

    let mut seen = std::collections::HashSet::new();
    let mut tokens = Vec::new();

    for word in text.split(|c: char| !c.is_alphanumeric()) {
        let token = word
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase();

        if token.len() < 3 {
            continue;
        }
        if stopwords.contains(token.as_str()) {
            continue;
        }
        if seen.insert(token.clone()) {
            tokens.push(token);
        }
    }

    tokens
}

/// Tokenize `text` for a search query — same as `tokenize` but without
/// stopword filtering so user intent is fully preserved.
pub fn tokenize_query(text: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut tokens = Vec::new();

    for word in text.split(|c: char| !c.is_alphanumeric()) {
        let token = word
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase();

        if token.len() < 2 {
            continue;
        }
        if seen.insert(token.clone()) {
            tokens.push(token);
        }
    }

    tokens
}
