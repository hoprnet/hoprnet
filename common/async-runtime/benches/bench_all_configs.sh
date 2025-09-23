#!/usr/bin/env bash

# Channel Metrics Benchmark Script
# 
# This script runs the channel_metrics_simple benchmark across all supported feature configurations
# to measure the performance impact of channel monitoring in hopr-async-runtime.
#
# Supported Configurations:
# - tokio_prometheus: Tokio runtime with Prometheus metrics
# - tokio_no_prometheus: Tokio runtime without metrics (zero overhead)
# - futures_prometheus: Futures runtime with Prometheus metrics  
# - futures_no_prometheus: Futures runtime without metrics (zero overhead)
#
# Usage Examples:
#   ./bench_all_configs.sh                    # Run all configurations
#   ./bench_all_configs.sh -c tokio_prometheus -q  # Quick test of one config
#   ./bench_all_configs.sh --help             # Show detailed help
#
# Results are saved to bench_results/ directory with timestamps for comparison.

set -e

PACKAGE="hopr-async-runtime"
RESULTS_DIR="bench_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
SCRIPT_NAME=$(basename "$0")

# Default values
RUN_ALL=true
SELECTED_CONFIG=""
QUICK_MODE=false
VERBOSE=false

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to display help
show_help() {
	cat << EOF
Usage: $SCRIPT_NAME [OPTIONS]

Benchmark script for hopr-async-runtime channel metrics performance testing.
Runs the simple benchmark suite (channel_metrics_simple) across different feature configurations.
Compares performance to measure monitoring overhead with reliable cross-configuration compatibility.

OPTIONS:
    -h, --help              Show this help message and exit
    -c, --config CONFIG     Run specific configuration only
                           Available configs:
                             - tokio_prometheus    (Tokio with Prometheus metrics)
                             - tokio_no_prometheus (Tokio without metrics)
                             - futures_prometheus  (Futures with Prometheus metrics)
                             - futures_no_prometheus (Futures without metrics)
                             - all (default)       (Run all configurations)
    -q, --quick             Run quick benchmarks (reduced sample size)
    -v, --verbose           Show detailed benchmark output
    -o, --output DIR        Specify output directory (default: bench_results)
    --clean                 Clean previous benchmark results before running
    --compare               Compare results from previous runs
    --baseline NAME         Save results as baseline with given name
    --vs-baseline NAME      Compare against named baseline

EXAMPLES:
    # Run all benchmarks (default)
    $SCRIPT_NAME

    # Run only Tokio with Prometheus configuration
    $SCRIPT_NAME -c tokio_prometheus

    # Quick benchmark run with verbose output
    $SCRIPT_NAME -q -v

    # Save results as baseline for future comparison
    $SCRIPT_NAME --baseline main

    # Compare against previously saved baseline
    $SCRIPT_NAME --vs-baseline main

    # Clean old results and run fresh benchmarks
    $SCRIPT_NAME --clean

BENCHMARK GROUPS:
    - channel_send:     Message throughput testing
    - try_send:        Non-blocking operation performance
    - concurrent:      Multi-sender contention testing
    - stream:          Stream trait implementation overhead
    - channel_creation: Memory and initialization overhead

EXPECTED PERFORMANCE:
    - Tokio + Prometheus:    ~5-10% overhead (accurate metrics)
    - Futures + Prometheus:  ~2-5% overhead (timing only)
    - Without Prometheus:    ~0% overhead (zero-cost abstraction)

OUTPUT:
    Results are saved to $RESULTS_DIR/ with timestamps
    Each configuration generates a separate result file

For more information, see benches/README.md

EOF
	exit 0
}

