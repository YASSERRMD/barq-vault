use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use barq_index::{Bm25Index, VectorIndex, HybridEngine, SearchParams};
use uuid::Uuid;
use serde_json::json;

fn bench_bm25_index(c: &mut Criterion) {
    let mut group = c.benchmark_group("index/bm25");

    for doc_count in [100usize, 1_000, 10_000] {
        let mut index = Bm25Index::new(1.5, 0.75);
        for i in 0..doc_count {
            let tokens: Vec<String> = vec![
                format!("token{}", i % 100),
                format!("word{}", i % 50),
                "common".to_string(),
            ];
            index.index_document(Uuid::new_v4(), &tokens);
        }

        group.bench_with_input(BenchmarkId::new("score", doc_count), &index, |b, idx| {
            b.iter(|| {
                black_box(idx.score(&["common".to_string(), "token5".to_string()], 10))
            })
        });
    }
    group.finish();
}

fn bench_vector_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("index/vector");

    for doc_count in [100usize, 1_000, 10_000] {
        let dim = 128;
        let mut index = VectorIndex::new(dim);
        for i in 0..doc_count {
            let emb: Vec<f32> = (0..dim).map(|j| ((i * j) as f32 / 1000.0).sin()).collect();
            index.upsert(Uuid::new_v4(), emb).unwrap();
        }
        let query: Vec<f32> = (0..dim).map(|i| (i as f32 / 100.0).cos()).collect();

        group.bench_with_input(BenchmarkId::new("cosine_k10", doc_count), &index, |b, idx| {
            b.iter(|| black_box(idx.search_cosine(&query, 10)))
        });
    }
    group.finish();
}

fn bench_hybrid_rrf(c: &mut Criterion) {
    let mut group = c.benchmark_group("index/hybrid_rrf");
    group.sample_size(20);

    let dim = 64;
    let mut engine = HybridEngine::new(1.5, 0.75, dim);
    for i in 0..1_000 {
        let id = Uuid::new_v4();
        let tokens = vec![format!("token{}", i % 100), "shared".to_string()];
        engine.bm25.index_document(id, &tokens);
        let emb: Vec<f32> = (0..dim).map(|j| (i as f32 * j as f32 / 100.0).sin()).collect();
        engine.vector.upsert(id, emb).unwrap();
    }
    let query_emb: Vec<f32> = (0..dim).map(|i| (i as f32 / 50.0).cos()).collect();

    group.bench_function("1000_docs_k10", |b| {
        b.iter(|| {
            engine.search(SearchParams {
                query_embedding: query_emb.clone(),
                query_text: "token42 shared".to_string(),
                vector_weight: 0.5,
                top_k: 10,
                modality_filter: None,
                metadata_filters: json!({}),
            }).unwrap()
        })
    });

    group.finish();
}

criterion_group!(benches, bench_bm25_index, bench_vector_search, bench_hybrid_rrf);
criterion_main!(benches);
