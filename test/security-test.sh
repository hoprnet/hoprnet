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
  msg "Usage: $0 <host> <api_port> <insecure_api_port> <admin_port> <insecure_admin_port> <api_token>"
  msg
}

declare host="${1}"
declare api_port="${2}"
declare insecure_api_port="${3}"
declare admin_port="${4}"
declare insecure_admin_port="${5}"
declare api_token="${6}"

log "Security tests started"
log "API @ ${host}:${api_port}"
log "No-auth API @ ${host}:${insecure_api_port}"
log "Admin websocket API @ ${host}:${admin_port}"
log "No-auth admin websocket API @ ${host}:${insecure_admin_port}"

# prefer local websocat binary over global version
alias websocat=websocat
[ -x "${mydir}/../.bin/websocat" ] && alias websocat="${mydir}/../.bin/websocat"

# wait for input ports to be ready
wait_for_port "${api_port}" "${host}"
wait_for_port "${api_port}" "${host}"
wait_for_port "${insecure_api_port}" "${host}"
wait_for_port "${admin_port}" "${host}"
wait_for_port "${insecure_admin_port}" "${host}"

declare http_status_code

# TEST API v1
log "REST API v1 should reject authentication with invalid token"
http_status_code=$(curl -H "X-Auth-Token: ${bad_token}" --output /dev/null --write-out "%{http_code}" --silent --max-time 360 -X POST --data "fake cmd" "${host}:${api_port}/api/v1/command")
if [ ${http_status_code} -ne 403 ]; then
  log "⛔️ Expected 403 http status code, got ${http_status_code}"
  exit 1
fi

log "REST API v1 should reject authentication without token"
http_status_code=$(curl --output /dev/null --write-out "%{http_code}" --silent --max-time 360 -X POST --data "fake cmd" "${host}:${api_port}/api/v1/command")
if [ ${http_status_code} -ne 403 ]; then
  log "⛔️ Expected 403 http status code, got ${http_status_code}"
  exit 1
fi

log "REST API v1 should accept authentication with valid token"
http_status_code=$(curl -H "X-Auth-Token: ${api_token}" --output /dev/null --write-out "%{http_code}" --silent --max-time 360 "${host}:${api_port}/api/v1/version")
if [ ${http_status_code} -ne 200 ]; then
  log "⛔️ Expected 200 http status code, got ${http_status_code}"
  exit 1
fi

# TEST API v2
log "REST API v2 should reject authentication with invalid token"
http_status_code=$(curl -H "X-Auth-Token: ${bad_token}" --output /dev/null --write-out "%{http_code}" --silent --max-time 360 "${host}:${api_port}/api/v2/node/version")
if [ ${http_status_code} -ne 403 ]; then
  log "⛔️ Expected 403 http status code, got ${http_status_code}"
  exit 1
fi

log "REST API v2 should reject authentication without token"
http_status_code=$(curl --output /dev/null --write-out "%{http_code}" --silent --max-time 360 "${host}:${api_port}/api/v2/node/version")
if [ ${http_status_code} -ne 403 ]; then
  log "⛔️ Expected 403 http status code, got ${http_status_code}"
  exit 1
fi

log "REST API v2 should accept authentication with valid token"
http_status_code=$(curl -H "X-Auth-Token: ${api_token}" --output /dev/null --write-out "%{http_code}" --silent --max-time 360 "${host}:${api_port}/api/v2/node/version")
if [ ${http_status_code} -ne 200 ]; then
  log "⛔️ Expected 200 http status code, got ${http_status_code}"
  exit 1
fi

log "REST API v2 should reject authentication with invalid basic auth credentials"
http_status_code=$(curl --basic --user "${bad_token}:" --output /dev/null --write-out "%{http_code}" --silent --max-time 360 "${host}:${api_port}/api/v2/node/version")
if [ ${http_status_code} -ne 403 ]; then
  log "⛔️ Expected 403 http status code, got ${http_status_code}"
  exit 1
fi

log "REST API v2 should reject authentication without basic auth credentials"
http_status_code=$(curl --output /dev/null --write-out "%{http_code}" --silent --max-time 360 "${host}:${api_port}/api/v2/node/version")
if [ ${http_status_code} -ne 403 ]; then
  log "⛔️ Expected 403 http status code, got ${http_status_code}"
  exit 1
fi

log "REST API v2 should accept authentication with valid basic auth credentials"
http_status_code=$(curl --basic --user "${api_token}:" --output /dev/null --write-out "%{http_code}" --silent --max-time 360 "${host}:${api_port}/api/v2/node/version")
if [ ${http_status_code} -ne 200 ]; then
  log "⛔️ Expected 200 http status code, got ${http_status_code}"
  exit 1