# Function to display error and exit
error_exit() {
	echo -e "${RED}Error: $1${NC}" >&2
	echo "Use '$SCRIPT_NAME --help' for usage information"
	exit 1
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
	case $1 in
		-h|--help)
			show_help
			;;
		-c|--config)
			SELECTED_CONFIG="$2"
			RUN_ALL=false
			shift 2
			;;
		-q|--quick)
			QUICK_MODE=true
			shift
			;;
		-v|--verbose)
			VERBOSE=true
			shift
			;;
		-o|--output)
			RESULTS_DIR="$2"
			shift 2
			;;
		--clean)
			echo "Cleaning previous benchmark results..."
			rm -rf "$RESULTS_DIR"
			shift
			;;
		--compare)
			if [ -d "$RESULTS_DIR" ]; then
				echo -e "${BLUE}Previous benchmark results:${NC}"
				ls -la "$RESULTS_DIR"/*.txt 2>/dev/null || echo "No previous results found"
				exit 0
			else
				echo "No previous results found in $RESULTS_DIR"
				exit 0
			fi
			;;
		--baseline)
			BASELINE_NAME="$2"
			shift 2
			;;
		--vs-baseline)
			VS_BASELINE="$2"
			shift 2
			;;
		*)
			error_exit "Unknown option: $1"
			;;
	esac
done

# Validate selected configuration
if [ "$RUN_ALL" = false ]; then
	case $SELECTED_CONFIG in
		tokio_prometheus|tokio_no_prometheus|futures_prometheus|futures_no_prometheus|all)
			if [ "$SELECTED_CONFIG" = "all" ]; then
				RUN_ALL=true
			fi
			;;
		*)
			error_exit "Invalid configuration: $SELECTED_CONFIG"
			;;
	esac
fi

# Create results directory
mkdir -p "$RESULTS_DIR"

# Set benchmark flags based on options
BENCH_FLAGS=""
if [ "$QUICK_MODE" = true ]; then
	BENCH_FLAGS="$BENCH_FLAGS -- --sample-size 10 --measurement-time 5"
fi

if [ "$VS_BASELINE" != "" ]; then
	BENCH_FLAGS="$BENCH_FLAGS -- --baseline $VS_BASELINE"
fi

if [ "$BASELINE_NAME" != "" ]; then
	BENCH_FLAGS="$BENCH_FLAGS -- --save-baseline $BASELINE_NAME"
fi

echo -e "${GREEN}Running channel metrics benchmarks${NC}"
echo "Configuration: ${SELECTED_CONFIG:-all}"
echo "Output directory: $RESULTS_DIR"
[ "$QUICK_MODE" = true ] && echo -e "${YELLOW}Quick mode enabled (reduced samples)${NC}"
[ "$VERBOSE" = true ] && echo "Verbose output enabled"
echo ""

# Function to run benchmark and save results
run_bench() {
	local name=$1
	local features=$2
	local no_default=$3

	echo -e "${BLUE}========================================${NC}"
	echo -e "${BLUE}Running: $name${NC}"
	echo -e "${BLUE}Features: $features${NC}"
	echo -e "${BLUE}========================================${NC}"

	local cmd="cargo bench -p $PACKAGE --features $features --bench channel_metrics_simple"
	if [ "$no_default" = "yes" ]; then
		cmd="$cmd --no-default-features"
	fi
	
	# Add benchmark flags
	if [ "$BENCH_FLAGS" != "" ]; then
		cmd="$cmd $BENCH_FLAGS"
	fi

	# Execute benchmark
	if [ "$VERBOSE" = true ]; then
		echo "Executing: $cmd"
		eval "$cmd" 2>&1 | tee "$RESULTS_DIR/${name}_${TIMESTAMP}.txt"
	else
		eval "$cmd" 2>&1 | tee "$RESULTS_DIR/${name}_${TIMESTAMP}.txt" | grep -E "(Benchmarking|time:|found)"
	fi

	echo ""
}

# Function to run specific configuration
run_config() {
	local config=$1
	case $config in
		tokio_prometheus)
			run_bench "tokio_with_prometheus" "runtime-tokio,prometheus" "no"
			;;
		tokio_no_prometheus)
			run_bench "tokio_no_prometheus" "runtime-tokio" "yes"
			;;
		futures_prometheus)
			run_bench "futures_with_prometheus" "runtime-futures,prometheus" "yes"
			;;
		futures_no_prometheus)
			run_bench "futures_no_prometheus" "runtime-futures" "yes"
			;;
		*)
			error_exit "Unknown configuration: $config"
			;;
	esac
}

# Run benchmarks based on selected configuration
if [ "$RUN_ALL" = true ]; then
	echo -e "${GREEN}Running all configurations...${NC}"
	echo ""
	
	# Configuration 1: Tokio with Prometheus (default setup)
	run_bench "tokio_with_prometheus" "runtime-tokio,prometheus" "no"

	# Configuration 2: Tokio without Prometheus
	run_bench "tokio_no_prometheus" "runtime-tokio" "yes"

	# Configuration 3: Futures with Prometheus
	run_bench "futures_with_prometheus" "runtime-futures,prometheus" "yes"

	# Configuration 4: Futures without Prometheus
	run_bench "futures_no_prometheus" "runtime-futures" "yes"
else
	echo -e "${GREEN}Running configuration: $SELECTED_CONFIG${NC}"
	echo ""
	run_config "$SELECTED_CONFIG"
fi

# Generate summary
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Benchmark Summary${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "${BLUE}Results saved to: $RESULTS_DIR/${NC}"
echo ""

# Determine which configurations were run
if [ "$RUN_ALL" = true ]; then
	CONFIGS_RUN="tokio_with_prometheus tokio_no_prometheus futures_with_prometheus futures_no_prometheus"
else
	case $SELECTED_CONFIG in
		tokio_prometheus)
			CONFIGS_RUN="tokio_with_prometheus"
			;;
		tokio_no_prometheus)
			CONFIGS_RUN="tokio_no_prometheus"
			;;
		futures_prometheus)
			CONFIGS_RUN="futures_with_prometheus"
			;;
		futures_no_prometheus)
			CONFIGS_RUN="futures_no_prometheus"
			;;
	esac
fi

echo -e "${YELLOW}Performance Comparison:${NC}"
echo ""

# Extract and display key metrics
for config in $CONFIGS_RUN; do
	echo -e "${BLUE}$config:${NC}"
	if [ -f "$RESULTS_DIR/${config}_${TIMESTAMP}.txt" ]; then
		# Try to extract key metrics
		grep -A2 "channel_send/monitored/buf_100_msgs_1000" "$RESULTS_DIR/${config}_${TIMESTAMP}.txt" 2>/dev/null || \
		grep -A1 "channel_creation/monitored/100" "$RESULTS_DIR/${config}_${TIMESTAMP}.txt" 2>/dev/null || \
		echo "  No standard metrics found - check full results file"
	else
		echo "  Results file not found"
	fi
	echo ""
done

echo -e "${YELLOW}Result files:${NC}"
for config in $CONFIGS_RUN; do
	if [ -f "$RESULTS_DIR/${config}_${TIMESTAMP}.txt" ]; then
		echo "  $RESULTS_DIR/${config}_${TIMESTAMP}.txt"
	fi
done
echo ""

echo -e "${YELLOW}Commands for detailed analysis:${NC}"
echo "  # View specific result file:"
echo "  less $RESULTS_DIR/<config>_${TIMESTAMP}.txt"
echo ""
echo "  # Compare with criterion:"
if [ "$BASELINE_NAME" != "" ]; then
	echo "  cargo bench -p $PACKAGE -- --baseline $BASELINE_NAME"
else
	echo "  cargo bench -p $PACKAGE -- --save-baseline before_changes"
	echo "  # (make changes, then:)"
	echo "  cargo bench -p $PACKAGE -- --baseline before_changes"
fi
echo ""

echo -e "${YELLOW}Expected performance overhead:${NC}"
echo "  - Tokio with Prometheus:    ~5-10% overhead (accurate metrics)"
echo "  - Futures with Prometheus:  ~2-5% overhead (timing only)"
echo "  - Without Prometheus:       ~0% overhead (zero-cost abstraction)"
echo ""

if [ "$QUICK_MODE" = true ]; then
	echo -e "${YELLOW}Note: Quick mode was used - results may be less precise${NC}"
	echo "For production measurements, run without -q/--quick flag"
	echo ""
fi

echo -e "${GREEN}For more information, see benches/README.md${NC}"

