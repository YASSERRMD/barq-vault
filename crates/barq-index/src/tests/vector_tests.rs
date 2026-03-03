use crate::vector::VectorIndex;
use barq_test_utils::builders::{random_embedding, similar_embedding, orthogonal_embedding};
use uuid::Uuid;

#[test]
fn test_vector_upsert_and_retrieve() {
    let mut index = VectorIndex::new(4);
    let id = Uuid::new_v4();
    index.upsert(id, vec![1.0, 0.0, 0.0, 0.0]).unwrap();

    let results = index.search_cosine(&[1.0, 0.0, 0.0, 0.0], 5);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, id);
}

#[test]
fn test_vector_dim_mismatch_error() {
    let mut index = VectorIndex::new(4);
    let id = Uuid::new_v4();
    let result = index.upsert(id, vec![1.0, 2.0]); // wrong dim
    assert!(result.is_err());
}

#[test]
fn test_vector_cosine_similarity_ordering() {
    let mut index = VectorIndex::new(384);
    let base = random_embedding(384);

    let id_similar = Uuid::new_v4();
    let id_different = Uuid::new_v4();

    index.upsert(id_similar, similar_embedding(&base, 0.05)).unwrap();
    index.upsert(id_different, orthogonal_embedding(384)).unwrap();

    let results = index.search_cosine(&base, 2);
    assert_eq!(results.len(), 2);
    // The similar embedding should rank above the orthogonal one
    assert_eq!(results[0].0, id_similar);
}

#[test]
fn test_vector_top_k_limit() {
    let mut index = VectorIndex::new(4);
    for _ in 0..10 {
        let id = Uuid::new_v4();
        index.upsert(id, vec![1.0, 0.0, 0.0, 0.0]).unwrap();
    }
    let results = index.search_cosine(&[1.0, 0.0, 0.0, 0.0], 3);
    assert_eq!(results.len(), 3);
}

#[test]
fn test_vector_remove() {
    let mut index = VectorIndex::new(4);
    let id = Uuid::new_v4();
    index.upsert(id, vec![1.0, 0.0, 0.0, 0.0]).unwrap();
    index.remove(id);
    let results = index.search_cosine(&[1.0, 0.0, 0.0, 0.0], 5);
    assert!(results.is_empty());
}
