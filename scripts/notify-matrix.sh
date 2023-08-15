#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="notify-matrix"
# shellcheck disable=SC1090
source "${mydir}/utils.sh"

usage() {
  msg
  msg "Usage: $0 <room> <message>"
  msg
  msg "The following environment variables are used to perform the request:"
  msg
  msg "MATRIX_SERVER, default 'https://matrix.org'"
  msg "MATRIX_ACCESS_TOKEN, default ''"
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

# do work

[ -z "${1:-}" ] && { msg "Parameter <room> required"; usage; exit 1; }
[ -z "${2:-}" ] && { msg "Parameter <message> required"; usage; exit 1; }
which curl > /dev/null || { msg "Required binary 'curl' not found in PATH"; exit 1; }

declare room="${1}"
declare msg="${2}"
declare server="${MATRIX_SERVER:-https://matrix.org}"
declare token="${MATRIX_ACCESS_TOKEN:-}"
declare event_id="$(date -u +%y%m%d%H%M%S)${RANDOM}"
declare url="${server}/_matrix/client/r0/rooms/%21${room}/send/m.room.message/${event_id}"

# escape tabs in message, and wrap in div tag
msg="<div>$(echo "${msg}" | sed 's/\t/\\\\t/g')</div>"

curl -s -X PUT \
  -H "Authorization: Bearer ${token}" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
  -d "{\"msgtype\": \"m.notice\", \"body\": \"${msg}\", \"formatted_body\": \"${msg}\", \"format\": \"org.matrix.custom.html\" }" \
  "${url}"
