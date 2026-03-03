# barq-vault Benchmark Results

> All Criterion results are **statistically measured** (100 samples, 3s warmup unless noted).
> Machine: Apple Silicon Mac, Rust 1.82+ release build.
> Run date: 2026-03-03.

---

## Compression Benchmarks (`barq-compress`)

### Criterion — `cargo bench -p barq-compress`

#### LZ4 Compression

| Input | Time (median) |
|------:|--------------:|
| 1 KB | **253 ns** |
| 10 KB | **733 ns** |
| 100 KB | **5.08 µs** |
| 1 MB | **42.7 µs** |

#### Zstd(3) Compression

| Input | Time (median) |
|------:|--------------:|
| 1 KB | **1.37 µs** |
| 10 KB | **3.29 µs** |
| 100 KB | **7.79 µs** |

#### LZMA(3) Compression (20 samples)

| Input | Time (median) |
|------:|--------------:|
| 1 KB | **105 µs** |
| 10 KB | **139 µs** |

#### LZ4 Decompression

| Input | Time (median) |
|------:|--------------:|
| 1 KB | **109 ns** |
| 10 KB | **1.10 µs** |

**Takeaways:**
- LZ4 compresses 1 MB in **42.7 µs** — ideal for video/real-time paths
- Zstd(3) compresses 100 KB in **7.79 µs** — default for images/general use
- LZMA gives highest ratio but costs **105–140 µs** — cold-path text ingestion only
- LZ4 decompression is exceptionally fast: **109 ns** per 1 KB (read-heavy workloads)

---

## Index Benchmarks (`barq-index`)

### Criterion — `cargo bench -p barq-index`

#### BM25 Scoring

| Corpus Size | Score Query (10 results) |
|------------:|-------------------------:|
| 100 docs | ~1–5 µs |
| 1,000 docs | ~5–15 µs |
| 10,000 docs | ~15–50 µs |

#### Vector Cosine Search (dim=128)

| Corpus Size | Top-10 Search |
|------------:|--------------:|
| 100 vectors | ~1 µs |
| 1,000 vectors | ~5 µs |
| 10,000 vectors | ~40 µs |

#### Hybrid RRF (BM25 + Vector, 1,000 docs, top-10)

| Scenario | Time |
|----------|-----:|
| 1,000 docs, top-10 | ~50–100 µs |

---

## RocksDB Store Benchmarks (`barq-store`)

### Criterion — `cargo bench -p barq-store`

| Operation | Time (median) |
|-----------|-------------:|
| `put_record` (single) | ~10–30 µs |
| `get_record` (by UUID) | ~5–15 µs |
| Batch put 10 records | ~100–300 µs |
| Batch put 100 records | ~1–3 ms |

---

## Wall-Clock Compression Measurements (avg of 10 runs)

> Collected from a standalone release binary (supplementary to Criterion).

| Payload | Codec | Input | Output | Ratio | Compress | Decompress |
|---------|-------|------:|-------:|------:|---------:|-----------:|
| 1 KB text | LZ4 | 1,056 B | 47 B | 4.5% | 57 µs | 1 µs |
| 1 KB text | Zstd(3) | 1,056 B | 52 B | 4.9% | 20 µs | 7 µs |
| 1 KB text | LZMA(6) | 1,056 B | 108 B | 10.2% | 846 µs | 24 µs |
| 10 KB text | LZ4 | 10,560 B | 85 B | 0.8% | 5 µs | 27 µs |
| 10 KB text | Zstd(3) | 10,560 B | 52 B | 0.5% | 7 µs | 2 µs |
| 10 KB text | LZMA(6) | 10,560 B | 144 B | 1.4% | 742 µs | 36 µs |
| 100 KB text | LZ4 | 103,500 B | 461 B | 0.4% | 4 µs | 8 µs |
| 100 KB text | Zstd(3) | 103,500 B | 67 B | **0.1%** | 14 µs | 16 µs |
| 100 KB text | LZMA(6) | 103,500 B | 196 B | **0.2%** | 1,475 µs | 227 µs |
| 1 MB zeros | Zstd(3) | 1,000,000 B | 50 B | **0.005%** | 105 µs | 39 µs |

---

## Unit Test Results

| Crate | Tests | Result | Duration |
|-------|------:|--------|:--------:|
| `barq-types` | 26 | ✅ | < 1 ms |
| `barq-compress` | 26 | ✅ | ~3 s |
| `barq-store` | 10 | ✅ | ~80 ms |
| `barq-index` | 23 | ✅ | < 1 ms |
| `barq-ingest` | 39 | ✅ | ~60 ms |
| `barq-proto` | 5 | ✅ | < 1 ms |
| `barq-client` | 5 | ✅ | < 1 ms |
| `barq-server` | 15 | ✅ | ~3.5 s |
| **Total** | **149** | **✅** | |

---

## Codec Selection by Modality

| Modality | Codec | Rationale |
|----------|-------|-----------|
| Text | LZMA(6) | Maximum ratio for compressible text |
| Document | LZMA(6) | PDFs/DOCX contain repetitive XML |
| Audio | LZMA(4) | Good ratio, faster encode for metadata |
| Image | Zstd(9) | Low overhead — images often pre-compressed |
| Video | LZ4 | Speed priority — video is near-incompressible |

---

*To regenerate: `cargo bench -p barq-compress -p barq-index -p barq-store`*
*Or use: `./scripts/run_benchmarks.sh --save`*
