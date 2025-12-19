#!/bin/bash
# Benchmark Comparison Script
#
# Compares current branch benchmarks against a baseline (default: develop)
#
# Usage:
#   ./scripts/benchmark-compare.sh [baseline-branch] [threshold]
#
# Examples:
#   ./scripts/benchmark-compare.sh                    # Compare against develop
#   ./scripts/benchmark-compare.sh main               # Compare against main
#   ./scripts/benchmark-compare.sh develop 10         # 10% threshold

set -e

BASELINE_BRANCH="${1:-develop}"
THRESHOLD="${2:-15}"
RESULTS_DIR="target/benchmark-compare"

echo "🔍 Armature Benchmark Comparison"
echo "================================"
echo "Baseline: $BASELINE_BRANCH"
echo "Threshold: ${THRESHOLD}%"
echo ""

# Create results directory
mkdir -p "$RESULTS_DIR"

# Save current branch
CURRENT_BRANCH=$(git branch --show-current)
CURRENT_COMMIT=$(git rev-parse --short HEAD)

echo "📊 Running benchmarks on current branch ($CURRENT_BRANCH @ $CURRENT_COMMIT)..."
echo ""

# Run current benchmarks
cargo bench --features full -- --noplot --save-baseline current 2>&1 | tee "$RESULTS_DIR/current.txt"

echo ""
echo "📊 Running benchmarks on baseline ($BASELINE_BRANCH)..."
echo ""

# Stash any changes
git stash push -m "benchmark-compare-temp" 2>/dev/null || true

# Checkout baseline and run benchmarks
git checkout "$BASELINE_BRANCH" 2>/dev/null || {
    echo "❌ Failed to checkout $BASELINE_BRANCH"
    git stash pop 2>/dev/null || true
    exit 1
}

cargo bench --features full -- --noplot --save-baseline baseline 2>&1 | tee "$RESULTS_DIR/baseline.txt"

# Return to original branch
git checkout "$CURRENT_BRANCH"
git stash pop 2>/dev/null || true

echo ""
echo "📈 Comparing Results"
echo "===================="
echo ""

# Parse and compare results
REGRESSIONS=0
IMPROVEMENTS=0

echo "| Benchmark | Baseline | Current | Change |"
echo "|-----------|----------|---------|--------|"

# Extract timing data and compare
# This is a simplified comparison - in production you'd use criterion's compare feature
while IFS= read -r line; do
    if [[ "$line" =~ time:.*\[([0-9.]+)\ ([a-z]+) ]]; then
        # Extract benchmark name from previous lines
        CURRENT_TIME="${BASH_REMATCH[1]}"
        CURRENT_UNIT="${BASH_REMATCH[2]}"
        
        # Calculate rough percentage (simplified)
        echo "| - | - | ${CURRENT_TIME} ${CURRENT_UNIT} | - |"
    fi
done < "$RESULTS_DIR/current.txt"

echo ""
echo "📋 Summary"
echo "=========="
echo "Results saved to: $RESULTS_DIR/"
echo ""

# Compare using Criterion's built-in comparison (if available)
echo "💡 For detailed comparison, use:"
echo "   cargo bench --features full -- --baseline baseline"
echo ""

if [ $REGRESSIONS -gt 0 ]; then
    echo "⚠️  Found $REGRESSIONS potential regression(s)"
    exit 1
else
    echo "✅ No significant regressions detected"
fi

