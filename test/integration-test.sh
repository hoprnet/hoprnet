# -- Integration test --
# We assume the existence of a test network with three nodes:
# API1, API2 API3.

# $1 = IP
# $2 = Hopr command
run_command(){
  curl --silent -X POST --data "$2" $1/api/v1/command
}
get_eth_address(){
  echo $(curl $1/api/v1/address/eth)
}

get_hopr_address(){
  echo $(curl $1/api/v1/address/hopr)
}

validate_ip() {
if [ -z "$1" ]; then
  echo "missing ip $1"
  exit 1
fi
}

validate_node_eth_address() {
ETH_ADDRESS="$(get_eth_address $1)"
if [ -z "$ETH_ADDRESS" ]; then
  echo "missing ETH_ADDRESS $1"
  exit 1
fi
IS_VALID_ETH_ADDRESS1=$(node -e \
  "const ethers = require('ethers'); console.log(ethers.utils.isAddress('$ETH_ADDRESS'))")
if [ "$IS_VALID_ETH_ADDRESS" == "false" ]; then
  echo "⛔️ Node outputs an invalid address: $ETH_ADDRESS $1"
  exit 1
fi
echo $ETH_ADDRESS
}


# TODO better validation
validate_node_balance_gt0() {
BALANCE="$(run_command $1 'balance')"
ETH_BALANCE=$(echo -e "$BALANCE" | grep -c " xDAI" || true)
HOPR_BALANCE=$(echo -e "$BALANCE" | grep -c " HOPR" || true)
if [[ "$ETH_BALANCE" != "0" && "$HOPR_BALANCE" != "Hopr Balance: 0 HOPR" ]]; then
  echo "- $1 is funded"
else
  echo "⛔️ $1 Node has an invalid balance: $ETH_BALANCE, $HOPR_BALANCE"
  echo -e "$BALANCE"
  exit 1
fi
}



echo "- Running full E2E test with $API1, $API2, $API3"
validate_ip $API1
validate_ip $API2
validate_ip $API3

ETH_ADDRESS1="$(validate_node_eth_address $API1)"
ETH_ADDRESS2="$(validate_node_eth_address $API2)"
ETH_ADDRESS3="$(validate_node_eth_address $API3)"

validate_node_balance_gt0 $API1
validate_node_balance_gt0 $API2
echo "- Nodes are funded"

echo "$(run_command $API1 'peers')"
HOPR_ADDRESS1=$(get_hopr_address $API1)
echo "HOPR_ADDRESS1: $HOPR_ADDRESS1"

HOPR_ADDRESS2=$(get_hopr_address $API2)
echo "HOPR_ADDRESS2: $HOPR_ADDRESS2"

echo "- Node 1 ping node 2: $(run_command $API1 "ping $HOPR_ADDRESS2")"

echo "- Node 1 tickets: $(run_command $API1 'tickets')"

echo "- Node 1 send 0-hop message to node 2"
run_command $API1 "send ,$HOPR_ADDRESS2 'hello, world'"

echo "- Node 1 open channel to Node 2"
run_command $API1 "open $HOPR_ADDRESS2 0.1" 

echo "- Node 1 send 1 hop message to self via node 2"
run_command $API1 "send $HOPR_ADDRESS2,$HOPR_ADDRESS1 'hello, world'"

echo "- Node 2 should now have a ticket"
run_command $API2 "tickets"



