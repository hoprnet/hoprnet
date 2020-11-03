#!/bin/bash
set -e

# Variables
BOB_ADDR=127.0.0.1
BOB_PORT=9876
CHARLIE_ADDR=127.0.0.1
CHARLIE_PORT=9877

# Store addresses
ALICE=$(hoprd --data='./test/fixtures/alice' --password="$DBPASS" --bootstrap --run "myAddress hopr")
BOB=$(hoprd --data='./test/fixtures/bob' --password="$DBPASS" --bootstrap --run="myAddress hopr")
CHARLIE=$(hoprd --data='./test/fixtures/charlie' --password="$DBPASS" --bootstrap --run "myAddress hopr")

# Run bob as bootstrap node
hoprd --data='./test/fixtures/bob' --password="$DBPASS" --host="$BOB_ADDR:$BOB_PORT" --bootstrap &
BOB_PID="$!"
BOB_MULTIADDR=/ip4/$BOB_ADDR/tcp/$BOB_PORT/p2p/$BOB
export HOPR_BOOTSTRAP_SERVERS=$BOB_MULTIADDR 
echo "Bootstrap Bob running as pid $BOB_PID on $BOB_MULTIADDR"

# Ping bootstrapnode
hoprd --data='./test/fixtures/alice' --password="$DBPASS" --run="info"
hoprd --data='./test/fixtures/alice' --password="$DBPASS" --run="ping $BOB"
hoprd --data='./test/fixtures/alice' --password="$DBPASS" --run="crawl; listConnectedPeers "

# Start charlie
hoprd --data='./test/fixtures/charlie' --password="$DBPASS" --host="$CHARLIE_ADDR:$CHARLIE_PORT" &
CHARLIE_PID="$!"
echo "Charlie running as pid $CHARLIE as $CHARLIE on $CHARLIE_ADDR:$CHARLIE_PORT"

# Ping Charlie
hoprd --data='./test/fixtures/alice' --password="$DBPASS" --run="crawl; ping $CHARLIE"

# Open channel alice -> bob and send a-b-c
DEBUG=hopr* hoprd --data='./test/fixtures/alice' --password="$DBPASS" --run="crawl; open $BOB 10; send $CHARLIE hi"

# Cleanup
kill $BOB_PID
kill $CHARLIE_PID
