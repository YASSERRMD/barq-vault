# barq-vault

> **AI-native multimodal retrieval vault** — ingest any file, store it compressed, search it semantically.

[![CI](https://github.com/YASSERRMD/barq-vault/actions/workflows/ci.yml/badge.svg)](https://github.com/YASSERRMD/barq-vault/actions/workflows/ci.yml)
![Rust](https://img.shields.io/badge/rust-1.82+-orange.svg)
![Tests](https://img.shields.io/badge/tests-117%20passing-brightgreen.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

---

## What is barq-vault?

barq-vault is a high-performance, production-grade multimodal retrieval system written in Rust. It accepts **text, PDF, images, audio, and video**, runs automatic extraction (OCR / STT / VLM captioning), summarizes content with any LLM, embeds it with any embedding provider, and stores everything in a RocksDB backend with C++ FFI compression.

Search is powered by a **hybrid BM25 + cosine vector engine with Reciprocal Rank Fusion (RRF)** — giving you the best of lexical and semantic retrieval without any external vector database.

---

## Key Features

| Feature | Detail |
|---------|--------|
| **Multi-modal ingestion** | Text · PDF · Images · Audio · Video |
| **Extraction** | OCR (Tesseract), STT (Whisper), VLM captioning, plain text |
| **Summarization** | OpenAI · Mistral · Gemini · Ollama — pluggable |
| **Embedding** | OpenAI · Mistral · Gemini · Ollama · local — pluggable |
| **Compression** | C++ LZMA · LZ4 · delta-f32+Zstd via FFI |
| **Storage** | RocksDB with 4 column families |
| **Search** | BM25 + cosine vector + metadata filter → RRF |
| **Transport** | gRPC (tonic) + REST (axum) |
| **CLI** | `barq-cli` with full ingest / search / fetch / config commands |
| **Docker** | Multi-stage Dockerfile + docker-compose |

---

## Architecture

```
barq-vault workspace
│
├── barq-types       — Shared domain types (BarqRecord, Modality, StorageMode, errors)
├── barq-compress    — C++ FFI compression (LZMA, LZ4, delta-f32 + Zstd)
├── barq-store       — RocksDB storage engine (records · payloads · metadata · embeddings)
├── barq-index       — BM25 + VectorIndex + MetadataIndex + HybridEngine (RRF)
├── barq-ingest      — Modality detector → extractor → summarizer → embedder → pipeline
├── barq-proto       — Protobuf / tonic gRPC definitions + type converters
├── barq-server      — gRPC + REST server with global AppState
├── barq-client      — Rust SDK (BarqVaultClient)
├── barq-cli         — clap CLI: ingest / search / fetch / delete / stats / config
└── barq-test-utils  — Shared test fixtures, mock builders, wiremock servers, assertions
```

---

## Compression Performance

Measured on release build, averaged over 10 runs. See [`docs/benchmarks.md`](docs/benchmarks.md) for full results.

| Payload | Codec | Ratio | Compress | Decompress |
|---------|-------|------:|---------:|-----------:|
| 10 KB text | LZ4 | 0.8% | **5 µs** | 27 µs |
| 10 KB text | Zstd(3) | 0.5% | **7 µs** | **2 µs** |
| 100 KB text | LZMA(6) | 0.2% | 1,475 µs | 227 µs |
| 1 MB zeros | Zstd(3) | 0.005% | 105 µs | 39 µs |

Codec is selected automatically per modality (Text → LZMA · Image → Zstd · Video → LZ4).

---

## Test Coverage

117 tests, 0 failures across the full workspace.

| Crate | Tests | What's covered |
|-------|------:|----------------|
| `barq-types` | 26 | Modality round-trips, record JSON/binary, error variants |
| `barq-compress` | 26 | LZMA / LZ4 / Zstd round-trips, delta-f32, embedding codec, property-based |
| `barq-store` | 10 | RocksDB open, full record/payload/metadata CRUD, metadata search |
| `barq-index` | 23 | Tokenizer, BM25 IDF/TF, cosine vector search, hybrid RRF fusion |
| `barq-ingest` | 27 | Extension + magic-byte detection (PDF/PNG/WAV), text chunker with overlap |
| `barq-proto` | 5 | Protobuf IngestRequest / SearchRequest binary round-trips |

```bash
cargo test --workspace --exclude barq-server
# test result: ok. 117 passed; 0 failed
```

---

## Quick Start

### With Docker (recommended)

```bash
cp .env.example .env
# Fill in your API keys (OPENAI_API_KEY, etc.)
docker compose up --build
```

The server will be available at:
- gRPC: `localhost:50051`
- REST: `localhost:8080`

### From source

```bash
cp .env.example .env
cargo build --release -p barq-server
./target/release/barq-server
```

---

## CLI Usage

```bash
# Ingest a file
barq-cli ingest ./report.pdf \
  --llm-provider openai --llm-key $OPENAI_API_KEY \
  --embed-provider openai --embed-key $OPENAI_API_KEY \
  --storage-mode hybrid_file

# Search
barq-cli search "quarterly earnings summary" --top-k 5

# Fetch by ID
barq-cli fetch <uuid> --stdout

# Print resolved config
barq-cli config show
```

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `BARQ_STORE_PATH` | RocksDB data directory (default: `./data`) |
| `BARQ_GRPC_ADDR` | gRPC listen address (default: `0.0.0.0:50051`) |
| `BARQ_REST_ADDR` | REST listen address (default: `0.0.0.0:8080`) |
| `OPENAI_API_KEY` | OpenAI key for LLM / embedding |
| `MISTRAL_API_KEY` | Mistral API key |
| `GEMINI_API_KEY` | Google Gemini API key |

See `.env.example` for the complete list.

---

## License

MIT © YASSERRMD
