#/bin/bash
alias hoprd="node $0/../../lib/index.js"

# Generate databases
hoprd --data='alice'  --password="opensesame" --run "myAddress hopr" || true
hoprd --data='bob'  --password="opensesame" --run "myAddress hopr" || true
hoprd --data='charlie'  --password="opensesame" --run "myAddress hopr" || true
hoprd --data='dave'  --password="opensesame" --run "myAddress hopr" || true

# Run bob as bootstrap node
hoprd --data='bob' --password="opensesame" --bootstrap &


