#!/bin/bash
set -e

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLIPPER_DIR="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$CLIPPER_DIR")"

# Get the target triple
if [ -n "$TAURI_ENV_TARGET_TRIPLE" ]; then
    # Use Tauri's target triple if available (set during tauri build)
    TARGET_TRIPLE="$TAURI_ENV_TARGET_TRIPLE"
elif [ -n "$1" ]; then
    # Use provided argument
    TARGET_TRIPLE="$1"
else
    # Detect current host
    TARGET_TRIPLE=$(rustc --print host-tuple)
fi

echo "Building clipper-server for target: $TARGET_TRIPLE"

# Build clipper-server
cd "$PROJECT_ROOT"

if [ "$TARGET_TRIPLE" = "$(rustc --print host-tuple)" ]; then
    # Native build
    cargo build --release -p clipper-server
    SOURCE_BINARY="$PROJECT_ROOT/target/release/clipper-server"
else
    # Cross-compile
    cargo build --release -p clipper-server --target "$TARGET_TRIPLE"
    SOURCE_BINARY="$PROJECT_ROOT/target/$TARGET_TRIPLE/release/clipper-server"
fi

# Add .exe suffix for Windows targets
if [[ "$TARGET_TRIPLE" == *"windows"* ]]; then
    SOURCE_BINARY="${SOURCE_BINARY}.exe"
    DEST_BINARY="$CLIPPER_DIR/src-tauri/binaries/clipper-server-${TARGET_TRIPLE}.exe"
else
    DEST_BINARY="$CLIPPER_DIR/src-tauri/binaries/clipper-server-${TARGET_TRIPLE}"
fi

# Ensure binaries directory exists
mkdir -p "$CLIPPER_DIR/src-tauri/binaries"

# Copy binary
cp "$SOURCE_BINARY" "$DEST_BINARY"

echo "Copied clipper-server to: $DEST_BINARY"
