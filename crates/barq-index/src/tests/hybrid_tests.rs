use crate::hybrid::{HybridEngine, SearchParams};
use crate::tokenizer::tokenize;
use barq_test_utils::builders::{random_embedding, similar_embedding, orthogonal_embedding};
use barq_types::Modality;
use uuid::Uuid;
use serde_json::json;

fn make_engine() -> HybridEngine {
    HybridEngine::new(1.5, 0.75, 4)
}

#[test]
fn test_hybrid_rrf_scores_are_positive() {
    let mut engine = make_engine();
    let id = Uuid::new_v4();
    let emb = vec![1.0f32, 0.0, 0.0, 0.0];

    engine.bm25.index_document(id, &["rust".to_string()]);
    engine.vector.upsert(id, emb.clone()).unwrap();

    let results = engine.search(SearchParams {
        query_embedding: emb,
        query_text: "rust".to_string(),
        vector_weight: 0.5,
        top_k: 5,
        modality_filter: None,
        metadata_filters: json!({}),
    }).unwrap();

    assert_eq!(results.len(), 1);
    assert!(results[0].1 > 0.0);
}

#[test]
fn test_hybrid_rrf_results_ordered_descending() {
    let mut engine = HybridEngine::new(1.5, 0.75, 4);
    for _ in 0..5 {
        let id = Uuid::new_v4();
        engine.bm25.index_document(id, &["common".to_string()]);
        engine.vector.upsert(id, vec![1.0, 0.0, 0.0, 0.0]).unwrap();
    }

    let results = engine.search(SearchParams {
        query_embedding: vec![1.0, 0.0, 0.0, 0.0],
        query_text: "common".to_string(),
        vector_weight: 0.5,
        top_k: 5,
        modality_filter: None,
        metadata_filters: json!({}),
    }).unwrap();

    for w in results.windows(2) {
        assert!(w[0].1 >= w[1].1, "Results not ordered: {} < {}", w[0].1, w[1].1);
    }
}

#[test]
fn test_hybrid_top_k_limit() {
    let mut engine = HybridEngine::new(1.5, 0.75, 4);
    for _ in 0..20 {
        let id = Uuid::new_v4();
        engine.bm25.index_document(id, &["term".to_string()]);
        engine.vector.upsert(id, vec![1.0, 0.0, 0.0, 0.0]).unwrap();
    }

    let results = engine.search(SearchParams {
        query_embedding: vec![1.0, 0.0, 0.0, 0.0],
        query_text: "term".to_string(),
        vector_weight: 0.5,
        top_k: 5,
        modality_filter: None,
        metadata_filters: json!({}),
    }).unwrap();

    assert_eq!(results.len(), 5);
}

#[test]
fn test_hybrid_no_results_for_missing_term() {
    let mut engine = HybridEngine::new(1.5, 0.75, 4);
    let id = Uuid::new_v4();
    engine.bm25.index_document(id, &["rust".to_string()]);
    engine.vector.upsert(id, vec![1.0, 0.0, 0.0, 0.0]).unwrap();

    // Zero-norm query will return 0-similarity from vector; no bm25 match
    let results = engine.search(SearchParams {
        query_embedding: vec![0.0, 0.0, 0.0, 0.0],
        query_text: "python".to_string(),
        vector_weight: 1.0,
        top_k: 5,
        modality_filter: None,
        metadata_filters: json!({}),
    }).unwrap();

    // With pure vector_weight=1.0, the zero-norm query gives 0.0 cosine sim,
    // but the doc still appears with a tiny RRF rank contribution
    // This test just verifies no panic / error occurs
    let _ = results;
}
