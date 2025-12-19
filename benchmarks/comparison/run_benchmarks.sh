#!/bin/bash
# Framework Comparison Benchmarks
# Usage: ./run_benchmarks.sh [connections] [duration]

set -e

CONNECTIONS=${1:-100}
DURATION=${2:-10s}
RESULTS_DIR="./results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_FILE="$RESULTS_DIR/benchmark_$TIMESTAMP.md"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

mkdir -p "$RESULTS_DIR"

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}           Framework Comparison Benchmarks${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "Connections: ${YELLOW}$CONNECTIONS${NC}"
echo -e "Duration: ${YELLOW}$DURATION${NC}"
echo ""

# Function to wait for server to be ready
wait_for_server() {
    local port=$1
    local max_attempts=30
    local attempt=0

    while ! curl -s "http://localhost:$port/json" > /dev/null 2>&1; do
        attempt=$((attempt + 1))
        if [ $attempt -ge $max_attempts ]; then
            echo -e "${RED}Server on port $port failed to start${NC}"
            return 1
        fi
        sleep 0.5
    done
    echo -e "${GREEN}Server on port $port is ready${NC}"
}

# Function to run benchmark and capture results
run_benchmark() {
    local name=$1
    local port=$2
    local endpoint=$3

    echo -e "\n${YELLOW}Benchmarking $name - $endpoint${NC}"

    # Run oha and capture output
    oha -c $CONNECTIONS -z $DURATION --no-tui "http://localhost:$port$endpoint" 2>&1
}

# Initialize results file
cat > "$RESULTS_FILE" << EOF
# Framework Comparison Benchmarks

**Date:** $(date '+%Y-%m-%d %H:%M:%S')
**System:** $(uname -s) $(uname -r)
**CPU:** $(grep -m1 'model name' /proc/cpuinfo 2>/dev/null | cut -d: -f2 | xargs || echo "Unknown")
**Connections:** $CONNECTIONS
**Duration:** $DURATION

---

EOF

# ============================================
# Build Rust servers
# ============================================
echo -e "${BLUE}Building Rust benchmark servers...${NC}"
cargo build --release --bin armature_bench --bin actix_bench --bin axum_bench 2>&1 | tail -5

# ============================================
# ARMATURE BENCHMARKS
# ============================================
echo -e "\n${GREEN}════════════════════════════════════════${NC}"
echo -e "${GREEN}  Starting Armature Benchmark${NC}"
echo -e "${GREEN}════════════════════════════════════════${NC}"

PORT=8080 ./target/release/armature_bench &
ARMATURE_PID=$!
wait_for_server 8080

echo "## Armature" >> "$RESULTS_FILE"
echo '```' >> "$RESULTS_FILE"
run_benchmark "Armature" 8080 "/json" | tee -a "$RESULTS_FILE"
echo '```' >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"

kill $ARMATURE_PID 2>/dev/null || true
sleep 1

# ============================================
# ACTIX-WEB BENCHMARKS
# ============================================
echo -e "\n${GREEN}════════════════════════════════════════${NC}"
echo -e "${GREEN}  Starting Actix-web Benchmark${NC}"
echo -e "${GREEN}════════════════════════════════════════${NC}"

PORT=8082 ./target/release/actix_bench &
ACTIX_PID=$!
wait_for_server 8082

echo "## Actix-web" >> "$RESULTS_FILE"
echo '```' >> "$RESULTS_FILE"
run_benchmark "Actix-web" 8082 "/json" | tee -a "$RESULTS_FILE"
echo '```' >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"

kill $ACTIX_PID 2>/dev/null || true
sleep 1

# ============================================
# AXUM BENCHMARKS
# ============================================
echo -e "\n${GREEN}════════════════════════════════════════${NC}"
echo -e "${GREEN}  Starting Axum Benchmark${NC}"
echo -e "${GREEN}════════════════════════════════════════${NC}"

PORT=8083 ./target/release/axum_bench &
AXUM_PID=$!
wait_for_server 8083

echo "## Axum" >> "$RESULTS_FILE"
echo '```' >> "$RESULTS_FILE"
run_benchmark "Axum" 8083 "/json" | tee -a "$RESULTS_FILE"
echo '```' >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"

kill $AXUM_PID 2>/dev/null || true
sleep 1

# ============================================
# EXPRESS BENCHMARKS (if Node.js available)
# ============================================
if command -v node &> /dev/null; then
    echo -e "\n${GREEN}════════════════════════════════════════${NC}"
    echo -e "${GREEN}  Starting Express.js Benchmark${NC}"
    echo -e "${GREEN}════════════════════════════════════════${NC}"

    # Check if express is installed
    if [ ! -d "node_modules" ]; then
        echo "Installing Express..."
        npm init -y > /dev/null 2>&1
        npm install express --save > /dev/null 2>&1
    fi

    PORT=8081 node express_bench.js &
    EXPRESS_PID=$!
    wait_for_server 8081

    echo "## Express.js (Node.js)" >> "$RESULTS_FILE"
    echo '```' >> "$RESULTS_FILE"
    run_benchmark "Express" 8081 "/json" | tee -a "$RESULTS_FILE"
    echo '```' >> "$RESULTS_FILE"
    echo "" >> "$RESULTS_FILE"

    kill $EXPRESS_PID 2>/dev/null || true
else
    echo -e "${YELLOW}Node.js not found, skipping Express benchmark${NC}"
fi

# ============================================
# Summary
# ============================================
echo -e "\n${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}                    Benchmarks Complete${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "Results saved to: ${GREEN}$RESULTS_FILE${NC}"

# Clean up any remaining processes
pkill -f "armature_bench" 2>/dev/null || true
pkill -f "actix_bench" 2>/dev/null || true
pkill -f "axum_bench" 2>/dev/null || true
pkill -f "express_bench" 2>/dev/null || true

echo -e "${GREEN}Done!${NC}"

