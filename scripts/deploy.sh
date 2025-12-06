#!/bin/bash
# Comprehensive deployment script for Armature
# Syncs versions, runs tests, builds, and prepares for release

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Get directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

echo -e "${BLUE}╔════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Armature Deployment Script      ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════╝${NC}"
echo

# Check if version provided
if [ -z "$1" ]; then
    echo -e "${RED}Error: Version number required${NC}"
    echo "Usage: $0 <version> [--skip-tests]"
    echo "Example: $0 0.2.0"
    exit 1
fi

NEW_VERSION="$1"
SKIP_TESTS=false

if [ "$2" = "--skip-tests" ]; then
    SKIP_TESTS=true
fi

# Validate version format
if ! [[ $NEW_VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9\.]+)?$ ]]; then
    echo -e "${RED}Error: Invalid version format${NC}"
    echo "Use semver format: MAJOR.MINOR.PATCH (e.g., 0.1.0 or 1.0.0-beta.1)"
    exit 1
fi

# Get current version
CURRENT_VERSION=$(grep '^version = ' "$ROOT_DIR/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')

echo -e "Current version: ${YELLOW}$CURRENT_VERSION${NC}"
echo -e "New version:     ${GREEN}$NEW_VERSION${NC}"
echo

# Confirm
read -p "Proceed with deployment? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 0
fi

echo

# Step 1: Check git status
echo -e "${BLUE}[1/7] Checking git status...${NC}"
if ! git diff-index --quiet HEAD --; then
    echo -e "${RED}Error: Working directory has uncommitted changes${NC}"
    echo "Please commit or stash changes before deploying"
    exit 1
fi
echo -e "${GREEN}✓ Working directory clean${NC}"
echo

# Step 2: Sync versions
echo -e "${BLUE}[2/7] Syncing versions across workspace...${NC}"
"$SCRIPT_DIR/sync-versions.sh" "$NEW_VERSION" <<< "y" > /dev/null
echo -e "${GREEN}✓ All versions synced to $NEW_VERSION${NC}"
echo

# Step 3: Format code
echo -e "${BLUE}[3/7] Formatting code...${NC}"
cd "$ROOT_DIR"
cargo fmt --all
echo -e "${GREEN}✓ Code formatted${NC}"
echo

# Step 4: Run clippy
echo -e "${BLUE}[4/7] Running clippy...${NC}"
if cargo clippy --all --all-features -- -D warnings; then
    echo -e "${GREEN}✓ Clippy passed${NC}"
else
    echo -e "${RED}✗ Clippy found issues${NC}"
    exit 1
fi
echo

# Step 5: Run tests
if [ "$SKIP_TESTS" = false ]; then
    echo -e "${BLUE}[5/7] Running tests...${NC}"
    if cargo test --all --all-features; then
        echo -e "${GREEN}✓ All tests passed${NC}"
    else
        echo -e "${RED}✗ Tests failed${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}[5/7] Skipping tests (--skip-tests flag)${NC}"
fi
echo

# Step 6: Build release
echo -e "${BLUE}[6/7] Building release...${NC}"
if cargo build --release --all-features; then
    echo -e "${GREEN}✓ Release build successful${NC}"
else
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi
echo

# Step 7: Create git commit and tag
echo -e "${BLUE}[7/7] Creating git commit and tag...${NC}"
git add -A
git commit -m "chore: bump version to $NEW_VERSION"
git tag -a "v$NEW_VERSION" -m "Release version $NEW_VERSION"
echo -e "${GREEN}✓ Commit and tag created${NC}"
echo

# Summary
echo -e "${GREEN}╔════════════════════════════════════╗${NC}"
echo -e "${GREEN}║     Deployment Successful! 🎉      ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════╝${NC}"
echo
echo "Version: v$NEW_VERSION"
echo
echo "Next steps:"
echo -e "  1. Review changes: ${YELLOW}git show${NC}"
echo -e "  2. Push to remote:  ${YELLOW}git push origin develop && git push origin v$NEW_VERSION${NC}"
echo -e "  3. Create release:  ${YELLOW}gh release create v$NEW_VERSION --generate-notes${NC}"
echo -e "  4. Publish crates:  ${YELLOW}cargo publish --all${NC}"
echo
echo "Or to undo:"
echo -e "  ${YELLOW}git reset --hard HEAD~1 && git tag -d v$NEW_VERSION${NC}"


