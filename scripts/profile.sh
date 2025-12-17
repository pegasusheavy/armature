#!/bin/bash
# Profile the Armature framework
# Usage: ./scripts/profile.sh [duration_seconds]

set -e

DURATION=${1:-30}
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

echo "🔬 Armature Profiling Script"
echo "============================"
echo ""

# Check for required tools
if ! command -v curl &> /dev/null; then
    echo "❌ curl is required but not installed."
    exit 1
fi

# Build in release mode with debug symbols
echo "📦 Building with profiling enabled..."
cargo build --example profiling_server --release 2>&1 | grep -v "warning:"

# Start the server in background
echo ""
echo "🚀 Starting profiling server..."
cargo run --example profiling_server --release 2>&1 | grep -v "warning:" &
SERVER_PID=$!

# Wait for server to start
sleep 2

# Find the port from the server output
PORT=$(grep -o "localhost:[0-9]*" /home/joseph/.cursor/projects/home-joseph-PegasusHeavyIndustries-armature/terminals/*.txt 2>/dev/null | tail -1 | cut -d: -f2)
if [ -z "$PORT" ]; then
    PORT=3000
fi

echo "📡 Server running on port $PORT"
echo ""
echo "⏱️  Generating load for ${DURATION} seconds..."
echo ""

# Generate load
START_TIME=$(date +%s)
REQUEST_COUNT=0

while [ $(($(date +%s) - START_TIME)) -lt $DURATION ]; do
    curl -s "http://localhost:$PORT/tasks" > /dev/null &
    curl -s "http://localhost:$PORT/tasks/1" > /dev/null &
    curl -s "http://localhost:$PORT/compute/light" > /dev/null &
    curl -s "http://localhost:$PORT/compute/heavy/500" > /dev/null &
    wait
    REQUEST_COUNT=$((REQUEST_COUNT + 4))

    if [ $((REQUEST_COUNT % 100)) -eq 0 ]; then
        ELAPSED=$(($(date +%s) - START_TIME))
        echo "   Requests: $REQUEST_COUNT, Elapsed: ${ELAPSED}s"
    fi
done

echo ""
echo "✅ Load generation complete: $REQUEST_COUNT requests"
echo ""

# Stop the server gracefully
echo "🛑 Stopping server..."
kill -INT $SERVER_PID 2>/dev/null || true
wait $SERVER_PID 2>/dev/null || true

sleep 1

# Check for output files
if [ -f "flamegraph-profile.svg" ]; then
    echo ""
    echo "📊 Results:"
    echo "   Flamegraph: $(pwd)/flamegraph-profile.svg"
    echo ""
    echo "Open the SVG file in a browser to explore the CPU profile."
    echo "Wider bars = more CPU time spent in that function."
else
    echo "⚠️  Flamegraph not generated. The server may have exited unexpectedly."
fi

echo ""
echo "🎉 Profiling complete!"

