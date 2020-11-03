#!/bin/bash
set -e
shopt -s expand_aliases

alias hoprd="node packages/hoprd/lib/index.js"

# Variables
BOB_ADDR=127.0.0.1
BOB_PORT=9876
CHARLIE_ADDR=127.0.0.1
CHARLIE_PORT=9877

# Check Databases
echo "alice"
hoprd --data="$FIXTURES/alice" --password="$DBPASS" --init --bootstrap --run "myAddress" 
hoprd --data="$FIXTURES/alice" --password="$DBPASS" --bootstrap --run "balance" 

echo "bob"
hoprd --data="$FIXTURES/bob" --password="$DBPASS" --init --bootstrap --run "myAddress"
hoprd --data="$FIXTURES/bob" --password="$DBPASS" --bootstrap --run "balance"

echo "charlie"
hoprd --data="$FIXTURES/charlie" --password="$DBPASS" --init --bootstrap --run "myAddress"
hoprd --data="$FIXTURES/charlie" --password="$DBPASS" --bootstrap --run "balance"

# Store addresses
ALICE=$(hoprd --data="$FIXTURES/alice" --password="$DBPASS" --bootstrap --run "myAddress hopr")
BOB=$(hoprd --data="$FIXTURES/bob" --password="$DBPASS" --bootstrap --run="myAddress hopr")
CHARLIE=$(hoprd --data="$FIXTURES/charlie" --password="$DBPASS" --bootstrap --run "myAddress hopr")

function finish {
  # Cleanup
  if [[ -n "$BOB_PID" ]]; then kill $BOB_PID; fi
  if [[ -n "$CHARLIE_PID" ]]; then kill $CHARLIE_PID; fi
}
trap finish EXIT

# Run bob as bootstrap node
hoprd --data="$FIXTURES/bob" --password="$DBPASS" --host="$BOB_ADDR:$BOB_PORT" --bootstrap &
BOB_PID="$!"
BOB_MULTIADDR=/ip4/$BOB_ADDR/tcp/$BOB_PORT/p2p/$BOB
export HOPR_BOOTSTRAP_SERVERS=$BOB_MULTIADDR 
echo "Bootstrap Bob running as pid $BOB_PID on $BOB_MULTIADDR"

# Ping bootstrapnode
hoprd --data="$FIXTURES/alice" --password="$DBPASS" --run="info"
hoprd --data="$FIXTURES/alice" --password="$DBPASS" --run="ping $BOB"
hoprd --data="$FIXTURES/alice" --password="$DBPASS" --run="crawl; listConnectedPeers "

# Start charlie
hoprd --data="$FIXTURES/charlie" --password="$DBPASS" --host="$CHARLIE_ADDR:$CHARLIE_PORT" &
CHARLIE_PID="$!"
echo "Charlie running as pid $CHARLIE as $CHARLIE on $CHARLIE_ADDR:$CHARLIE_PORT"

# Ping Charlie
hoprd --data="$FIXTURES/alice" --password="$DBPASS" --run="crawl; ping $CHARLIE"

# Open channel alice -> bob and send a-b-c
#DEBUG=hopr* hoprd --data="$FIXTURES/alice" --password="$DBPASS" --run="crawl; open $BOB 0.01; send $CHARLIE hi"

