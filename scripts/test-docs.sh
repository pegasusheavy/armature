#!/bin/bash
# scripts/test-docs.sh
# Run documentation tests across all workspace members

set -e

echo "📚 Armature Documentation Test Runner"
echo "======================================"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counters
TOTAL=0
PASSED=0
FAILED=0

# List of all workspace members
MEMBERS=(
    "armature-core"
    "armature-macro"
    "armature-config"
    "armature-graphql"
    "armature-angular"
    "armature-react"
    "armature-vue"
    "armature-svelte"
    "armature-jwt"
    "armature-auth"
    "armature-testing"
    "armature-validation"
    "armature-openapi"
    "armature-cache"
    "armature-cron"
    "armature-queue"
    "armature-opentelemetry"
    "armature-security"
    "armature-acme"
    "armature-csrf"
    "armature-xss"
    "armature-handlebars"
)

# Function to test a single member
test_member() {
    local member=$1
    TOTAL=$((TOTAL + 1))

    echo -n "Testing $member... "

    if cargo test --doc -p "$member" --quiet 2>&1 | grep -q "test result: ok"; then
        echo -e "${GREEN}✓ PASSED${NC}"
        PASSED=$((PASSED + 1))
        return 0
    else
        echo -e "${RED}✗ FAILED${NC}"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

# Test each member
for member in "${MEMBERS[@]}"; do
    test_member "$member" || true
done

echo ""
echo "======================================"
echo "Summary:"
echo "  Total:  $TOTAL"
echo -e "  ${GREEN}Passed: $PASSED${NC}"
if [ $FAILED -gt 0 ]; then
    echo -e "  ${RED}Failed: $FAILED${NC}"
fi
echo ""

# Exit with error if any tests failed
if [ $FAILED -gt 0 ]; then
    echo -e "${RED}❌ Some documentation tests failed${NC}"
    exit 1
else
    echo -e "${GREEN}✅ All documentation tests passed!${NC}"
    exit 0
fi


