#!/bin/bash
# =============================================================================
# Cleanup script for gh-pages branch
# =============================================================================
# This script removes unnecessary files from the gh-pages deployment directory
# to keep the branch clean and minimize deployment size.
#
# Usage:
#   ./scripts/cleanup-gh-pages.sh [directory]
#
# If no directory is specified, defaults to ./public
# =============================================================================

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default directory
DEPLOY_DIR="${1:-./public}"

echo -e "${BLUE}🧹 Cleaning up gh-pages deployment directory: ${DEPLOY_DIR}${NC}"
echo ""

# Check if directory exists
if [ ! -d "$DEPLOY_DIR" ]; then
    echo -e "${RED}Error: Directory '$DEPLOY_DIR' does not exist${NC}"
    exit 1
fi

# Track removed files count
REMOVED_COUNT=0

# Function to remove files/directories matching a pattern
cleanup() {
    local pattern="$1"
    local description="$2"

    while IFS= read -r -d '' file; do
        if [ -e "$file" ]; then
            rm -rf "$file"
            echo -e "  ${YELLOW}✗${NC} Removed: ${file#$DEPLOY_DIR/}"
            ((REMOVED_COUNT++)) || true
        fi
    done < <(find "$DEPLOY_DIR" -name "$pattern" -print0 2>/dev/null || true)
}

# Function to remove files by extension
cleanup_ext() {
    local ext="$1"
    local description="$2"

    while IFS= read -r -d '' file; do
        if [ -e "$file" ]; then
            rm -rf "$file"
            echo -e "  ${YELLOW}✗${NC} Removed: ${file#$DEPLOY_DIR/}"
            ((REMOVED_COUNT++)) || true
        fi
    done < <(find "$DEPLOY_DIR" -name "*.$ext" -print0 2>/dev/null || true)
}

echo -e "${BLUE}Removing development and build artifacts...${NC}"

# Source maps (not needed in production unless debugging)
cleanup_ext "map" "Source maps"

# TypeScript files (should be compiled)
cleanup_ext "ts" "TypeScript source files"
cleanup "tsconfig*.json" "TypeScript configs"

# Lock files
cleanup "package-lock.json" "npm lock files"
cleanup "pnpm-lock.yaml" "pnpm lock files"
cleanup "yarn.lock" "yarn lock files"
cleanup "Cargo.lock" "Cargo lock files"

# Development configs
cleanup ".eslintrc*" "ESLint configs"
cleanup ".prettierrc*" "Prettier configs"
cleanup "eslint.config.*" "ESLint configs"
cleanup ".editorconfig" "Editor configs"
cleanup "angular.json" "Angular CLI config"
cleanup "vite.config.*" "Vite configs"
cleanup "vitest.config.*" "Vitest configs"
cleanup "jest.config.*" "Jest configs"
cleanup "karma.conf.*" "Karma configs"

# Git files
cleanup ".gitignore" "Git ignore files"
cleanup ".gitattributes" "Git attributes"
cleanup ".git" "Git directories"

# CI/CD files
cleanup ".github" "GitHub configs"
cleanup ".gitlab-ci.yml" "GitLab CI"
cleanup ".travis.yml" "Travis CI"
cleanup "Jenkinsfile" "Jenkins configs"

# IDE and editor files
cleanup ".vscode" "VS Code settings"
cleanup ".idea" "JetBrains IDE settings"
cleanup "*.swp" "Vim swap files"
cleanup "*.swo" "Vim swap files"
cleanup "*~" "Backup files"
cleanup ".DS_Store" "macOS files"
cleanup "Thumbs.db" "Windows thumbnails"

# Build/test directories that shouldn't be deployed
cleanup "node_modules" "Node modules"
cleanup "__pycache__" "Python cache"
cleanup ".angular" "Angular cache"
cleanup ".next" "Next.js cache"
cleanup "coverage" "Coverage reports"
cleanup ".nyc_output" "NYC output"
cleanup "dist" "Nested dist directories"
cleanup "target" "Rust target directories"

# Documentation source files (keep only built docs)
# Note: Keep .md files - the Angular website loads them for documentation
# cleanup "*.md" "Markdown files"
cleanup "*.rst" "RST files"

# Rust-specific (in API docs)
cleanup ".rustdoc_fingerprint.json" "Rustdoc fingerprint"
cleanup "COPYRIGHT" "Copyright files"
cleanup "LICENSE*" "License files in subdirs"

# Package files
cleanup "package.json" "package.json files"
cleanup "Cargo.toml" "Cargo.toml files"

# Test files
cleanup "*.spec.ts" "Test spec files"
cleanup "*.spec.js" "Test spec files"
cleanup "*.test.ts" "Test files"
cleanup "*.test.js" "Test files"
cleanup "__tests__" "Test directories"
cleanup "tests" "Test directories"

echo ""
echo -e "${BLUE}Removing empty directories...${NC}"

# Remove empty directories
EMPTY_DIRS=$(find "$DEPLOY_DIR" -type d -empty 2>/dev/null | wc -l)
find "$DEPLOY_DIR" -type d -empty -delete 2>/dev/null || true
if [ "$EMPTY_DIRS" -gt 0 ]; then
    echo -e "  ${YELLOW}✗${NC} Removed $EMPTY_DIRS empty directories"
fi

echo ""

# Calculate final size
FINAL_SIZE=$(du -sh "$DEPLOY_DIR" 2>/dev/null | cut -f1)

echo -e "${GREEN}✓ Cleanup complete!${NC}"
echo -e "  Files removed: ${REMOVED_COUNT}"
echo -e "  Final size: ${FINAL_SIZE}"
echo ""

# List remaining files (for verification)
if [ "${VERBOSE:-}" = "1" ]; then
    echo -e "${BLUE}Remaining files:${NC}"
    find "$DEPLOY_DIR" -type f | head -50

    TOTAL_FILES=$(find "$DEPLOY_DIR" -type f | wc -l)
    if [ "$TOTAL_FILES" -gt 50 ]; then
        echo "  ... and $((TOTAL_FILES - 50)) more files"
    fi
fi

