#!/bin/sh
# Wrapper script for clipper-server with backup/restore support
#
# Environment variables:
#   CLIPPER_BACKUP_ON_EXIT=true     - Create backup on shutdown
#   CLIPPER_RESTORE_ON_START=true   - Restore from backup on startup if db is empty
#   CLIPPER_BACKUP_PATH=/data/backup.tar.gz - Path to backup file
#   CLIPPER_INCLUDE_FILES=true      - Include storage files in backup (default: false)
#
# All other CLIPPER_* variables are passed through to clipper-server

set -e

BACKUP_PATH="${CLIPPER_BACKUP_PATH:-/data/backup.tar.gz}"
DB_PATH="${CLIPPER_DB_PATH:-/data/db}"
STORAGE_PATH="${CLIPPER_STORAGE_PATH:-/data/storage}"
INCLUDE_FILES="${CLIPPER_INCLUDE_FILES:-false}"

# Function to create backup
create_backup() {
    echo "Creating backup at ${BACKUP_PATH}..."

    # Create a temporary directory for staging
    TEMP_DIR=$(mktemp -d)
    trap "rm -rf ${TEMP_DIR}" EXIT

    if [ "$INCLUDE_FILES" = "true" ] && [ -d "$STORAGE_PATH" ]; then
        # Backup both db and storage
        tar -czf "${BACKUP_PATH}" -C "$(dirname "$DB_PATH")" "$(basename "$DB_PATH")" -C "$(dirname "$STORAGE_PATH")" "$(basename "$STORAGE_PATH")" 2>/dev/null || \
        tar -czf "${BACKUP_PATH}" -C "$(dirname "$DB_PATH")" "$(basename "$DB_PATH")"
        echo "Backup created (including files)"
    else
        # Backup only db
        tar -czf "${BACKUP_PATH}" -C "$(dirname "$DB_PATH")" "$(basename "$DB_PATH")"
        echo "Backup created (database only)"
    fi
}

# Function to restore from backup
restore_backup() {
    if [ ! -f "$BACKUP_PATH" ]; then
        echo "No backup file found at ${BACKUP_PATH}, skipping restore"
        return 0
    fi

    # Check if database directory is empty or doesn't exist
    if [ -d "$DB_PATH" ] && [ "$(ls -A "$DB_PATH" 2>/dev/null)" ]; then
        echo "Database directory is not empty, skipping restore"
        return 0
    fi

    echo "Restoring from backup at ${BACKUP_PATH}..."

    # Create parent directories
    mkdir -p "$(dirname "$DB_PATH")"
    mkdir -p "$(dirname "$STORAGE_PATH")"

    # Extract backup
    # The backup contains paths relative to /data, so extract to /data
    tar -xzf "${BACKUP_PATH}" -C "$(dirname "$DB_PATH")"

    echo "Restore completed"
}

# Handle shutdown signal
shutdown_handler() {
    echo "Received shutdown signal"

    # Wait for server to stop (it should handle SIGTERM gracefully)
    wait $SERVER_PID 2>/dev/null || true

    if [ "$CLIPPER_BACKUP_ON_EXIT" = "true" ]; then
        create_backup
    fi

    exit 0
}

# Set up signal handlers
trap shutdown_handler SIGTERM SIGINT

# Restore on start if enabled
if [ "$CLIPPER_RESTORE_ON_START" = "true" ]; then
    restore_backup
fi

# Start the server in background so we can handle signals
/app/clipper-server "$@" &
SERVER_PID=$!

# Wait for the server process
wait $SERVER_PID
EXIT_CODE=$?

# Create backup on exit if enabled (for normal exit)
if [ "$CLIPPER_BACKUP_ON_EXIT" = "true" ]; then
    create_backup
fi

exit $EXIT_CODE
