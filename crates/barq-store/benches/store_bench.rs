use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use barq_store::BarqStore;
use barq_types::{BarqRecord, Modality, StorageMode, CodecType};
use uuid::Uuid;
use tempfile::TempDir;
use serde_json::json;

fn make_record(i: usize) -> BarqRecord {
    BarqRecord {
        id: Uuid::new_v4(),
        parent_id: None,
        chunk_index: 0,
        total_chunks: 1,
        modality: Modality::Text,
        storage_mode: StorageMode::TextOnly,
        codec: CodecType::Lz4,
        filename: Some(format!("doc_{}.txt", i)),
        mime_type: Some("text/plain".into()),
        summary: format!("Summary for document number {}", i),
        embedding: vec![],
        compressed_embed: vec![],
        embedding_dim: 0,
        bm25_tokens: vec!["token".into(), format!("doc{}", i)],
        metadata: json!({"idx": i}),
        compressed_payload: None,
        original_size: 100,
        compressed_size: 0,
        compression_ratio: 0.0,
        created_at: 0,
        updated_at: 0,
        checksum: [0u8; 32],
    }
}

fn bench_store_put(c: &mut Criterion) {
    let dir = TempDir::new().unwrap();
    let store = BarqStore::open(dir.path().to_str().unwrap()).unwrap();

    c.bench_function("store/put_record", |b| {
        b.iter(|| {
            let record = make_record(0);
            store.put_record(black_box(&record)).unwrap()
        })
    });
}

fn bench_store_get(c: &mut Criterion) {
    let dir = TempDir::new().unwrap();
    let store = BarqStore::open(dir.path().to_str().unwrap()).unwrap();
    let record = make_record(42);
    let id = record.id;
    store.put_record(&record).unwrap();

    c.bench_function("store/get_record", |b| {
        b.iter(|| store.get_record(black_box(id)).unwrap())
    });
}

fn bench_store_put_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("store/put_batch");
    for n in [10usize, 100, 1_000] {
        let dir = TempDir::new().unwrap();
        let store = BarqStore::open(dir.path().to_str().unwrap()).unwrap();
        let records: Vec<BarqRecord> = (0..n).map(make_record).collect();

        group.bench_with_input(BenchmarkId::from_parameter(n), &records, |b, recs| {
            b.iter(|| {
                for r in recs {
                    store.put_record(black_box(r)).unwrap();
                }
            })
        });
    }
    group.finish();
}

criterion_group!(benches, bench_store_put, bench_store_get, bench_store_put_batch);
criterion_main!(benches);
