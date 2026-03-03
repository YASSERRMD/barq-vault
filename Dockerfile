# Stage 1: Build environment
FROM rust:1.80-slim-bookworm AS builder

# Install system dependencies required for Rust crates (RocksDB, Protobuf, compression codecs)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential \
    cmake \
    libclang-dev \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the entire workspace
COPY . .

# Build the server and cli in release mode
RUN cargo build --release -p barq-server
RUN cargo build --release -p barq-cli

# Stage 2: Production runtime environment
FROM debian:bookworm-slim

# Install runtime dependencies (OpenSSL, ca-certificates for outgoing HTTPS requests to LLMs)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binaries from the builder stage
COPY --from=builder /app/target/release/barq-server /usr/local/bin/barq-server
COPY --from=builder /app/target/release/barq-cli /usr/local/bin/barq-cli

# Copy default config (can be overridden by environment variables)
COPY config/ /app/config/

# Ensure the local data directory exists
RUN mkdir -p /data

# Default environment configuration
ENV BARQ_ENV=production
ENV RUST_LOG=info
ENV BARQ_SERVER__STORE_PATH=/data

# Expose gRPC port and REST API port
EXPOSE 50051
EXPOSE 8080

# Run the server binary by default
CMD ["barq-server"]
