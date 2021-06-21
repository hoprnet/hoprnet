#!/bin/bash

set -Eeuo pipefail
set -x

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="e2e-test"
source "${mydir}/../scripts/utils.sh"

usage() {
  msg
  msg "Usage: $0 <rest_ip:port> <admin_ip:port>"
  msg
}

declare rest_host_port="${1}"
declare admin_host_port="${2}"

log " - Security tests started"
log " - Rest API @ ${rest_host_port}"
log " - WS API @ ${admin_host_port}"

# should fail authentication without proper token
log " - Testing REST rejecting null token"
STATUS_CODE=$(curl -H "X-Auth-Token: bad-token" --output /dev/null --write-out "%{http_code}" --silent --max-time 360 -X POST --data "fake cmd" "${rest_host_port}/api/v1/command")
if [ ${STATUS_CODE} -ne 403 ]; then
  log " - Didn't get 403 with bad token"
  exit 1
fi

# should fail authentication without a token
log " - Testing REST rejecting bad token"
STATUS_CODE=$(curl --output /dev/null --write-out "%{http_code}" --silent --max-time 360 -X POST --data "fake cmd" "${rest_host_port}/api/v1/command")
if [ ${STATUS_CODE} -ne 403 ]; then
  log " - Didn't get 403 with no token"
  exit 1
fi

# should reject admin panel commands with no tocken
log " - Testing WS rejecting null token"
WS_RESPONSE=$(npx wscat --connect ws://${admin_host_port} --execute info)
if [ "${WS_RESPONSE}" != "authentication failed" ]; then
  log " - Didn't fail ws authentication with no token"
  log " - Expected response: 'authentication failed' "
  log " - Actual response:"
  log "${WS_RESPONSE}"
  exit 1
fi

# should reject admin panel commands with bad token
log " - Testing WS rejecting bad token"
WS_RESPONSE=$(npx wscat --connect ws://${admin_host_port} --execute info --header "Cookie:X-Auth-Token=bad-token")
if [ "${WS_RESPONSE}" != "authentication failed" ]; then
  log " - Didn't fail ws authentication with bad token"
  log " - Expected response: 'authentication failed' "
  log " - Actual response:"
  log "${WS_RESPONSE}"
  exit 1
fi

# should execute commands with right token
log " - Testing WS executing commands with right token"
WS_RESPONSE=$(npx wscat --connect ws://${admin_host_port} --execute info --header "Cookie:X-Auth-Token=e2e-api-token")
log "${WS_RESPONSE}" | grep -q "ws connection authenticated with token"

log " - Security tests finished successfully"
