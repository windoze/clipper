# Build stage
FROM rust:1.91-bookworm AS builder

# Install build dependencies for RocksDB
RUN apt-get update && apt-get install -y \
    clang \
    libclang-dev \
    cmake \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy only the crates needed for clipper-server
COPY clipper-indexer ./clipper-indexer
COPY clipper-server ./clipper-server

# Create a minimal workspace Cargo.toml for the server build
RUN echo '[workspace]\nmembers = ["clipper-server", "clipper-indexer"]\nresolver = "2"' > Cargo.toml

# Build release binary
RUN cargo build --release -p clipper-server

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/clipper-server /app/clipper-server

# Create data directories
RUN mkdir -p /data/db /data/storage

# Set environment variables
ENV CLIPPER_DB_PATH=/data/db
ENV CLIPPER_STORAGE_PATH=/data/storage
ENV CLIPPER_LISTEN_ADDR=0.0.0.0
ENV PORT=3000
ENV RUST_LOG=clipper_server=info,tower_http=info

# Expose the server port
EXPOSE 3000

# Define volume for persistent data
VOLUME ["/data"]

# Run the server
CMD ["/app/clipper-server"]
