#!/bin/bash

set -e 

declare rest_host_port="${1}"
declare admin_host_port="${2}"

echo " - Security tests started"
echo " - Rest API @ ${rest_host_port}"
echo " - WS API @ ${admin_host_port}"

# should fail authentication without proper token
echo " - Testing REST rejecting null token"
STATUS_CODE=$(curl -H "X-Auth-Token: bad-token" --output /dev/null --write-out "%{http_code}" --silent --max-time 360 -X POST --data "fake cmd" "${rest_host_port}/api/v1/command")
if [ ${STATUS_CODE} -ne 403 ]; then
  echo " - Didn't get 403 with bad token"
  exit 1
fi

# should fail authentication without a token
echo " - Testing REST rejecting bad token"
STATUS_CODE=$(curl --output /dev/null --write-out "%{http_code}" --silent --max-time 360 -X POST --data "fake cmd" "${rest_host_port}/api/v1/command")
if [ ${STATUS_CODE} -ne 403 ]; then
  echo " - Didn't get 403 with no token"
  exit 1
fi

# should reject admin panel commands with no tocken
echo " - Testing WS rejecting null token"
WS_RESPONSE=$(npx wscat --connect ws://${admin_host_port} --execute info)
if [ "${WS_RESPONSE}" != "authentication failed" ]; then
  echo " - Didn't fail ws authentication with no token"
  echo " - Expected response: 'authentication failed' "
  echo " - Actual response:"
  echo "${WS_RESPONSE}"
  exit 1
fi

# should reject admin panel commands with bad token
echo " - Testing WS rejecting bad token"
WS_RESPONSE=$(npx wscat --connect ws://${admin_host_port} --execute info --header "Cookie:X-Auth-Token=bad-token")
if [ "${WS_RESPONSE}" != "authentication failed" ]; then
  echo " - Didn't fail ws authentication with bad token"
  echo " - Expected response: 'authentication failed' "
  echo " - Actual response:"
  echo "${WS_RESPONSE}"
  exit 1
fi

# should execute commands with right token
echo " - Testing WS executing commands with right token"
WS_RESPONSE=$(npx wscat --connect ws://${admin_host_port} --execute info --header "Cookie:X-Auth-Token=e2e-api-token")
echo " - Actual response:"
echo "${WS_RESPONSE}"
echo "${WS_RESPONSE}" | grep -q "ws connection authenticated with token"

echo " - Security tests finished successfully"
