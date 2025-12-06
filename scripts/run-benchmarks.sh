#!/bin/bash
# scripts/run-benchmarks.sh
# Comprehensive benchmark runner for Armature

# Exit immediately if a command exits with a non-zero status
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to display usage
usage() {
    echo -e "${CYAN}Armature Benchmark Runner${NC}"
    echo "========================="
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -a, --all              Run all benchmarks"
    echo "  -c, --core             Run core benchmarks only"
    echo "  -s, --security         Run security benchmarks only"
    echo "  -v, --validation       Run validation benchmarks only"
    echo "  -d, --data             Run data benchmarks only"
    echo "  -b, --baseline NAME    Save results as baseline"
    echo "  -p, --compare NAME     Compare with baseline"
    echo "  -o, --open             Open HTML report after running"
    echo "  -q, --quick            Quick run (fewer samples)"
    echo "  -h, --help             Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 --all               # Run all benchmarks"
    echo "  $0 --core --open       # Run core benchmarks and open report"
    echo "  $0 --baseline main     # Save as baseline 'main'"
    echo "  $0 --compare main      # Compare with baseline 'main'"
    echo "  $0 -a -b v0.1.0        # Run all and save as v0.1.0"
    exit 1
}

# Default values
RUN_ALL=false
RUN_CORE=false
RUN_SECURITY=false
RUN_VALIDATION=false
RUN_DATA=false
BASELINE=""
COMPARE=""
OPEN_REPORT=false
QUICK=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -a|--all)
            RUN_ALL=true
            shift
            ;;
        -c|--core)
            RUN_CORE=true
            shift
            ;;
        -s|--security)
            RUN_SECURITY=true
            shift
            ;;
        -v|--validation)
            RUN_VALIDATION=true
            shift
            ;;
        -d|--data)
            RUN_DATA=true
            shift
            ;;
        -b|--baseline)
            BASELINE="$2"
            shift 2
            ;;
        -p|--compare)
            COMPARE="$2"
            shift 2
            ;;
        -o|--open)
            OPEN_REPORT=true
            shift
            ;;
        -q|--quick)
            QUICK=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            usage
            ;;
    esac
done

# If no specific benchmark selected, show usage
if [[ "$RUN_ALL" == false && "$RUN_CORE" == false && "$RUN_SECURITY" == false && "$RUN_VALIDATION" == false && "$RUN_DATA" == false ]]; then
    usage
fi

echo -e "${CYAN}╔════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║   Armature Benchmark Suite Runner     ║${NC}"
echo -e "${CYAN}╔════════════════════════════════════════╗${NC}"
echo ""

# Build benchmark options
BENCH_OPTS=""
if [[ "$QUICK" == true ]]; then
    BENCH_OPTS="$BENCH_OPTS --sample-size 100"
    echo -e "${YELLOW}⚡ Quick mode enabled (100 samples)${NC}"
fi

if [[ -n "$BASELINE" ]]; then
    BENCH_OPTS="$BENCH_OPTS --save-baseline $BASELINE"
    echo -e "${GREEN}💾 Saving baseline as: $BASELINE${NC}"
fi

if [[ -n "$COMPARE" ]]; then
    BENCH_OPTS="$BENCH_OPTS --baseline $COMPARE"
    echo -e "${BLUE}📊 Comparing with baseline: $COMPARE${NC}"
fi

echo ""

# Function to run a benchmark
run_benchmark() {
    local name=$1
    local bench_name=$2
    local icon=$3
    
    echo -e "${PURPLE}${icon} Running ${name}...${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    if cargo bench --bench "$bench_name" $BENCH_OPTS; then
        echo -e "${GREEN}✅ ${name} completed${NC}\n"
    else
        echo -e "${RED}❌ ${name} failed${NC}\n"
        exit 1
    fi
}

# Run benchmarks
START_TIME=$(date +%s)

if [[ "$RUN_ALL" == true ]]; then
    echo -e "${CYAN}Running all benchmark suites...${NC}\n"
    run_benchmark "Core Benchmarks" "core_benchmarks" "🔧"
    run_benchmark "Security Benchmarks" "security_benchmarks" "🔒"
    run_benchmark "Validation Benchmarks" "validation_benchmarks" "✅"
    run_benchmark "Data Benchmarks" "data_benchmarks" "💾"
else
    if [[ "$RUN_CORE" == true ]]; then
        run_benchmark "Core Benchmarks" "core_benchmarks" "🔧"
    fi
    
    if [[ "$RUN_SECURITY" == true ]]; then
        run_benchmark "Security Benchmarks" "security_benchmarks" "🔒"
    fi
    
    if [[ "$RUN_VALIDATION" == true ]]; then
        run_benchmark "Validation Benchmarks" "validation_benchmarks" "✅"
    fi
    
    if [[ "$RUN_DATA" == true ]]; then
        run_benchmark "Data Benchmarks" "data_benchmarks" "💾"
    fi
fi

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

# Summary
echo -e "${CYAN}╔════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║           Benchmark Summary            ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}✅ All benchmarks completed successfully${NC}"
echo -e "${BLUE}⏱️  Total time: ${DURATION}s${NC}"
echo ""

# Report location
REPORT_PATH="target/criterion/report/index.html"
if [[ -f "$REPORT_PATH" ]]; then
    echo -e "${YELLOW}📊 HTML report available at:${NC}"
    echo -e "   ${REPORT_PATH}"
    echo ""
    
    # Open report if requested
    if [[ "$OPEN_REPORT" == true ]]; then
        echo -e "${GREEN}🌐 Opening report in browser...${NC}"
        if command -v xdg-open &> /dev/null; then
            xdg-open "$REPORT_PATH"
        elif command -v open &> /dev/null; then
            open "$REPORT_PATH"
        else
            echo -e "${YELLOW}⚠️  Could not open browser automatically${NC}"
        fi
    fi
fi

# Performance tips
echo -e "${PURPLE}💡 Tips:${NC}"
echo "   • View detailed report: open $REPORT_PATH"
echo "   • Save baseline: $0 --all --baseline v0.1.0"
echo "   • Compare: $0 --all --compare v0.1.0"
echo "   • Quick test: $0 --all --quick"
echo ""

# List available baselines
BASELINE_DIR="target/criterion"
if [[ -d "$BASELINE_DIR" ]]; then
    BASELINES=$(find "$BASELINE_DIR" -type d -name "*.baseline" 2>/dev/null | wc -l)
    if [[ $BASELINES -gt 0 ]]; then
        echo -e "${BLUE}📂 Available baselines:${NC}"
        find "$BASELINE_DIR" -type d -name "*.baseline" -exec basename {} \; | sed 's/.baseline$//' | sed 's/^/   • /'
        echo ""
    fi
fi

echo -e "${CYAN}════════════════════════════════════════${NC}"

