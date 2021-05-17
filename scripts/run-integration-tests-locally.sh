source 'test/e2e/0_configuration.sh'
source scripts/testnet.sh

# Running RPC
rpc_network

# Run node 1
API1="127.0.0.1:3001"
DEBUG="hopr*" hoprd --data="/tmp/NODE1"  > "/tmp/NODE1-log.txt" 2>&1 &
NODE1_PID="$!"

ETH_ADDRESS=$(curl -s http://$API1/api/v1/address/eth)

