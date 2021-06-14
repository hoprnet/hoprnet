#!/bin/bash
set -e #u

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
    echo "Port is not free $1"
    echo "Process: $(lsof -i ":${port}" -s TCP:LISTEN || :)"
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
    echo "Waiting (${delay}) seconds for port ${port}"
    if [ -n "${log_file}" ] && [ -f "${log_file}" ]; then
      echo "Last 5 logs:"
      tail -n 5 "${log_file}"
    fi
    sleep ${delay}
    ((i=i+1))
    if [ $i -gt ${max_wait} ]; then
      exit 1
    fi
  done
}
