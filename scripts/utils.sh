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
# $2 = host to check port on
# $3 = optional: file to tail for debug info
# $4 = optional: delay between checks in seconds, defaults to 2s
# $5 = optional: max number of checks, defaults to 1000
function wait_for_http_port() {
  local port=${1}
  local host=${2}
  local log_file=${3:-}
  local delay=${4:-2}
  local max_wait=${5:-1000}
  local cmd="curl --silent "${host}:${port}""

  wait_for_port "${port}" "${host}" "${log_file}" "${delay}" "${max_wait}" "${cmd}"
}

# $1 = port to wait for
# $2 = optional: host to check port on, defaults to 127.0.0.1
# $3 = optional: file to tail for debug info
# $4 = optional: delay between checks in seconds, defaults to 2s
# $5 = optional: max number of checks, defaults to 1000
# $6 = optional: command to check
function wait_for_port() {
  local port=${1}
  local host=${2:-127.0.0.1}
  local log_file=${3:-}
  local delay=${4:-10}
  local max_wait=${5:-1000}
  # by default we do a basic listen check
  local cmd=${6:-nc -z -w 1 ${host} ${port}}

  i=0
  until ${cmd}; do
    log "Waiting ${delay} seconds for port to be reachable ${host}:${port}"
    if [ -s "${log_file}" ]; then
      log "Last 5 logs from ${log_file}:"
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

# $1 command to execute
# $2 optional: number of retries, defaults to 0
# $3 optional: seconds between retries, defaults to 1
# $4 optional: print command before execution, defaults to false
try_cmd() {
  local cmd="${1}"
  local retries_left=${2:-0}
  local wait_in_sec="${3:-1}"
  local verbose="${4:-false}"
  local cmd_exit_code result

  if [ "${verbose}" = "true" ]; then
    log "Executing command: ${cmd}"
  fi

  if [ ${retries_left} -le 0 ]; then
    # no retries left, so we just execute the command as is
    eval ${cmd}
  else
    # the output needs to be captured to not mess up the return result
    # also exit on error needs to be disabled for execution of the command and re-enabled afterwards again
    local output_file=$(mktemp -q)
    set +Eeo pipefail
    if ${cmd} > ${output_file}; then
      # command succeeded, return the output
      set -Eeo pipefail
      local result
      result=$(cat ${output_file})
      rm -f ${output_file}
      echo ${result}
    else
      # command failed, need to retry
      set -Eeo pipefail
      rm -f ${output_file}
      ((retries_left--))
      if [ ${wait_in_sec} > 0 ]; then
        sleep ${wait_in_sec}
      fi
      log "Retrying command ${retries_left} more time(s)"
      try_cmd "${cmd}" ${retries_left} ${wait_in_sec} ${verbose}
    fi
  fi
}

# $1 = file to monitor
# $2 = regexp to look for
# $3 = delay, defaults to 1.0 (seconds)
function wait_for_regex {
  local file="${1}"
  local regex="${2}"
  local delay="${delay:-1.0}"

  while true; do
    if [ -f ${file} ]; then
      local res=$(grep -E "${regex}" "${file}" || echo "")
      if [[ "${res}" != "" ]]; then
        echo "${res}"
        return 0
      fi
    fi
    sleep ${delay}
  done
}

setup_colors