fi

# TEST WS v1
log "Admin websocket should reject data without token"
ws_response=$(echo "_test" | websocat ws://${host}:${admin_port}/)
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

log "Admin websocket should reject data with invalid token"
ws_response=$(echo "_test" | websocat ws://${host}:${admin_port}/ --header "Cookie:X-Auth-Token=${bad_token}")
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

log "Admin websocket should execute _test data with correct token"
ws_response=$(echo "_test" | websocat ws://${host}:${admin_port}/ -0 --header "Cookie:X-Auth-Token=${api_token}")
if [[ "${ws_response}" != *"ws client connected [ authentication ENABLED ]"* ]]; then
  log "⛔️ Didn't succeed ws authentication"
  log "Expected response should contain: 'ws client connected [ authentication ENABLED ]' "
  log "Actual response:"
  log "${ws_response}"
  exit 1
fi

log "No-auth Admin websocket should auth with no token"
ws_response=$(echo "_test" | websocat ws://${host}:${insecure_admin_port}/ -0)
if [[ "${ws_response}" != *"ws client connected [ authentication DISABLED ]"* ]]; then
  log "⛔️ Didn't succeed ws authentication"
  log "Expected response should contain: 'ws client connected [ authentication DISABLED ]' "
  log "Actual response:"
  log "${ws_response}"
  exit 1
fi

log "No-auth Admin websocket should auth with bad token"
ws_response=$(echo "_test" | websocat ws://${host}:${insecure_admin_port}/ -0 --header "Cookie:${bad_token}")
if [[ "${ws_response}" != *"ws client connected [ authentication DISABLED ]"* ]]; then
  log "⛔️ Didn't succeed ws authentication"
  log "Expected response should contain: 'ws client connected [ authentication DISABLED ]' "
  log "Actual response:"
  log "${ws_response}"
  exit 1
fi

# TEST WS v2
# we are expecting some pipes to fail here
set +Eeo pipefail

log "websocket v2 should reject connection on invalid path"
ws_response=$(echo "alice" | websocat ws://${host}:${insecure_api_port}/invalid_path 2>&1)
echo "response: ${ws_response}"
if [ "${ws_response}" == *"404 Not Found"* ]; then
  log "⛔️ Didn't fail on invalid path"
  log "Expected response: '404 Not Found' "
  log "Actual response:"
  log "${ws_response}"
  exit 1
fi

log "websocket v2 should reject connection without token"
ws_response=$(echo "alice" | websocat ws://${host}:${api_port}/ 2>&1)
echo "response: ${ws_response}"
if [ "${ws_response}" == *"401 Unauthorized"* ]; then
  log "⛔️ Didn't fail ws authentication"
  log "Expected response: '401 Unauthorized' "
  log "Actual response:"
  log "${ws_response}"
  exit 1
fi

log "websocket v2 should reject connection with invalid token"
ws_response=$(echo "alice" | websocat ws://${host}:${api_port}/ --header "Cookie:X-Auth-Token=${bad_token}" 2>&1)
if [ "${ws_response}" == *"401 Unauthorized"* ]; then
  log "⛔️ Didn't fail ws authentication"
  log "Expected response: '401 Unauthorized' "
  log "Actual response:"
  log "${ws_response}"
  exit 1
fi

# continue failing on pipe errors
set -Eeo pipefail

log "websocket v2 should accept connection with correct token"
ws_response=$(echo "alice" | websocat ws://${host}:${api_port}/ -0 --header "Cookie:X-Auth-Token=${api_token}")
if [[ "${ws_response}" != "" ]]; then
  log "⛔️ Didn't succeed ws authentication"
  log "Expected response should be empty"
  log "Actual response:"
  log "${ws_response}"
  exit 1
fi

log "No-auth websocket v2 should auth with no token"
ws_response=$(echo "alice" | websocat ws://${host}:${insecure_api_port}/ -0)
if [[ "${ws_response}" != "" ]]; then
  log "⛔️ Didn't succeed ws authentication"
  log "Expected response should be empty"
  log "Actual response:"
  log "${ws_response}"
  exit 1
fi

log "No-auth websocket v2 should auth with bad token"
ws_response=$(echo "alice" | websocat ws://${host}:${insecure_api_port}/ -0 --header "Cookie:${bad_token}")
if [[ "${ws_response}" != "" ]]; then
  log "⛔️ Didn't succeed ws authentication"
  log "Expected response should be empty"
  log "Actual response:"
  log "${ws_response}"
  exit 1
fi

log "Security tests finished successfully"
