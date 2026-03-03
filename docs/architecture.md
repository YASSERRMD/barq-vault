# barq-vault Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                          CLIENTS                                │
│   barq-cli     barq-client SDK     REST (curl/web)             │
└────────┬────────────────┬────────────────┬──────────────────────┘
         │  gRPC (50051)  │                │ HTTP/REST (8080)
         ▼                ▼                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      barq-server                                │
│   ┌──────────────┐            ┌────────────────────────────┐   │
│   │  tonic gRPC  │            │      axum REST router       │   │
│   │  + auth      │            │      + auth middleware       │   │
│   └──────┬───────┘            └───────────┬────────────────┘   │
│          └────────────┬───────────────────┘                    │
│                       ▼                                         │
│              ┌──────────────────┐                               │
│              │    AppState      │                               │
│              │ store / index /  │                               │
│              │ ingest_pipeline  │                               │
│              └────────┬─────────┘                               │
└───────────────────────┼─────────────────────────────────────────┘
                        │
         ┌──────────────┼──────────────────────┐
         ▼              ▼                       ▼
  ┌─────────────┐ ┌──────────────┐  ┌───────────────────────┐
  │ barq-store  │ │  barq-index  │  │     barq-ingest        │
  │  RocksDB   │ │ BM25+vector  │  │ OCR/STT/VLM/LLM/embed │
  │  4 CFs     │ │  +meta+RRF   │  │      pipeline          │
  └─────────────┘ └──────────────┘  └───────────────────────┘
         │                                       │
         ▼                                       ▼
  ┌─────────────┐                    ┌───────────────────────┐
  │barq-compress│                    │    barq-compress       │
  │ C++ LZMA   │                    │  (codec selection)     │
  │ LZ4 delta  │                    └───────────────────────┘
  └─────────────┘
```

---

## Data Flows

### Ingestion Flow

```
File Input
   │
   ▼
detect_modality(filename, magic_bytes)
   │
   ▼
Text Extraction (modality-specific):
  Text/Doc  → PlainTextExtractor / DocumentExtractor
  Image     → OcrExtractor (tesseract) + VlmExtractor (Gemini/GPT-4V)
  Audio     → SttExtractor (whisper local / OpenAI)
  Video     → VlmExtractor (keyframes) + SttExtractor (audio track)
   │
   ▼
Chunker.chunk_text()  (sentence-boundary, with overlap)
   │
   ├── for each chunk:
   │     LLM Summarizer → summary string
   │     Embedder → Vec<f32> (1536-dim)
   │     tokenize(summary) → BM25 tokens
   │
   ▼
barq-compress: select_codec_for_modality()
  Text/Doc  → LZMA level 6
  Audio     → LZMA level 4
  Image     → Zstd level 9
  Video     → LZ4
  Embeddings→ delta-f32 + Zstd level 3
   │
   ▼
blake3::hash(original_bytes) → checksum [u8; 32]
   │
   ▼
Assemble BarqRecord(s) — one per chunk
   │
   ├── barq-store: put_record() + put_payload()
   └── barq-index: BM25 + vector + metadata index_new()
```

### Query / Search Flow

```
Query String
   │
   ▼
Embedder.embed(query_text) → Vec<f32>
   │
   ▼
HybridEngine.search(SearchParams):
   ├── BM25.score(tokenize_query(text)) → ranked BM25 list
   ├── VectorIndex.search_cosine(query_emb) → ranked vector list
   └── RRF fusion:
         score = w × 1/(k+rank_v) + (1-w) × 1/(k+rank_b)
         where k=60, w=vector_weight
   │
   ▼
Apply modality / metadata filters
   │
   ▼
Fetch BarqRecord for each top-K result from store
   │
   ▼
Optionally: get_payload + decompress
   │
   ▼
