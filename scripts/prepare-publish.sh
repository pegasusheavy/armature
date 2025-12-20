#!/bin/bash
# =============================================================================
# Prepare Armature Crates for Publishing
# =============================================================================
#
# This script prepares the workspace for publishing to crates.io by:
# 1. Converting path dependencies to version dependencies
# 2. Ensuring all required fields are present
# 3. Creating a backup of original Cargo.toml files
#
# Usage:
#   ./scripts/prepare-publish.sh [OPTIONS]
#
# Options:
#   --version VERSION  Set the version for all crates (default: from workspace)
#   --restore          Restore original Cargo.toml files from backup
#   --dry-run          Show changes without applying them
#   --help             Show this help message
#
# =============================================================================

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
VERSION=""
RESTORE=false
DRY_RUN=false
BACKUP_DIR=".publish-backup"

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

show_help() {
    cat << 'EOF'
Prepare Armature Crates for Publishing

This script prepares the workspace for publishing to crates.io.

USAGE:
    ./scripts/prepare-publish.sh [OPTIONS]

OPTIONS:
    --version VERSION  Set the version for all crates
    --restore          Restore original Cargo.toml files from backup
    --dry-run          Show changes without applying them
    --help             Show this help message

WHAT IT DOES:
    1. Backs up all Cargo.toml files
    2. Converts path dependencies to version dependencies
    3. Ensures all required metadata is present

AFTER PUBLISHING:
    Run with --restore to revert to path dependencies for development.

EXAMPLES:
    # Prepare for publishing version 0.1.0
    ./scripts/prepare-publish.sh --version 0.1.0

    # Preview changes
    ./scripts/prepare-publish.sh --version 0.1.0 --dry-run

    # Restore after publishing
    ./scripts/prepare-publish.sh --restore

EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --version)
            VERSION="$2"
            shift 2
            ;;
        --restore)
            RESTORE=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --help|-h)
            show_help
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# Get workspace version if not specified
if [[ -z "$VERSION" && "$RESTORE" != "true" ]]; then
    VERSION=$(grep -A1 '\[workspace.package\]' Cargo.toml | grep 'version' | sed 's/.*"\(.*\)".*/\1/')
    if [[ -z "$VERSION" ]]; then
        log_error "Could not determine version. Use --version to specify."
        exit 1
    fi
    log_info "Using workspace version: $VERSION"
fi

# Get all workspace members
get_workspace_members() {
    grep -E '^\s+"armature-' Cargo.toml | sed 's/.*"\(armature-[^"]*\)".*/\1/' | sort -u
}

# =============================================================================
# Restore
# =============================================================================

restore_backups() {
    if [[ ! -d "$BACKUP_DIR" ]]; then
        log_error "No backup directory found at $BACKUP_DIR"
        exit 1
    fi

    log_info "Restoring Cargo.toml files from backup..."

    local members
    members=$(get_workspace_members)
    local restored=0

    for crate in $members; do
        local backup="$BACKUP_DIR/$crate/Cargo.toml"
        local target="$crate/Cargo.toml"

        if [[ -f "$backup" ]]; then
            if [[ "$DRY_RUN" == "true" ]]; then
                log_info "[DRY RUN] Would restore: $target"
            else
                cp "$backup" "$target"
                ((restored++))
            fi
        fi
    done

    # Restore root Cargo.toml if backed up
    if [[ -f "$BACKUP_DIR/Cargo.toml" ]]; then
        if [[ "$DRY_RUN" == "true" ]]; then
            log_info "[DRY RUN] Would restore: Cargo.toml"
        else
            cp "$BACKUP_DIR/Cargo.toml" "Cargo.toml"
            ((restored++))
        fi
    fi

    if [[ "$DRY_RUN" != "true" ]]; then
        rm -rf "$BACKUP_DIR"
        log_success "Restored $restored files"
    fi
}

# =============================================================================
# Prepare
# =============================================================================

