#!/usr/bin/env bash

# prevent execution of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" || { echo "This script should only be sourced." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# $1=version string, semver
function get_version_maj_min() {
  echo $(get_version_maj_min_pat $1 | cut -d. -f1,2)
}

# $1=version string, semver
function get_version_maj_min_pat() {
  # From https://github.com/cloudflare/semver_bash/blob/master/semver.sh
  # Fixed https://github.com/cloudflare/semver_bash/issues/4
  local RE='[^0-9]*\([0-9]*\)[.]\([0-9]*\)[.]\([0-9]*\)\([0-9A-Za-z-]*\).*'
  local MAJ=$(echo "$1" | sed -e "s#$RE#\1#")
  local MIN=$(echo "$1" | sed -e "s#$RE#\2#")
  local PAT=$(echo "$1" | sed -e "s#$RE#\3#")
  echo "$MAJ.$MIN.$PAT"
}

# $1 = port
function ensure_port_is_free() {
  local port=${1}

  if lsof -i ":${port}" -s TCP:LISTEN; then
    log "Port is not free $1"
    log "Process: $(lsof -i ":${port}" -s TCP:LISTEN || :)"
    exit 1
  fi
}

# $1 = port to wait for
# $2 = optional: file to tail for debug info
# $3 = optional: delay between checks in seconds, defaults to 2s
# $4 = optional: max number of checks, defaults to 1000
function wait_for_http_port() {
  local port=${1}
  local log_file=${2:-}
  local delay=${3:-2}
  local max_wait=${4:-1000}
  local cmd="curl --silent "localhost:${port}""

  wait_for_port "${port}" "${log_file}" "${delay}" "${max_wait}" "${cmd}"
}

# $1 = port to wait for
# $2 = optional: file to tail for debug info
# $3 = optional: delay between checks in seconds, defaults to 2s
# $4 = optional: max number of checks, defaults to 1000
# $5 = optional: command to check
function wait_for_port() {
  local port=${1}
  local log_file=${2:-}
  local delay=${3:-2}
  local max_wait=${4:-1000}
  # by default we do a basic listen check
  local cmd=${5:-lsof -i ":${port}" -s TCP:LISTEN}

  i=0
  until ${cmd}; do
    log "Waiting (${delay}) seconds for port ${port}"
    if [ -s "${log_file}" ]; then
      log "Last 5 logs:"
      tail -n 5 "${log_file}" | sed "s/^/\\t/"
    fi
    sleep ${delay}
    ((i=i+1))
    if [ $i -gt ${max_wait} ]; then
      exit 1
    fi
  done
}

setup_colors() {
  if [[ -t 2 ]] && [[ -z "${NO_COLOR:-}" ]] && [[ "${TERM:-}" != "dumb" ]]; then
    NOFORMAT='\033[0m' RED='\033[0;31m' GREEN='\033[0;32m' ORANGE='\033[0;33m'
    BLUE='\033[0;34m' PURPLE='\033[0;35m' CYAN='\033[0;36m'
    YELLOW='\033[1;33m'
  else
    NOFORMAT='' RED='' GREEN='' ORANGE='' BLUE='' PURPLE='' CYAN=''
    YELLOW=''
  fi
}

log() {
  local time
  # second-precision is enough
  time=$(date -u +%y-%m-%dT%H:%M:%SZ)
  echo >&2 -e "$CYAN${time} [${HOPR_LOG_ID:-}]$NOFORMAT ${1-}"
}

msg() {
  echo >&2 -e "${1-}"
}

setup_colors
