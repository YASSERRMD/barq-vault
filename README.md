# barq-vault

> **AI-native multimodal retrieval vault** — store, compress, and semantically search any file type using BM25 + vector hybrid search over a RocksDB backend with C++ FFI compression.

![CI](https://github.com/YASSERRMD/barq-vault/actions/workflows/ci.yml/badge.svg)
![Rust](https://img.shields.io/badge/rust-1.82+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

## Overview

barq-vault is a high-performance, AI-native multimodal retrieval system built in Rust. It supports ingesting text, images, audio, video, and documents with automatic OCR, STT, VLM captioning, LLM summarization, and vector embedding. Data is stored in RocksDB with C++ LZMA/LZ4/Zstd compression and searched via a hybrid BM25 + cosine vector index with Reciprocal Rank Fusion (RRF).

## Architecture

- **barq-types** — Shared domain types
- **barq-compress** — C++ FFI compression (LZMA, LZ4, delta-f32 + Zstd)
- **barq-store** — RocksDB storage engine (4 column families)
- **barq-index** — BM25 + vector + metadata + RRF hybrid engine
- **barq-ingest** — OCR, STT, VLM, LLM summarization, embedding pipeline
- **barq-proto** — Protobuf / tonic gRPC definitions
- **barq-server** — gRPC + REST server
- **barq-client** — Rust SDK
- **barq-cli** — Command-line interface

## Quick Start

```bash
cp .env.example .env
# Fill in your API keys
cargo build --release -p barq-server
./target/release/barq-server
```

## License

MIT
