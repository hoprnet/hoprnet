#!/usr/bin/env bash

BINARY="$1"
DUMP_FILE="$2"
OUTPUT_PREFIX="$3"

if [ $# -ne 3 ]; then
  echo "Usage: $0 <binary_path> <dump_file> <output_prefix>"
  exit 1
fi

echo "Generating comprehensive analysis for $DUMP_FILE..."

# 1. Overall memory usage
jeprof --show_bytes --svg "$BINARY" "$DUMP_FILE" >"${OUTPUT_PREFIX}_overview.svg"

# 2. Top memory consumers
jeprof --show_bytes --nodecount=20 --svg "$BINARY" "$DUMP_FILE" >"${OUTPUT_PREFIX}_top20.svg"

# 3. Filtered view (significant allocations only)
jeprof --show_bytes --nodefraction=0.01 --edgefraction=0.01 --svg "$BINARY" "$DUMP_FILE" >"${OUTPUT_PREFIX}_significant.svg"

# 4. Call graph with line numbers
jeprof --show_bytes --lines --svg "$BINARY" "$DUMP_FILE" >"${OUTPUT_PREFIX}_detailed.svg"

# 5. Object count analysis
jeprof --alloc_objects --svg "$BINARY" "$DUMP_FILE" >"${OUTPUT_PREFIX}_objects.svg"

# Focus on Rust-specific allocations
jeprof --show_bytes --focus="rust_|std::|tokio::|serde::" --svg "$BINARY" "$DUMP_FILE" >"${OUTPUT_PREFIX}_rust_specific.svg"

# Ignore Rust runtime allocations to see application logic
jeprof --show_bytes --ignore="rust_begin_unwind|__rust_|std::panic" "$BINARY" "$DUMP_FILE" >"${OUTPUT_PREFIX}_rust_logic.svg"

# Show only large allocations (useful for finding memory hogs)
jeprof --show_bytes --nodefraction=0.05 --edgefraction=0.01 --svg "$BINARY" "$DUMP_FILE" >"${OUTPUT_PREFIX}_rust_allocs.svg"

echo "Analysis complete:"
echo "  - Overview: ${OUTPUT_PREFIX}_overview.svg"
echo "  - Top 20: ${OUTPUT_PREFIX}_top20.svg"
echo "  - Significant: ${OUTPUT_PREFIX}_significant.svg"
echo "  - Detailed: ${OUTPUT_PREFIX}_detailed.svg"
echo "  - Objects: ${OUTPUT_PREFIX}_objects.svg"
echo "  - Rust Specific: ${OUTPUT_PREFIX}_rust_specific.svg"
echo "  - Rust Logic: ${OUTPUT_PREFIX}_rust_logic.svg"
echo "  - Rust Alloc: ${OUTPUT_PREFIX}_rust_allocs.svg"
