#!/bin/bash
# =============================================================================
# Armature Workspace Publisher
# =============================================================================
#
# Publishes all workspace crates to crates.io in the correct dependency order.
#
# Usage:
#   ./scripts/publish.sh [OPTIONS]
#
# Options:
#   --dry-run       Show publish order without actually publishing
#   --check         Verify all crates are ready to publish
#   --single CRATE  Publish only the specified crate
#   --from CRATE    Publish starting from the specified crate
#   --skip CRATE    Skip the specified crate (can be used multiple times)
#   --no-verify     Skip cargo publish verification step
#   --help          Show this help message
#
# Environment:
#   CARGO_REGISTRY_TOKEN  Required for publishing (or use `cargo login`)
#
# =============================================================================

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
DRY_RUN=false
CHECK_ONLY=false
SINGLE_CRATE=""
FROM_CRATE=""
NO_VERIFY=false
SKIP_CRATES=()
PUBLISH_DELAY=30  # Delay between publishes to allow crates.io to index

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
Armature Workspace Publisher

Publishes all workspace crates to crates.io in the correct dependency order.

USAGE:
    ./scripts/publish.sh [OPTIONS]

OPTIONS:
    --dry-run       Show publish order without actually publishing
    --check         Verify all crates are ready to publish
    --single CRATE  Publish only the specified crate
    --from CRATE    Start publishing from the specified crate
    --skip CRATE    Skip the specified crate (can be repeated)
    --no-verify     Skip cargo publish verification step
    --help          Show this help message

ENVIRONMENT:
    CARGO_REGISTRY_TOKEN  API token for crates.io (or use `cargo login`)

EXAMPLES:
    # See publish order
    ./scripts/publish.sh --dry-run

    # Check all crates are ready
    ./scripts/publish.sh --check

    # Publish everything
    ./scripts/publish.sh

    # Publish single crate
    ./scripts/publish.sh --single armature-log

    # Resume from a specific crate
    ./scripts/publish.sh --from armature-auth

    # Skip problematic crates
    ./scripts/publish.sh --skip armature-cli --skip armature-ferron

DEPENDENCY ORDER:
    The script automatically determines the correct publish order by
    analyzing inter-workspace dependencies. Crates with no workspace
    dependencies are published first.

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --check)
            CHECK_ONLY=true
            shift
            ;;
        --single)
            SINGLE_CRATE="$2"
            shift 2
            ;;
        --from)
            FROM_CRATE="$2"
            shift 2
            ;;
        --skip)
            SKIP_CRATES+=("$2")
            shift 2
            ;;
        --no-verify)
            NO_VERIFY=true
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

# =============================================================================
# Dependency Analysis
# =============================================================================

# Get all workspace members
get_workspace_members() {
    grep -E '^\s+"armature-' Cargo.toml | sed 's/.*"\(armature-[^"]*\)".*/\1/' | sort -u
}

# Get workspace dependencies for a crate
get_workspace_deps() {
    local crate=$1
    local cargo_toml="$crate/Cargo.toml"

    if [[ ! -f "$cargo_toml" ]]; then
        return
    fi

    # Extract path dependencies that are workspace members
    grep -E 'path\s*=\s*"\.\./armature-' "$cargo_toml" 2>/dev/null | \
        sed 's/.*path\s*=\s*"\.\.\/\(armature-[^"]*\)".*/\1/' | \
        sort -u
}

