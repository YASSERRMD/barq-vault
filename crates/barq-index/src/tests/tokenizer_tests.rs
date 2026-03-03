use crate::tokenizer::{tokenize, tokenize_query};
use rstest::rstest;

#[rstest]
#[case("Hello World", vec!["hello", "world"])]
#[case("This is a test of the system", vec!["test", "system"])]
#[case("Data-driven optimization!", vec!["data", "driven", "optimization"])]
fn test_tokenize_basic(#[case] input: &str, #[case] expected: Vec<&str>) {
    let tokens = tokenize(input);
    assert_eq!(tokens, expected);
}

#[test]
fn test_tokenize_deduplication() {
    let input = "test test test again again";
    let tokens = tokenize(input);
    assert_eq!(tokens, vec!["test", "again"]);
}

#[test]
fn test_tokenize_filters_stopwords() {
    let input = "the quick brown fox";
    let tokens = tokenize(input);
    // "the" is a stopword, should be filtered
    assert!(!tokens.contains(&"the".to_string()));
    assert!(tokens.contains(&"quick".to_string()));
    assert!(tokens.contains(&"brown".to_string()));
    assert!(tokens.contains(&"fox".to_string()));
}

#[test]
fn test_tokenize_query_preserves_stopwords() {
    let input = "to be or not to be";
    let tokens = tokenize_query(input);
    // tokenize_query doesn't strip stopwords, only deduplicates
    assert!(tokens.contains(&"to".to_string()) || tokens.contains(&"be".to_string()));
}

#[test]
fn test_tokenize_empty() {
    assert!(tokenize("").is_empty());
    assert!(tokenize("   ").is_empty());
    assert!(tokenize("!!!").is_empty());
}

#[test]
fn test_tokenize_short_tokens_filtered() {
    // Tokens < 3 chars should be filtered
    let tokens = tokenize("a ab abc abcd");
    assert!(!tokens.contains(&"a".to_string()));
    assert!(!tokens.contains(&"ab".to_string()));
    assert!(tokens.contains(&"abc".to_string()));
    assert!(tokens.contains(&"abcd".to_string()));
}
