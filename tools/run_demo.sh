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

# Step 2: Read our address from step 1
MY_ADDRESS=$(cat "$RESULTS_DIR/my_address")
echo ""
echo "=== Generating animated graph visualization ==="
echo "  Our address: $MY_ADDRESS"

# Step 3: Visualize with graph-timelapse
uv run --project "$SCRIPT_DIR/graph-timelapse" \
  graph-timelapse \
  "$LOG" \
  "$RESULTS_DIR/network_graph.dot" \
  --out-dir "$RESULTS_DIR" \
  --me "$MY_ADDRESS" \
  --no-gif

echo ""
echo "Results stored in: $RESULTS_DIR"
echo "  Graph HTML:  $RESULTS_DIR/graph.html"
echo "  Graph DOT:   $RESULTS_DIR/network_graph.dot"
echo "  Graph PNG:   $RESULTS_DIR/network_graph.png"
echo "  Test log:    $RESULTS_DIR/test_output.log"
