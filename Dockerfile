# =============================================================================
# Stage 1: Build Web UI (platform-independent, runs on build host)
# =============================================================================
FROM --platform=$BUILDPLATFORM node:22-bookworm-slim AS web-builder

WORKDIR /app

# Copy shared UI package and install its dependencies first
COPY packages/clipper-ui/package*.json ./packages/clipper-ui/
WORKDIR /app/packages/clipper-ui
RUN npm ci

# Copy the rest of the shared UI source
COPY packages/clipper-ui/ ./

# Copy web UI package.json and install dependencies
WORKDIR /app
COPY clipper-server/web/package*.json ./clipper-server/web/

WORKDIR /app/clipper-server/web

# Install dependencies (including the local clipper-ui package)
RUN npm ci

# Copy the rest of the web source
COPY clipper-server/web/ ./

# Build the web UI
RUN npm run build

# =============================================================================
# Stage 2: Build Rust Binary with embedded Web UI
# Uses native compilation per architecture (via QEMU emulation in buildx)
# =============================================================================
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

# Copy the built web UI from the web-builder stage
COPY --from=web-builder /app/clipper-server/web/dist ./clipper-server/web/dist

# Create a minimal workspace Cargo.toml for the server build
RUN echo '[workspace]\nmembers = ["clipper-server", "clipper-indexer"]\nresolver = "2"' > Cargo.toml

# Build release binary with embedded web UI
RUN cargo build --release -p clipper-server --features embed-web

# =============================================================================
# Stage 3: Runtime (using distroless for minimal size ~20MB base)
# =============================================================================
FROM gcr.io/distroless/cc-debian12:nonroot

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/clipper-server /app/clipper-server

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

# Run as nonroot user (UID 65532)
USER nonroot:nonroot

# Run the server
ENTRYPOINT ["/app/clipper-server"]
