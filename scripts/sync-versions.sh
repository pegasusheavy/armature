#!/bin/bash
# Sync versions across all workspace crates
# Usage: ./scripts/sync-versions.sh [new_version]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the root directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

echo -e "${GREEN}Armature Version Sync Script${NC}"
echo "================================"
echo

# Get current version from root Cargo.toml
CURRENT_VERSION=$(grep '^version = ' "$ROOT_DIR/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
echo -e "Current version: ${YELLOW}$CURRENT_VERSION${NC}"

# Check if new version provided
if [ -n "$1" ]; then
    NEW_VERSION="$1"
    echo -e "New version: ${GREEN}$NEW_VERSION${NC}"
    echo
    
    # Validate version format (basic semver check)
    if ! [[ $NEW_VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9\.]+)?$ ]]; then
        echo -e "${RED}Error: Invalid version format. Use semver format (e.g., 0.1.0 or 1.0.0-beta.1)${NC}"
        exit 1
    fi
    
    # Confirm version bump
    read -p "Update all crates from $CURRENT_VERSION to $NEW_VERSION? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 0
    fi
    
    echo
    echo "Updating versions..."
    echo
    
    # Update workspace version
    sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" "$ROOT_DIR/Cargo.toml"
    echo "✓ Updated root Cargo.toml"
    
    # Update all workspace member versions
    for crate_dir in "$ROOT_DIR"/armature-*; do
        if [ -d "$crate_dir" ]; then
            crate_name=$(basename "$crate_dir")
            if [ -f "$crate_dir/Cargo.toml" ]; then
                # Skip if using workspace version (already updated via workspace)
                if grep -q '^version\.workspace = true' "$crate_dir/Cargo.toml"; then
                    echo "✓ $crate_name (inherits from workspace)"
                else
                    # Update version in crate Cargo.toml
                    sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" "$crate_dir/Cargo.toml"
                    echo "✓ Updated $crate_name/Cargo.toml"
                fi
            fi
        fi
    done
    
    echo
    echo -e "${GREEN}All versions updated to $NEW_VERSION${NC}"
    echo
    echo "Next steps:"
    echo "1. Review changes: git diff"
    echo "2. Test: cargo test --all"
    echo "3. Build: cargo build --all"
    echo "4. Commit: git commit -am \"chore: bump version to $NEW_VERSION\""
    echo "5. Tag: git tag v$NEW_VERSION"
    echo "6. Push: git push && git push --tags"
    
else
    # Just list current versions
    echo "Workspace member versions:"
    echo
    
    for crate_dir in "$ROOT_DIR"/armature-*; do
        if [ -d "$crate_dir" ]; then
            crate_name=$(basename "$crate_dir")
            if [ -f "$crate_dir/Cargo.toml" ]; then
                # Check if using workspace version
                if grep -q '^version\.workspace = true' "$crate_dir/Cargo.toml"; then
                    echo -e "  ${GREEN}✓${NC} $crate_name: $CURRENT_VERSION (workspace)"
                else
                    version=$(grep '^version = ' "$crate_dir/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
                    if [ "$version" = "$CURRENT_VERSION" ]; then
                        echo -e "  ${GREEN}✓${NC} $crate_name: $version"
                    else
                        echo -e "  ${RED}✗${NC} $crate_name: $version (out of sync!)"
                    fi
                fi
            fi
        fi
    done
    
    echo
    echo "Usage: $0 [new_version]"
    echo "Example: $0 0.2.0"
fi

