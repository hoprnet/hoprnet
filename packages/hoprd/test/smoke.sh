#/bin/bash
alias hoprd="node $0/../../lib/index.js"

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
BOB_ADDR=127.0.0.1
BOB_PORT=9876
hoprd --data='bob' --host="$BOB_ADDR:$BOB_PORT" --password="opensesame" --bootstrap &
BOB_PID="$!"
BOB_MULTIADDR=/ip4/$BOB_ADDR/tcp/$BOB_PORT/p2p/$BOB
export HOPR_BOOTSTRAP_SERVERS=$BOB_MULTIADDR 
echo "Bootstrap Bob running as pid $BOB_PID as $BOB on $BOB_ADDR"

# Ping bootstrapnode
hoprd --data='alice' --password="opensesame" --run="ping $BOB"






# Cleanup
kill $BOB_PID
