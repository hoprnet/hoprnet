#/bin/bash
alias hoprd="node $0/../../lib/index.js"


# Variables
BOB_ADDR=127.0.0.1
BOB_PORT=9876
CHARLIE_ADDR=127.0.0.1
CHARLIE_PORT=9877


set -v

# Check databases and funded
#hoprd --data='alice'  --password="opensesame" --run "myAddress hopr" || exit 1
#hoprd --data='bob'  --password="opensesame" --run "myAddress hopr" || exit 1
#hoprd --data='charlie'  --password="opensesame" --run "myAddress hopr" || exit 1

# Store addresses
ALICE=$(hoprd --data='alice'  --password="opensesame" --run "myAddress hopr")
BOB=$(hoprd --data='bob' --password="opensesame" --run="myAddress hopr")
CHARLIE=$(hoprd --data='charlie'  --password="opensesame" --run "myAddress hopr")

# Run bob as bootstrap node
hoprd --data='bob' --host="$BOB_ADDR:$BOB_PORT" --password="opensesame" --bootstrap &
BOB_PID="$!"
BOB_MULTIADDR=/ip4/$BOB_ADDR/tcp/$BOB_PORT/p2p/$BOB
export HOPR_BOOTSTRAP_SERVERS=$BOB_MULTIADDR 
echo "Bootstrap Bob running as pid $BOB_PID on $BOB_MULTIADDR"

# Ping bootstrapnode
hoprd --data='alice' --password="opensesame" --run="ping $BOB"

# Start charlie
hoprd --data='charlie' --password="opensesame" --host="$CHARLIE_ADDR:$CHARLIE_PORT" &
CHARLIE_PID="$!"
echo "Charlie running as pid $CHARLIE as $CHARLIE on $CHARLIE_ADDR:$CHARLIE_PORT"

# Ping Charlie
hoprd --data='alice' --password="opensesame" --run="crawl; ping $CHARLIE"

# Open channel alice -> bob and send a-b-c
DEBUG=hopr* hoprd --data='alice' --password="opensesame" --run="crawl; open $BOB; send $CHARLIE hi"

# Multihop send self via Bob and Charlie
#DEBUG=hopr* hoprd --data='alice' --password="opensesame" --settings="{\"route\":\"$BOB,$CHARLIE\"}" --run="crawl; send $ALICE hi"




# Cleanup
kill $BOB_PID
kill $CHARLIE_PID