# Backup a file
backup_file() {
    local file=$1
    local backup_path="$BACKUP_DIR/$(dirname "$file")"

    mkdir -p "$backup_path"
    cp "$file" "$BACKUP_DIR/$file"
}

# Convert path dependency to version dependency
convert_path_to_version() {
    local file=$1
    local version=$2

    # Pattern: armature-xxx = { path = "../armature-xxx" }
    # or: armature-xxx = { path = "../armature-xxx", ... }

    # Simple case: just path
    sed -i.tmp -E "s/(armature-[a-z-]+)\s*=\s*\{\s*path\s*=\s*\"[^\"]+\"\s*\}/\1 = { version = \"$version\" }/g" "$file"

    # With features: path = "...", features = [...]
    sed -i.tmp -E "s/(armature-[a-z-]+)\s*=\s*\{\s*path\s*=\s*\"[^\"]+\",\s*(features\s*=\s*\[[^\]]*\])\s*\}/\1 = { version = \"$version\", \2 }/g" "$file"

    # With optional: path = "...", optional = true
    sed -i.tmp -E "s/(armature-[a-z-]+)\s*=\s*\{\s*path\s*=\s*\"[^\"]+\",\s*(optional\s*=\s*true)\s*\}/\1 = { version = \"$version\", \2 }/g" "$file"

    # Reverse order: features first
    sed -i.tmp -E "s/(armature-[a-z-]+)\s*=\s*\{\s*(features\s*=\s*\[[^\]]*\]),\s*path\s*=\s*\"[^\"]+\"\s*\}/\1 = { version = \"$version\", \2 }/g" "$file"

    # Clean up temp files
    rm -f "$file.tmp"
}

# Ensure version is set correctly
set_version() {
    local file=$1
    local version=$2

    # If using workspace version
    if grep -q 'version.workspace\s*=\s*true' "$file"; then
        return
    fi

    # If version = "x.y.z" exists, update it
    if grep -q '^version\s*=' "$file"; then
        sed -i.tmp -E "s/^version\s*=\s*\"[^\"]+\"/version = \"$version\"/" "$file"
        rm -f "$file.tmp"
    fi
}

# Prepare a single crate
prepare_crate() {
    local crate=$1
    local version=$2
    local cargo_toml="$crate/Cargo.toml"

    if [[ ! -f "$cargo_toml" ]]; then
        log_warn "Skipping $crate: Cargo.toml not found"
        return
    fi

    log_info "Preparing $crate..."

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would convert path deps to version = \"$version\""
        # Show what would change
        grep -n 'path\s*=\s*"\.\./armature-' "$cargo_toml" 2>/dev/null || true
    else
        backup_file "$cargo_toml"
        convert_path_to_version "$cargo_toml" "$version"
        set_version "$cargo_toml" "$version"
    fi
}

# Prepare all crates
prepare_all() {
    log_info "Preparing workspace for publishing version $VERSION..."
    echo ""

    # Create backup directory
    if [[ "$DRY_RUN" != "true" ]]; then
        mkdir -p "$BACKUP_DIR"
        backup_file "Cargo.toml"
    fi

    local members
    members=$(get_workspace_members)
    local prepared=0

    for crate in $members; do
        prepare_crate "$crate" "$VERSION"
        ((prepared++))
    done

    echo ""
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "Dry run complete. $prepared crates would be prepared."
    else
        log_success "Prepared $prepared crates for publishing"
        echo ""
        echo "Next steps:"
        echo "  1. Review changes: git diff"
        echo "  2. Run: ./scripts/publish.sh --dry-run"
        echo "  3. Publish: ./scripts/publish.sh"
        echo "  4. Restore: ./scripts/prepare-publish.sh --restore"
    fi
}

# =============================================================================
# Main
# =============================================================================

if [[ "$RESTORE" == "true" ]]; then
    restore_backups
else
    prepare_all
fi

