#!/usr/bin/env bash

# prevent execution of this script, only allow sourcing
$(return >/dev/null 2>&1)
test "$?" -eq "0" || { echo "This script should only be sourced." >&2; exit 1; }

export apiToken="^^LOCAL-testing-123^^"
export HOPR_NODE_1_HTTP_URL=$(gp url 13301)
export HOPR_NODE_1_WS_URL=$(gp url 19501 | sed 's/https/wss/')
export HOPR_NODE_1_ADDR=$(curl --silent -H "x-auth-token: $apiToken" "$HOPR_NODE_1_HTTP_URL/api/v3/account/addresses" | jq -r '.hopr')
export HOPR_NODE_2_HTTP_URL=$(gp url 13302)
export HOPR_NODE_2_WS_URL=$(gp url 19502 | sed 's/https/wss/')
export HOPR_NODE_2_ADDR=$(curl --silent -H "x-auth-token: $apiToken" "$HOPR_NODE_2_HTTP_URL/api/v3/account/addresses" | jq -r '.hopr')
export HOPR_NODE_3_HTTP_URL=$(gp url 13303)
export HOPR_NODE_3_WS_URL=$(gp url 19503 | sed 's/https/wss/')
export HOPR_NODE_3_ADDR=$(curl --silent -H "x-auth-token: $apiToken" "$HOPR_NODE_3_HTTP_URL/api/v3/account/addresses" | jq -r '.hopr')
export HOPR_NODE_4_HTTP_URL=$(gp url 13304)
export HOPR_NODE_4_WS_URL=$(gp url 19504 | sed 's/https/wss/')
export HOPR_NODE_4_ADDR=$(curl --silent -H "x-auth-token: $apiToken" "$HOPR_NODE_4_HTTP_URL/api/v3/account/addresses" | jq -r '.hopr')
export HOPR_NODE_5_HTTP_URL=$(gp url 13305)
export HOPR_NODE_5_WS_URL=$(gp url 19505 | sed 's/https/wss/')
export HOPR_NODE_5_ADDR=$(curl --silent -H "x-auth-token: $apiToken" "$HOPR_NODE_5_HTTP_URL/api/v3/account/addresses" | jq -r '.hopr')

echo -e "\n"
echo "🌐 Node 1 REST API URL:  $HOPR_NODE_1_HTTP_URL"
echo "🔌 Node 1 WebSocket URL: $HOPR_NODE_1_WS_URL"
echo "💻 Node 1 HOPR Address:  $HOPR_NODE_1_ADDR"
echo "---"
echo "🌐 Node 2 REST API URL:  $HOPR_NODE_2_HTTP_URL"
echo "🔌 Node 2 WebSocket URL: $HOPR_NODE_2_WS_URL"
echo "💻 Node 2 HOPR Address:  $HOPR_NODE_2_ADDR"
echo "---"
echo "🌐 Node 3 REST API URL:  $HOPR_NODE_3_HTTP_URL"
echo "🔌 Node 3 WebSocket URL: $HOPR_NODE_3_WS_URL"
echo "💻 Node 3 HOPR Address:  $HOPR_NODE_3_ADDR"
echo "---"
echo "🌐 Node 4 REST API URL:  $HOPR_NODE_4_HTTP_URL"
echo "🔌 Node 4 WebSocket URL: $HOPR_NODE_4_WS_URL"
echo "💻 Node 4 HOPR Address:  $HOPR_NODE_4_ADDR"
echo "---"
echo "🌐 Node 5 REST API URL:  $HOPR_NODE_5_HTTP_URL"
echo "🔌 Node 5 WebSocket URL: $HOPR_NODE_5_WS_URL"
echo "💻 Node 5 HOPR Address:  $HOPR_NODE_5_ADDR"
echo -e "\n"