# Build dependency graph and compute publish order using topological sort
compute_publish_order() {
    local members
    members=$(get_workspace_members)

    declare -A in_degree
    declare -A deps
    declare -a order

    # Initialize
    for crate in $members; do
        in_degree[$crate]=0
        deps[$crate]=""
    done

    # Build dependency graph
    for crate in $members; do
        local crate_deps
        crate_deps=$(get_workspace_deps "$crate")
        deps[$crate]="$crate_deps"

        for dep in $crate_deps; do
            if [[ -n "${in_degree[$dep]+x}" ]]; then
                ((in_degree[$crate]++))
            fi
        done
    done

    # Kahn's algorithm for topological sort
    local queue=()

    # Find all crates with no dependencies
    for crate in $members; do
        if [[ ${in_degree[$crate]} -eq 0 ]]; then
            queue+=("$crate")
        fi
    done

    while [[ ${#queue[@]} -gt 0 ]]; do
        # Sort queue for deterministic order
        IFS=$'\n' sorted_queue=($(sort <<<"${queue[*]}")); unset IFS
        local current="${sorted_queue[0]}"
        queue=("${sorted_queue[@]:1}")

        order+=("$current")

        # For each crate that depends on current
        for crate in $members; do
            if [[ "${deps[$crate]}" == *"$current"* ]]; then
                ((in_degree[$crate]--))
                if [[ ${in_degree[$crate]} -eq 0 ]]; then
                    queue+=("$crate")
                fi
            fi
        done
    done

    # Check for cycles
    local total_members
    total_members=$(echo "$members" | wc -w)
    if [[ ${#order[@]} -ne $total_members ]]; then
        log_error "Circular dependency detected!"
        exit 1
    fi

    echo "${order[@]}"
}

# =============================================================================
# Verification
# =============================================================================

# Check if a crate is ready to publish
check_crate() {
    local crate=$1
    local cargo_toml="$crate/Cargo.toml"
    local errors=()

    # Check Cargo.toml exists
    if [[ ! -f "$cargo_toml" ]]; then
        errors+=("Cargo.toml not found")
    else
        # Check for required fields
        if ! grep -q '^name\s*=' "$cargo_toml" && ! grep -q 'name.workspace' "$cargo_toml"; then
            errors+=("Missing 'name' field")
        fi

        if ! grep -q 'version' "$cargo_toml"; then
            errors+=("Missing 'version' field")
        fi

        if ! grep -q 'license' "$cargo_toml"; then
            errors+=("Missing 'license' field")
        fi

        if ! grep -q 'description' "$cargo_toml"; then
            errors+=("Missing 'description' field")
        fi

        # Check for path dependencies (warning - will need conversion)
        local path_deps
        path_deps=$(grep -cE 'path\s*=\s*"\.\./armature-' "$cargo_toml" 2>/dev/null) || path_deps=0
        if [[ $path_deps -gt 0 ]]; then
            errors+=("$path_deps path deps (run prepare-publish.sh)")
        fi
    fi

    # Check src/lib.rs or src/main.rs exists
    if [[ ! -f "$crate/src/lib.rs" && ! -f "$crate/src/main.rs" ]]; then
        errors+=("Missing src/lib.rs or src/main.rs")
    fi

    if [[ ${#errors[@]} -gt 0 ]]; then
        echo "WARN: ${errors[*]}"
        return 0  # Don't fail on warnings
    else
        echo "OK"
        return 0
    fi
}

# Check all crates
check_all_crates() {
    log_info "Checking all crates for publish readiness..."
    echo ""

    local publish_order
    publish_order=$(compute_publish_order)

    local failed=0
    local passed=0
    local warned=0

    for crate in $publish_order; do
        local result
        result=$(check_crate "$crate")

        if [[ "$result" == "OK" ]]; then
            echo -e "  ${GREEN}✓${NC} $crate"
            passed=$((passed + 1))
        elif [[ "$result" == WARN:* ]]; then
            echo -e "  ${YELLOW}!${NC} $crate: ${result#WARN: }"
            warned=$((warned + 1))
        else
            echo -e "  ${RED}✗${NC} $crate: ${result#FAIL: }"
            failed=$((failed + 1))
        fi
    done

    echo ""
    echo "Results: $passed ready, $warned warnings, $failed failed"

    if [[ $failed -gt 0 ]]; then
        return 1
    fi
    return 0
}

# =============================================================================
# Publishing
# =============================================================================

# Publish a single crate
publish_crate() {
    local crate=$1

    log_info "Publishing $crate..."

    cd "$PROJECT_ROOT/$crate"

    local publish_args=()
    if [[ "$NO_VERIFY" == "true" ]]; then
        publish_args+=("--no-verify")
    fi

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would publish: cargo publish ${publish_args[*]}"
    else
        cargo publish "${publish_args[@]}"
        log_success "$crate published successfully"
    fi

    cd "$PROJECT_ROOT"
}

# Check if crate should be skipped
should_skip() {
    local crate=$1

    for skip in "${SKIP_CRATES[@]}"; do
        if [[ "$crate" == "$skip" ]]; then
            return 0
        fi
    done
    return 1
}

# Main publish function
publish_all() {
    log_info "Computing publish order..."

    local publish_order
    publish_order=$(compute_publish_order)

    echo ""
    log_info "Publish order:"
    local i=1
    for crate in $publish_order; do
        local status=""
        if should_skip "$crate"; then
            status=" ${YELLOW}(skip)${NC}"
        fi
        echo -e "  $i. $crate$status"
        ((i++))
    done
    echo ""

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "Dry run complete. No packages were published."
        return 0
    fi

    if [[ "$CHECK_ONLY" == "true" ]]; then
        check_all_crates
        return $?
    fi

    # Confirm before publishing
    if [[ -z "$SINGLE_CRATE" ]]; then
        echo -e "${YELLOW}This will publish ${#publish_order[@]} crates to crates.io.${NC}"
        echo -n "Continue? [y/N] "
        read -r confirm
        if [[ "$confirm" != "y" && "$confirm" != "Y" ]]; then
            log_info "Aborted."
            return 0
        fi
    fi

    # Publish crates
    local started=false
    local published=0
    local skipped=0

    for crate in $publish_order; do
        # Handle --from flag
        if [[ -n "$FROM_CRATE" && "$started" == "false" ]]; then
            if [[ "$crate" == "$FROM_CRATE" ]]; then
                started=true
            else
                log_info "Skipping $crate (before --from)"
                ((skipped++))
                continue
            fi
        fi

        # Handle --single flag
        if [[ -n "$SINGLE_CRATE" && "$crate" != "$SINGLE_CRATE" ]]; then
            continue
        fi

        # Handle --skip flag
        if should_skip "$crate"; then
            log_warn "Skipping $crate (--skip)"
            ((skipped++))
            continue
        fi

        publish_crate "$crate"
        ((published++))

        # Exit if single crate mode
        if [[ -n "$SINGLE_CRATE" ]]; then
            break
        fi

        # Delay between publishes to allow crates.io to index
        if [[ "$DRY_RUN" != "true" && $published -lt ${#publish_order[@]} ]]; then
            log_info "Waiting ${PUBLISH_DELAY}s for crates.io to index..."
            sleep $PUBLISH_DELAY
        fi
    done

    echo ""
    log_success "Publishing complete!"
    echo "  Published: $published"
    echo "  Skipped: $skipped"
}

# =============================================================================
# Main
# =============================================================================

if [[ "$CHECK_ONLY" == "true" ]]; then
    check_all_crates
else
    publish_all
fi

