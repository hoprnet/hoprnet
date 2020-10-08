#/bin/bash
alias hoprd="node $0/../../lib/index.js"

set -v
set -e

# Check databases and funded
hoprd --data='alice'  --password="opensesame" --run "myAddress hopr"
hoprd --data='bob'  --password="opensesame" --run "myAddress hopr"
hoprd --data='charlie'  --password="opensesame" --run "myAddress hopr"

# Store addresses
ALICE=$(hoprd --data='alice'  --password="opensesame" --run "myAddress hopr")
BOB=$(hoprd --data='bob' --password="opensesame" --run="myAddress hopr")
CHARLIE=$(hoprd --data='charlie'  --password="opensesame" --run "myAddress hopr")

# Run bob as bootstrap node
BOB_PID=hoprd --data='bob' --password="opensesame" --bootstrap &


# Ping bootstrapnode
hoprd --data='alice' --password="opensesame" --run="ping ${BOB}"


