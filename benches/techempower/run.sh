#!/bin/bash
# TechEmpower Benchmark Runner
# Usage: ./benches/techempower/run.sh [host:port]

set -e

HOST="${1:-127.0.0.1:8080}"
DURATION="${DURATION:-15s}"
THREADS="${THREADS:-4}"
CONNECTIONS="${CONNECTIONS:-256}"

echo "╔════════════════════════════════════════════════════════════╗"
echo "║           TechEmpower Benchmark Runner                     ║"
echo "╠════════════════════════════════════════════════════════════╣"
echo "║  Target: $HOST"
echo "║  Duration: $DURATION"
echo "║  Threads: $THREADS"
echo "║  Connections: $CONNECTIONS"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Check if wrk is installed
if ! command -v wrk &> /dev/null; then
    echo "❌ wrk not found. Install with:"
    echo "   Ubuntu: sudo apt-get install wrk"
    echo "   macOS: brew install wrk"
    exit 1
fi

# Check if server is running
if ! curl -s "http://$HOST/json" > /dev/null 2>&1; then
    echo "❌ Server not responding at http://$HOST"
    echo "   Start the server with: cargo run --release --example techempower_server"
    exit 1
fi

echo "✅ Server is responding"
echo ""

# Results file
RESULTS_FILE="benchmark_results_$(date +%Y%m%d_%H%M%S).txt"
echo "TechEmpower Benchmark Results - $(date)" > "$RESULTS_FILE"
echo "Host: $HOST" >> "$RESULTS_FILE"
echo "Duration: $DURATION, Threads: $THREADS, Connections: $CONNECTIONS" >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"

run_benchmark() {
    local name="$1"
    local endpoint="$2"
    local extra_args="${3:-}"
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📊 $name"
    echo "   Endpoint: $endpoint"
    echo ""
    
    echo "=== $name ===" >> "$RESULTS_FILE"
    wrk -t"$THREADS" -c"$CONNECTIONS" -d"$DURATION" $extra_args "http://$HOST$endpoint" | tee -a "$RESULTS_FILE"
    echo "" >> "$RESULTS_FILE"
    echo ""
    
    # Brief pause between tests
    sleep 2
}

# Run all benchmarks
echo ""
echo "🏁 Starting benchmarks..."
echo ""

run_benchmark "JSON Serialization" "/json"
run_benchmark "Plaintext" "/plaintext" "-H 'Accept: text/plain'"
run_benchmark "Single DB Query" "/db"
run_benchmark "Multiple Queries (20)" "/queries?queries=20"
run_benchmark "Fortunes" "/fortunes"
run_benchmark "Updates (20)" "/updates?queries=20"
run_benchmark "Cached Queries (20)" "/cached-queries?queries=20"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Benchmark complete!"
echo "📄 Results saved to: $RESULTS_FILE"
echo ""

# Summary
echo "📈 Quick Summary:"
echo ""
grep "Requests/sec" "$RESULTS_FILE" | while read line; do
    echo "   $line"
done

