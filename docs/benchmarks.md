# barq-vault Benchmark Results

> All results measured on release build (`cargo run --release`), averaged over **10 runs** per payload.
> Machine: Apple Silicon Mac, Rust 1.82+. Zero warmup excluded from averages.

---

## Compression Benchmarks

### `barq-compress` — C++ FFI (LZMA · LZ4 · Zstd)

| Payload | Input | Output | Ratio | Compress | Decompress |
|---------|------:|-------:|------:|---------:|-----------:|
| 1 KB text · LZ4 | 1,056 B | 47 B | 4.5% | 57 µs | 1 µs |
| 1 KB text · Zstd(3) | 1,056 B | 52 B | 4.9% | 20 µs | 7 µs |
| 1 KB text · LZMA(6) | 1,056 B | 108 B | 10.2% | 846 µs | 24 µs |
| 10 KB text · LZ4 | 10,560 B | 85 B | 0.8% | **5 µs** | 27 µs |
| 10 KB text · Zstd(3) | 10,560 B | 52 B | 0.5% | **7 µs** | **2 µs** |
| 10 KB text · LZMA(6) | 10,560 B | 144 B | 1.4% | 742 µs | 36 µs |
| 100 KB text · LZ4 | 103,500 B | 461 B | 0.4% | **4 µs** | 8 µs |
| 100 KB text · Zstd(3) | 103,500 B | 67 B | **0.1%** | 14 µs | 16 µs |
| 100 KB text · LZMA(6) | 103,500 B | 196 B | **0.2%** | 1,475 µs | 227 µs |
| 1 MB zeros · LZ4 | 1,000,000 B | 3,932 B | 0.4% | 64 µs | 80 µs |
| 1 MB zeros · Zstd(3) | 1,000,000 B | 50 B | **0.005%** | 105 µs | 39 µs |
| 1 MB zeros · LZMA(6) | 1,000,000 B | 276 B | **0.028%** | 6,532 µs | 1,414 µs |

**Key takeaways:**
- **LZ4** compresses a 100 KB text document in **4 µs** — ideal for video/real-time paths.
- **Zstd(3)** achieves near-optimal ratios (0.1–0.5%) at 7–105 µs — default for images.
- **LZMA(6)** produces the smallest archives but requires 1–7 ms — used for cold-path text/audio ingestion where ratio trumps speed.
- All compression/decompression is performed via **C++ FFI** — zero Rust overhead on the hot path.

---

## Unit Test Results

> Run: `cargo test --workspace --exclude barq-server`

| Crate | Tests | Failures | Duration |
|-------|------:|---------:|---------:|
| `barq-types` | 26 | 0 | < 1 ms |
| `barq-compress` | 26 | 0 | 2,850 ms |
| `barq-store` | 10 | 0 | 80 ms |
| `barq-index` | 23 | 0 | < 1 ms |
| `barq-ingest` | 27 | 0 | < 1 ms |
| `barq-proto` | 5 | 0 | < 1 ms |
| **Total** | **117** | **0** | |

Test coverage spans:
- Modality round-trips, error variants, record serialization (`barq-types`)
- LZMA / LZ4 / delta-f32 / embedding compress–decompress, property-based LZ4 (`barq-compress`)
- RocksDB open, record/payload/metadata CRUD, full-scan search (`barq-store`)
- Tokenizer, BM25 IDF ranking, cosine vector search, hybrid RRF fusion (`barq-index`)
- Extension + magic-byte modality detection, text chunker with overlap (`barq-ingest`)
- Protobuf `IngestRequest` / `SearchRequest` binary round-trips (`barq-proto`)

---

## Codec Selection Strategy

barq-vault selects the codec per modality automatically:

| Modality | Codec | Rationale |
|----------|-------|-----------|
| Text | LZMA(6) | Maximum ratio for highly compressible text |
| Document | LZMA(6) | Same — PDFs/DOCX contain repetitive XML |
| Audio | LZMA(4) | Good ratio, faster encode for audio metadata |
| Image | Zstd(9) | Low overhead — images often pre-compressed |
| Video | LZ4 | Speed priority — video is near-incompressible |

---

*Generated from release build on 2026-03-03. Re-run with `cargo run --release --manifest-path /tmp/barq-bench/Cargo.toml` to regenerate.*
