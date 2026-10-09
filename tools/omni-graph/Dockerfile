# Multi-stage Dockerfile for Omni-Graph Rust Orchestrator
# Architecture: linux/arm64 native on Apple Silicon

# ─── Builder Stage ────────────────────────────────────────────────────────────
FROM rust:latest AS builder

WORKDIR /app

# Install native compilation dependencies for tree-sitter C grammar bindings
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    clang \
    libclang-dev \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy manifest files
COPY Cargo.toml Cargo.lock* ./

# Copy source tree and tests
COPY src ./src
COPY tests ./tests

# Build production release binary
RUN cargo build --release

# ─── Runtime Stage ────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install minimal runtime dependencies (ca-certificates for HTTPS, curl for healthchecks)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/omni-graph /usr/local/bin/omni-graph

# Copy default schema
COPY src/db/schema.surql /app/schema.surql

EXPOSE 8080

ENV RUST_LOG=info
ENV PORT=8080

CMD ["omni-graph"]