Return SearchResponse[ { id, summary, score, filename, … } ]
```

---

## Crate Responsibilities

### barq-types
Pure shared domain types with zero dependencies on other barq crates. Defines `BarqRecord`, `Modality`, `StorageMode`, `CodecType`, all API request/response structs, and `BarqError`/`BarqResult`. All other crates depend on this crate.

### barq-compress
C++ FFI compression engine wrapping LZMA (liblzma), LZ4 (liblz4), and Zstd. Implements AVX2-accelerated delta-encoding for f32 embedding vectors before Zstd compression. Exposes a safe Rust API with codec dispatch and modality-aware codec selection. Built via `build.rs` using the `cc` and `bindgen` crates.

### barq-store
RocksDB-backed storage engine with 4 column families: `CF_RECORDS` (serialized BarqRecord metadata), `CF_PAYLOADS` (compressed raw bytes), `CF_METADATA` (JSON tags), `CF_INDEX_META` (index housekeeping). Provides CRUD operations for records, payloads, and metadata. Used at startup for index bootstrap.

### barq-index
In-memory hybrid search index combining BM25 inverted index (configurable k1/b), cosine-similarity vector index (in-memory brute-force, HNSW planned), and a metadata index for modality/tag filtering. The `HybridEngine` fuses BM25 and vector results via Reciprocal Rank Fusion. `IndexManager` bootstraps from RocksDB on server startup.

### barq-ingest
Multi-modal ingestion pipeline. Orchestrates: modality detection → text extraction (plain text, document parsing, OCR via tesseract, STT via whisper/OpenAI API, VLM captioning via Gemini/GPT-4V/Ollama) → chunking → LLM summarization (OpenAI/Mistral/Gemini/Anthropic/Ollama) → embedding generation (OpenAI/Cohere/Mistral/Ollama) → compression → record assembly. All provider calls include timeout, retry with exponential backoff, and structured error propagation.

### barq-proto
Tonic/protobuf definitions and generated code for the `BarqVault` gRPC service. Contains the `.proto` build pipeline and domain ↔ proto type converters. Re-exports server and client traits generated by tonic.

### barq-server
Production binary: boots `AppState` (opens store, bootstraps indexes), starts tonic gRPC server (port 50051) and axum REST server (port 8080) concurrently. Implements all gRPC RPC handlers and REST route handlers. Includes bearer-token auth interceptor for gRPC and axum middleware for REST. Supports optional mTLS. Handles graceful shutdown on SIGTERM/Ctrl-C with 30-second in-flight drain.

### barq-client
Rust SDK providing a transport-agnostic `BarqVaultClient` API. Supports gRPC transport (via tonic) and REST transport (via reqwest). Provides high-level methods: `ingest_file`, `ingest_text`, `search`, `fetch`, `delete`. Runs the ingest pipeline locally before uploading embeddings and summaries.

### barq-cli
`clap`-based command-line interface exposing `ingest`, `search`, `fetch`, `delete`, `ping`, `stats`, and `config show` subcommands. Builds `ClientConfig` from CLI flags + environment variables and delegates to `barq-client`.

---

## Protocol Choices

**gRPC (primary)**: Binary framing via protobuf reduces payload size by 30–60% vs JSON. Native streaming for large file ingest (`IngestStream`) and file fetch (`Fetch` → chunked download). mTLS support via tonic's TLS stack. Strongly typed service contracts enforceable at compile time.

**REST (secondary)**: JSON over HTTP/1.1 for web clients and simple integrations. Axum provides async handlers with zero-copy streaming for fetch. The `/v1/health` endpoint is exempt from auth for load-balancer probes.

---

## Storage Rationale

**RocksDB LSM tree** selected for:
- Append-heavy write patterns (new records are always appended, rarely updated)
- Key-range iteration needed for index bootstrap on startup
- Column family isolation: metadata CF can be inspected independently without deserializing payload blobs
- Built-in Zstd block-level compression at the SST layer (double-compression intentionally avoided for payloads since they are already compressed)

**4 Column Families**:
- `CF_RECORDS` — light record metadata (bincode ~500 bytes/record), hot read path for search result enrichment
- `CF_PAYLOADS` — compressed binary blobs (cold, rarely read unless fetch requested)
- `CF_METADATA` — JSON tags (hot for metadata filter queries)
- `CF_INDEX_META` — index housekeeping, reserved for future persistent index support

---

## Compression Rationale

| Modality | Codec | Rationale |
|----------|-------|-----------|
| Text / Document | LZMA level 6 | Maximum ratio for text; 10–100× compression common |
| Audio (raw/PCM) | LZMA level 4 | Good ratio with faster encode than level 9 |
| Image | Zstd level 9 | Already partially compressed (JPEG/PNG); Zstd overhead minimal |
| Video | LZ4 | Speed priority; video is near-incompressible; LZ4 adds framing only |
| Embeddings | delta-f32 + Zstd 3 | Delta encoding decorrelates adjacent float values; Zstd then achieves 3–5× compression on residuals |

LZMA (via liblzma/XZ) is the gold standard for compressible text data with ratios 10–100×. LZ4 is the fastest compressor available (~500 MB/s encode) and appropriate when data entropy is already high. Zstd level 3 provides the best speed/ratio tradeoff for moderately structured binary data like embedding matrices.
