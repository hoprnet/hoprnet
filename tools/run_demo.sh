#!/usr/bin/env bash
#
# Demo: drive a running HOPR node through an N-hop session and render the network graph.
#
# Prerequisites
# -------------
# The demo talks to an already-running hoprd (`API`, default http://127.0.0.1:3001/api/v4)
# that has funded channels. A local cluster needs a local chain, and the repo's chain image
# for that is the Blokli Anvil image already used by the `just localcluster` recipe:
#
#   docker rm -f anvil_blokli || true
#   docker run --rm --name anvil_blokli --platform linux/amd64 -p 8080:8080 -d \
#     europe-west3-docker.pkg.dev/hoprassociation/docker-images/bloklid-anvil:latest
#   cargo run -p hoprd-localcluster -- --chain-url http://localhost:8080 \
#     --hoprd-bin ./result/bin/hoprd --size 3
#
# (`just localcluster <size>` does all of the above; see the `localcluster` recipe in the
# justfile, which is the source of truth for both the image reference and the chain URL.)
# Then point the demo at one node's API:
#
#   API=http://127.0.0.1:3001/api/v4 TOKEN=<token> LOG=<that node's log> tools/run_demo.sh
#
# Set `LOG` to the hoprd log file of the node under test (the graph and session diagnostics are
# parsed out of it).
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
