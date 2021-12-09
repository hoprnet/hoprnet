#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# need to pass an alias into sub-shells
shopt -s expand_aliases

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="e2e-security-test"
source "${mydir}/../scripts/utils.sh"

declare bad_token="bad_token"

usage() {
  msg
  msg "Usage: $0 <host> <rest_port> <admin_port> <insecure_admin_port> <api_token>"
  msg
}

declare host="${1}"
declare rest_port="${2}"
declare admin_port="${3}"
declare insecure_admin_port="${4}"
declare api_token="${5}"

log "Security tests started"
log "Rest API @ ${host}:${rest_port}"
log "Admin websocket API @ ${host}:${admin_port}"
log "No-auth admin websocket API @ ${host}:${insecure_admin_port}"

# prefer local websocat binary over global version
alias websocat=websocat
[ -x "${mydir}/../.bin/websocat" ] && alias websocat="${mydir}/../.bin/websocat"

# wait for input ports to be ready
wait_for_port "${rest_port}" "${host}"
wait_for_port "${admin_port}" "${host}"
wait_for_port "${insecure_admin_port}" "${host}"

declare http_status_code

log "REST API v1 should reject authentication with invalid token"
http_status_code=$(curl -H "X-Auth-Token: ${bad_token}" --output /dev/null --write-out "%{http_code}" --silent --max-time 360 -X POST --data "fake cmd" "${host}:${rest_port}/api/v1/command")
if [ ${http_status_code} -ne 403 ]; then
  log "⛔️ Expected 403 http status code, got ${http_status_code}"
  exit 1
fi

log "REST API v1 should reject authentication without token"
http_status_code=$(curl --output /dev/null --write-out "%{http_code}" --silent --max-time 360 -X POST --data "fake cmd" "${host}:${rest_port}/api/v1/command")
if [ ${http_status_code} -ne 403 ]; then
  log "⛔️ Expected 403 http status code, got ${http_status_code}"
  exit 1
fi

log "REST API v1 should accept authentication with valid token"
http_status_code=$(curl -H "X-Auth-Token: ${api_token}" --output /dev/null --write-out "%{http_code}" --silent --max-time 360 "${host}:${rest_port}/api/v1/version")
if [ ${http_status_code} -ne 200 ]; then
  log "⛔️ Expected 200 http status code, got ${http_status_code}"
  exit 1
fi

log "REST API v2 should reject authentication with invalid token"
http_status_code=$(curl -H "X-Auth-Token: ${bad_token}" --output /dev/null --write-out "%{http_code}" --silent --max-time 360 "${host}:${rest_port}/api/v2/node/version")
if [ ${http_status_code} -ne 403 ]; then
  log "⛔️ Expected 403 http status code, got ${http_status_code}"
  exit 1
fi

log "REST API v2 should reject authentication without token"
http_status_code=$(curl --output /dev/null --write-out "%{http_code}" --silent --max-time 360 "${host}:${rest_port}/api/v2/node/version")
if [ ${http_status_code} -ne 403 ]; then
  log "⛔️ Expected 403 http status code, got ${http_status_code}"
  exit 1
fi

log "REST API v2 should accept authentication with valid token"
http_status_code=$(curl -H "X-Auth-Token: ${api_token}" --output /dev/null --write-out "%{http_code}" --silent --max-time 360 "${host}:${rest_port}/api/v2/node/version")
if [ ${http_status_code} -ne 200 ]; then
  log "⛔️ Expected 200 http status code, got ${http_status_code}"
  exit 1
fi

log "REST API v2 should reject authentication with invalid basic auth credentials"
http_status_code=$(curl --basic --user "${bad_token}" --output /dev/null --write-out "%{http_code}" --silent --max-time 360 "${host}:${rest_port}/api/v2/node/version")
if [ ${http_status_code} -ne 403 ]; then
  log "⛔️ Expected 403 http status code, got ${http_status_code}"
  exit 1
fi

log "REST API v2 should reject authentication without basic auth credentials"
http_status_code=$(curl --output /dev/null --write-out "%{http_code}" --silent --max-time 360 "${host}:${rest_port}/api/v2/node/version")
if [ ${http_status_code} -ne 403 ]; then
  log "⛔️ Expected 403 http status code, got ${http_status_code}"
  exit 1
fi

log "REST API v2 should accept authentication with valid basic auth credentials"
http_status_code=$(curl --basic --user "${api_token}" --output /dev/null --write-out "%{http_code}" --silent --max-time 360 "${host}:${rest_port}/api/v2/node/version")
if [ ${http_status_code} -ne 200 ]; then
  log "⛔️ Expected 200 http status code, got ${http_status_code}"
  exit 1
fi

declare ws_response
declare msg_type

log "Admin websocket should reject commands without token"
ws_response=$(echo "info" | websocat ws://${host}:${admin_port}/)
msg_type=$(echo "${ws_response}" | jq .type --raw-output )

if [ "${msg_type}" != "auth-failed" ]; then
  log "⛔️ Didn't fail ws authentication"
  log "Expected response: '{ type: 'auth-failed' } "
  log "Actual response:"
  log "${ws_response}"
  log "Msg type:"
  log ${msg_type}
  exit 1
fi

log "Admin websocket should reject commands with invalid token"
ws_response=$(echo "info" | websocat ws://${host}:${admin_port}/ --header "Cookie:X-Auth-Token=${bad_token}")
msg_type=$(echo "${ws_response}" | jq .type --raw-output )

if [ "${msg_type}" != "auth-failed" ]; then
  log "⛔️ Didn't fail ws authentication"
  log "Expected response: '{ type: 'auth-failed' } "
  log "Actual response:"
  log "${ws_response}"
  log "Msg type:"
  log ${msg_type}
  exit 1
fi

log "Admin websocket should execute info command with correct token"
ws_response=$(echo "info" | websocat ws://${host}:${admin_port}/ -0 --header "Cookie:X-Auth-Token=${api_token}")
if [[ "${ws_response}" != *"ws client connected [ authentication ENABLED ]"* ]]; then
  log "⛔️ Didn't succeed ws authentication"
  log "Expected response should contain: 'ws client connected [ authentication ENABLED ]' "
  log "Actual response:"
  log "${ws_response}"
  exit 1
fi

log "No-auth admin websocket should auth with no token"
ws_response=$(echo "info" | websocat ws://${host}:${insecure_admin_port}/ -0)
if [[ "${ws_response}" != *"ws client connected [ authentication DISABLED ]"* ]]; then
  log "⛔️ Didn't succeed ws authentication"
  log "Expected response should contain: 'ws client connected [ authentication DISABLED ]' "
  log "Actual response:"
  log "${ws_response}"
  exit 1
fi

log "No-auth admin websocket should auth with bad token"
ws_response=$(echo "info" | websocat ws://${host}:${insecure_admin_port}/ -0 --header "Cookie:${bad_token}")
if [[ "${ws_response}" != *"ws client connected [ authentication DISABLED ]"* ]]; then
  log "⛔️ Didn't succeed ws authentication"
  log "Expected response should contain: 'ws client connected [ authentication DISABLED ]' "
  log "Actual response:"
  log "${ws_response}"
  exit 1
fi

log "Security tests finished successfully"
