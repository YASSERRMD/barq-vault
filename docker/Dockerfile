# Stage 1: Builder
FROM rust:1.82-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    liblzma-dev \
    liblz4-dev \
    clang \
    libclang-dev \
    cmake \
    pkg-config \
    libssl-dev \
    tesseract-ocr \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/barq-vault

# Copy the entire workspace
COPY . .

# Build the release binary for barq-server
RUN cargo build --release -p barq-server

# Stage 2: Runtime
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies only
RUN apt-get update && apt-get install -y \
    liblzma5 \
    liblz4-1 \
    libssl3 \
    ca-certificates \
    tesseract-ocr \
    ffmpeg \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/barq-vault/target/release/barq-server .

# Copy default configurations
COPY --from=builder /usr/src/barq-vault/config /etc/barq-vault/config

# Create a non-root user and setup data directory
RUN useradd -m -s /bin/bash barq && \
    mkdir -p /var/barq-vault/data && \
    chown -R barq:barq /var/barq-vault /etc/barq-vault

# Switch to non-root user
USER barq

# Expose gRPC and REST ports
EXPOSE 50051 8080

# Environment variables
ENV BARQ_ENV=production
ENV RUST_LOG=info
# Tell the app where to find its config
ENV BARQ__SERVER__STORE_PATH=/var/barq-vault/data

# Run the server
ENTRYPOINT ["./barq-server"]
