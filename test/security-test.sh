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
log "Admin websocket API @ ${host}:${admin_port}"

wait_for_port ${rest_port}
wait_for_port ${admin_port}

declare http_status_code

log "REST API should reject authentication without token"
http_status_code=$(curl -H "X-Auth-Token: bad-token" --output /dev/null --write-out "%{http_code}" --silent --max-time 360 -X POST --data "fake cmd" "${host}:${rest_port}/api/v1/command")
if [ ${http_status_code} -ne 403 ]; then
  log "⛔️ Expected 403 http status code, got ${http_status_code}"
  exit 1
fi

log "REST API should reject authentication with invalid token"
http_status_code=$(curl --output /dev/null --write-out "%{http_code}" --silent --max-time 360 -X POST --data "fake cmd" "${host}:${rest_port}/api/v1/command")
if [ ${http_status_code} -ne 403 ]; then
  log "⛔️ Expected 403 http status code, got ${http_status_code}"
  exit 1
fi

declare ws_response

log "Admin websocket should reject commands without token"
ws_response=$(echo "info" | websocat ws://${host}:${admin_port}/)
if [ "${ws_response}" != "authentication failed" ]; then
  log "⛔️ Didn't fail ws authentication"
  log "Expected response: 'authentication failed' "
  log "Actual response:"
  log "${ws_response}"
  exit 1
fi

log "Admin websocket should reject commands with invalid token"
ws_response=$(echo "info" | websocat ws://${host}:${admin_port}/ --header "Cookie:X-Auth-Token=bad-token")
if [ "${ws_response}" != "authentication failed" ]; then
  log "⛔️ Didn't fail ws authentication"
  log "Expected response: 'authentication failed' "
  log "Actual response:"
  log "${ws_response}"
  exit 1
fi

log "Admin websocket should execute info command with correct token"
ws_response=$(echo "info" | websocat ws://${host}:${admin_port}/ --header "Cookie:X-Auth-Token=e2e-api-token")
echo "${ws_response}" | grep -q "ws connection authenticated with token"

log "Security tests finished successfully"
