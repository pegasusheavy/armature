#!/bin/bash

# Sync documentation files from docs/ to web/public/docs/
# This ensures the website always has the latest documentation

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

SOURCE_DIR="$PROJECT_ROOT/docs"
DEST_DIR="$PROJECT_ROOT/web/public/docs"

# Create destination directory if it doesn't exist
mkdir -p "$DEST_DIR"

# Copy all markdown files
echo "Syncing documentation files..."
cp "$SOURCE_DIR"/*.md "$DEST_DIR/"

# Count files
SOURCE_COUNT=$(ls "$SOURCE_DIR"/*.md 2>/dev/null | wc -l)
DEST_COUNT=$(ls "$DEST_DIR"/*.md 2>/dev/null | wc -l)

echo "Source files: $SOURCE_COUNT"
echo "Destination files: $DEST_COUNT"

if [ "$SOURCE_COUNT" -eq "$DEST_COUNT" ]; then
    echo "✅ Documentation sync complete!"
else
    echo "⚠️  Warning: File counts don't match. Check for errors."
    exit 1
fi

