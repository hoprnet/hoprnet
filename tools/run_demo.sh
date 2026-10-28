#!/usr/bin/env bash
set -euo pipefail

RESULTS_DIR="/tmp/hopr-demo-results"
mkdir -p "$RESULTS_DIR"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

API="${API:-http://127.0.0.1:3001/api/v4}"
TOKEN="${TOKEN:-hopr-demo}"
HOPS="${HOPS:-1}"
LOG="${LOG:-$HOME/demo/jura.log}"

echo "=== HOPR Demo ==="
echo "  API:     $API"
echo "  Token:   ${TOKEN:0:4}..."
echo "  Hops:    $HOPS"
echo "  Log:     $LOG"
echo "  Output:  $RESULTS_DIR"
echo ""

# Step 1: Run the test (extracts graph, channels, session, writes our address)
python3 "$SCRIPT_DIR/test_n_hop_session.py" \
  --api "$API" \
  --token "$TOKEN" \
  --hops "$HOPS" \
  --log "$LOG" \
  --graph-output "$RESULTS_DIR/network_graph" \
  --output "$RESULTS_DIR/downloaded_file" \
  --address-file "$RESULTS_DIR/my_address" \
  2>&1 | tee "$RESULTS_DIR/test_output.log"

echo ""
echo "Results stored in: $RESULTS_DIR"
echo "  Graph DOT:   $RESULTS_DIR/network_graph.dot"
echo "  Test log:    $RESULTS_DIR/test_output.log"
