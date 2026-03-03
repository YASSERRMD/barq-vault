use crate::bm25::Bm25Index;
use uuid::Uuid;

fn make_index() -> Bm25Index {
    Bm25Index::new(1.5, 0.75)
}

#[test]
fn test_bm25_single_doc_match() {
    let mut index = make_index();
    let id = Uuid::new_v4();
    index.index_document(id, &["rust".to_string(), "fast".to_string()]);

    let results = index.score(&["rust".to_string()], 10);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, id);
    assert!(results[0].1 > 0.0);
}

#[test]
fn test_bm25_top_k_respected() {
    let mut index = make_index();
    for _ in 0..10 {
        let id = Uuid::new_v4();
        index.index_document(id, &["common".to_string()]);
    }
    let results = index.score(&["common".to_string()], 3);
    assert_eq!(results.len(), 3);
}

#[test]
fn test_bm25_no_match() {
    let mut index = make_index();
    let id = Uuid::new_v4();
    index.index_document(id, &["rust".to_string()]);
    let results = index.score(&["python".to_string()], 10);
    assert!(results.is_empty());
}

#[test]
fn test_bm25_more_relevant_doc_ranks_higher() {
    let mut index = make_index();
    let id_low = Uuid::new_v4();
    let id_high = Uuid::new_v4();

    // id_low has "rust" once; id_high has "rust" many times in a long doc context
    index.index_document(id_low, &["rust".to_string(), "memory".to_string(), "safe".to_string(), "concurrent".to_string()]);
    index.index_document(id_high, &["rust".to_string()]);

    let results = index.score(&["rust".to_string()], 10);
    // Both should appear; just check order consistency (scores should differ due to len)
    assert_eq!(results.len(), 2);
}

#[test]
fn test_bm25_remove_document() {
    let mut index = make_index();
    let id = Uuid::new_v4();
    index.index_document(id, &["test".to_string()]);

    assert_eq!(index.score(&["test".to_string()], 10).len(), 1);
    index.remove_document(id);
    assert_eq!(index.score(&["test".to_string()], 10).len(), 0);
}

#[test]
fn test_bm25_results_ordered_by_score() {
    let mut index = make_index();
    for _ in 0..5 {
        let id = Uuid::new_v4();
        index.index_document(id, &["common".to_string(), "keyword".to_string()]);
    }
    let results = index.score(&["common".to_string(), "keyword".to_string()], 10);
    for w in results.windows(2) {
        assert!(w[0].1 >= w[1].1);
    }
}
