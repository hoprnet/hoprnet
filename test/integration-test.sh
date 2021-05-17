#
#
#
#

source scripts/testnet.sh

NODE1="$(vm_name 'node-1' $TESTNET_NAME)"
NODE2="$(vm_name 'node-2' $TESTNET_NAME)"
NODE3="$(vm_name 'node-3' $TESTNET_NAME)"
IP1="$(gcloud_get_ip $NODE1)"
IP2="$(gcloud_get_ip $NODE2)"
IP3="$(gcloud_get_ip $NODE3)"
ETH_ADDRESS1="$(get_eth_address $IP1)"
ETH_ADDRESS2="$(get_eth_address $IP2)"
ETH_ADDRESS3="$(get_eth_address $IP3)"
echo "Node 1: $NODE1 IP: $IP1, ETH: $ETH_ADDRESS1"
echo "Node 2: $NODE2 IP: $IP2, ETH: $ETH_ADDRESS2"
echo "Node 3: $NODE3 IP: $IP3, ETH: $ETH_ADDRESS3"



echo "Query node-1"
echo "$(run_command $IP1 'balance')"
echo "$(run_command $IP1 'peers')"







#echo "Cleaning up after deploy"
#cleanup
