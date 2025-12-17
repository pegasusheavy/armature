#!/bin/bash
# Framework Comparison Benchmark
# Compares Armature against Axum and Actix-web
# Usage: ./benches/techempower/compare.sh

set -e

DURATION="${DURATION:-15s}"
THREADS="${THREADS:-4}"
CONNECTIONS="${CONNECTIONS:-256}"

echo "╔════════════════════════════════════════════════════════════╗"
echo "║         Framework Comparison Benchmark                     ║"
echo "╠════════════════════════════════════════════════════════════╣"
echo "║  Frameworks: Armature, Axum, Actix-web                     ║"
echo "║  Tests: JSON, Plaintext                                    ║"
echo "║  Duration: $DURATION per test                              ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Check if wrk is installed
if ! command -v wrk &> /dev/null; then
    echo "❌ wrk not found. Install with:"
    echo "   Ubuntu: sudo apt-get install wrk"
    echo "   macOS: brew install wrk"
    exit 1
fi

# Ports for each framework
ARMATURE_PORT=8080
AXUM_PORT=8081
ACTIX_PORT=8082

# Results
RESULTS_FILE="comparison_$(date +%Y%m%d_%H%M%S).md"

cat > "$RESULTS_FILE" << EOF
# Framework Comparison Results

**Date:** $(date)
**Duration:** $DURATION per test
**Threads:** $THREADS
**Connections:** $CONNECTIONS

## JSON Serialization

| Framework | Requests/sec | Latency (avg) | Latency (max) | Transfer/sec |
|-----------|-------------|---------------|---------------|--------------|
EOF

echo ""
echo "🔨 Building release binaries..."
echo ""

# Build Armature
echo "Building Armature..."
cargo build --release --example techempower_server 2>/dev/null

# Build comparison servers if they exist
if [ -d "benches/comparison_servers/axum_server" ]; then
    echo "Building Axum server..."
    (cd benches/comparison_servers/axum_server && cargo build --release 2>/dev/null) || true
fi

if [ -d "benches/comparison_servers/actix_server" ]; then
    echo "Building Actix server..."
    (cd benches/comparison_servers/actix_server && cargo build --release 2>/dev/null) || true
fi

echo ""

# Function to extract metrics from wrk output
extract_metrics() {
    local output="$1"
    local rps=$(echo "$output" | grep "Requests/sec" | awk '{print $2}')
    local lat_avg=$(echo "$output" | grep -E "^\s+Latency" | awk '{print $2}')
    local lat_max=$(echo "$output" | grep -E "^\s+Latency" | awk '{print $4}')
    local transfer=$(echo "$output" | grep "Transfer/sec" | awk '{print $2}')
    echo "$rps|$lat_avg|$lat_max|$transfer"
}

# Function to run benchmark
run_test() {
    local name="$1"
    local port="$2"
    local endpoint="$3"
    
    if curl -s "http://127.0.0.1:$port$endpoint" > /dev/null 2>&1; then
        local output=$(wrk -t"$THREADS" -c"$CONNECTIONS" -d"$DURATION" "http://127.0.0.1:$port$endpoint" 2>&1)
        local metrics=$(extract_metrics "$output")
        local rps=$(echo "$metrics" | cut -d'|' -f1)
        local lat_avg=$(echo "$metrics" | cut -d'|' -f2)
        local lat_max=$(echo "$metrics" | cut -d'|' -f3)
        local transfer=$(echo "$metrics" | cut -d'|' -f4)
        echo "| $name | $rps | $lat_avg | $lat_max | $transfer |"
    else
        echo "| $name | - | - | - | (not running) |"
    fi
}

echo "🚀 Starting benchmark servers..."
echo ""

# Start Armature
echo "Starting Armature on port $ARMATURE_PORT..."
./target/release/examples/techempower_server &
ARMATURE_PID=$!
sleep 2

# Start Axum if available
if [ -f "benches/comparison_servers/axum_server/target/release/axum_server" ]; then
    echo "Starting Axum on port $AXUM_PORT..."
    ./benches/comparison_servers/axum_server/target/release/axum_server &
    AXUM_PID=$!
    sleep 2
fi

# Start Actix if available
if [ -f "benches/comparison_servers/actix_server/target/release/actix_server" ]; then
    echo "Starting Actix on port $ACTIX_PORT..."
    ./benches/comparison_servers/actix_server/target/release/actix_server &
    ACTIX_PID=$!
    sleep 2
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Running JSON Serialization Benchmark"
echo ""

# JSON tests
run_test "Armature" "$ARMATURE_PORT" "/json" >> "$RESULTS_FILE"
run_test "Axum" "$AXUM_PORT" "/json" >> "$RESULTS_FILE"
run_test "Actix-web" "$ACTIX_PORT" "/json" >> "$RESULTS_FILE"

cat >> "$RESULTS_FILE" << EOF

## Plaintext

| Framework | Requests/sec | Latency (avg) | Latency (max) | Transfer/sec |
|-----------|-------------|---------------|---------------|--------------|
EOF

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Running Plaintext Benchmark"
echo ""

# Plaintext tests
run_test "Armature" "$ARMATURE_PORT" "/plaintext" >> "$RESULTS_FILE"
run_test "Axum" "$AXUM_PORT" "/plaintext" >> "$RESULTS_FILE"
run_test "Actix-web" "$ACTIX_PORT" "/plaintext" >> "$RESULTS_FILE"

# Cleanup
echo ""
echo "🧹 Stopping servers..."
kill $ARMATURE_PID 2>/dev/null || true
[ -n "$AXUM_PID" ] && kill $AXUM_PID 2>/dev/null || true
[ -n "$ACTIX_PID" ] && kill $ACTIX_PID 2>/dev/null || true

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Comparison complete!"
echo "📄 Results saved to: $RESULTS_FILE"
echo ""
cat "$RESULTS_FILE"

