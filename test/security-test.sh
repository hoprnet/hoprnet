#!/bin/bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="e2e-test"
source "${mydir}/../scripts/utils.sh"

usage() {
  msg
  msg "Usage: $0 <host> <rest_port> <admin_port>"
  msg
}

declare host="${1}"
declare rest_port="${2}"
declare admin_port="${3}"

log "Security tests started"
log "Rest API @ ${host}:${rest_port}"
log "WS API @ ${host}:${admin_port}"

# test REST port open
nc -z ${host} ${rest_port}

# test admin port open
nc -z ${host} ${admin_port}

# should fail REST authentication without proper token
log "Testing REST rejecting null token"
STATUS_CODE=$(curl -H "X-Auth-Token: bad-token" --output /dev/null --write-out "%{http_code}" --silent --max-time 360 -X POST --data "fake cmd" "${host}:${rest_port}/api/v1/command")
if [ ${STATUS_CODE} -ne 403 ]; then
  log "Didn't get 403 with bad token"
  exit 1
fi

# should fail REST authentication without a token
log "Testing REST rejecting bad token"
STATUS_CODE=$(curl --output /dev/null --write-out "%{http_code}" --silent --max-time 360 -X POST --data "fake cmd" "${host}:${rest_port}/api/v1/command")
if [ ${STATUS_CODE} -ne 403 ]; then
  log "Didn't get 403 with no token"
  exit 1
fi

# should reject admin panel commands with no tocken
log "Testing WS rejecting with no token"
WS_RESPONSE=$(echo "info" | websocat ws://${host}:${admin_port}/)
if [ "${WS_RESPONSE}" != "authentication failed" ]; then
  log "Didn't fail ws authentication with no token"
  log "Expected response: 'authentication failed' "
  log "Actual response:"
  log "${WS_RESPONSE}"
  exit 1
fi

# should reject admin panel commands with bad token
log "Testing WS rejecting bad token"
WS_RESPONSE=$(echo "info" | websocat ws://${host}:${admin_port}/ --header "Cookie:X-Auth-Token=bad-token")
if [ "${WS_RESPONSE}" != "authentication failed" ]; then
  log "Didn't fail ws authentication with bad token"
  log "Expected response: 'authentication failed' "
  log "Actual response:"
  log "${WS_RESPONSE}"
  exit 1
fi

# should execute admin panel commands with right token
log "Testing WS executing commands with right token"
WS_RESPONSE=$(echo "info" | websocat ws://${host}:${admin_port}/ --header "Cookie:X-Auth-Token=e2e-api-token")
echo "${WS_RESPONSE}" | grep -q "ws connection authenticated with token"

log "Security tests finished successfully"
