#!/usr/bin/env bash

# prevent execution of this script, only allow sourcing
$(return >/dev/null 2>&1)
test "$?" -eq "0" || { echo "This script should only be sourced." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# $1 = port
function ensure_port_is_free() {
  local port=${1}

  if lsof -i ":${port}" -s TCP:LISTEN; then
    log "Port is not free $1"
    log "Process: $(lsof -t -i ":${port}" -s TCP:LISTEN || :)"
    exit 1
  fi
}

# $1 = filename
# $2 = expected content
function expect_file_content {
  local filename="${1}"
  local expected="${2}"
  local actual="$(cat "${filename}")"

  if [[ "${expected}" != "${actual}" ]]; then
    log "⛔️ bad content for ${filename}"
    log "expected: "
    log "${expected}"
    log "actual: "
    log "${actual}"
    exit 1
  fi
}

# $1 = file to monitor
# $2 = regexp to look for
function wait_for_regex_in_file() {    
    local file=${1}
    local regex=${2}    
 
    log "Waiting for ${regex} in ${file}..."    
    
    local delay=0.1
    
    while true; do
      sleep ${delay}
      local res=$(grep -E "${regex}" "${file}" || echo "")
      if [[ "${res}" != "" ]]; then
        return 0
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
