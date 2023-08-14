#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="notify-matrix-github-workflow-failure"
# shellcheck disable=SC1090
source "${mydir}/utils.sh"

usage() {
  msg
  msg "Usage: $0 <room> <repo> <workflow> <run_id>"
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
[ -z "${2:-}" ] && { msg "Parameter <repo> required"; usage; exit 1; }
[ -z "${3:-}" ] && { msg "Parameter <workflow> required"; usage; exit 1; }
[ -z "${4:-}" ] && { msg "Parameter <run_id> required"; usage; exit 1; }

declare room="${1}"
declare repo="${2}"
declare workflow="${3}"
declare run_id="${4}"

declare url="https://github.com/${repo}/actions/runs/${run_id}"
declare branch
branch=$(git rev-parse --abbrev-ref HEAD)
declare msg="Github workflow ${workflow} failed on branch ${branch}, see ${url}"

"${mydir}"/notify-matrix.sh "${room}" "${msg}"
