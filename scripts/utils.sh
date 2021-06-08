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

  if lsof -i ":${port}" | grep -q 'LISTEN' && true || false; then
    echo "Port is not free $1"
    echo "Process: $(lsof -i ":${port}" | grep 'LISTEN' || :)"
    exit 1
  fi
}

# $1 = port to wait for
# $2 = optional: delay between checks in seconds, defaults to 2s
# $3 = optional: max number of checks, defaults to 1000
# $4 = optional: file to tail for debug info
function wait_for_http_port() {
  local port=${1}
  local delay=${2:-2}
  local max_wait=${3:-1000}
  local log_file=${4:-}

  i=0
  until curl --silent "localhost:${port}"; do
    echo "Waiting (${delay}) seconds for port ${port}"
    if [ -n "${log_file}" ]; then
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
