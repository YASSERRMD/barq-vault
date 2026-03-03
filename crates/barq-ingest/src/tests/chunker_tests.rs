use crate::chunker::{chunk_text, should_chunk, ChunkConfig};

#[test]
fn test_should_chunk_false_for_short_text() {
    let text = "short text";
    assert!(!should_chunk(text, 512));
}

#[test]
fn test_should_chunk_true_for_long_text() {
    let text = "word ".repeat(600);
    assert!(should_chunk(&text, 512));
}

#[test]
fn test_chunk_short_text_returns_single_chunk() {
    let text = "This is a short document.";
    let config = ChunkConfig::default();
    let chunks = chunk_text(text, &config);
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0], text);
}

#[test]
fn test_chunk_long_text_produces_multiple_chunks() {
    let sentence = "This is one sentence. ";
    let text = sentence.repeat(100); // ~400 words
    let config = ChunkConfig {
        chunk_size_tokens: 20,
        overlap_tokens: 5,
    };
    let chunks = chunk_text(&text, &config);
    assert!(chunks.len() > 1, "Expected multiple chunks, got {}", chunks.len());
}

#[test]
fn test_chunks_overlap() {
    let sentence = "This is one sentence. ";
    let text = sentence.repeat(50);
    let config = ChunkConfig {
        chunk_size_tokens: 10,
        overlap_tokens: 3,
    };
    let chunks = chunk_text(&text, &config);
    if chunks.len() >= 2 {
        // The first few words of chunk[1] should appear in chunk[0]
        let last_words_of_first: Vec<&str> = chunks[0].split_whitespace().rev().take(3).collect();
        let first_words_of_second: Vec<&str> = chunks[1].split_whitespace().take(10).collect();
        let has_overlap = last_words_of_first.iter().any(|w| first_words_of_second.contains(w));
        assert!(has_overlap, "No overlap detected between consecutive chunks");
    }
}

#[test]
fn test_chunk_empty_text() {
    let config = ChunkConfig::default();
    let chunks = chunk_text("", &config);
    // Empty text wraps in one chunk (the text itself)
    assert_eq!(chunks.len(), 1);
}
