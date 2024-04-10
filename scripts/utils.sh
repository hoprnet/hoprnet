#!/usr/bin/env bash

# prevent execution of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" || { echo "This script should only be sourced." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# $1=version string, semver
function get_version_maj_min() {
  echo $(get_version_maj_min_pat "$1" | cut -d. -f1,2)
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
  local port
  port="${1}"

  if lsof -i ":${port}" -s TCP:LISTEN; then
    log "Port is not free $1"
    log "Process: $(lsof -i ":${port}" -s TCP:LISTEN || :)"
    exit 1
  fi
}

setup_colors() {
  if [[ -t 2 ]] && [[ -z "${NO_COLOR:-}" ]] && [[ "${TERM:-}" != "dumb" ]]; then
    # shellcheck disable=SC2034
    NOFORMAT='\033[0m' RED='\033[0;31m' GREEN='\033[0;32m' ORANGE='\033[0;33m'
    # shellcheck disable=SC2034
    BLUE='\033[0;34m' PURPLE='\033[0;35m' CYAN='\033[0;36m'
    # shellcheck disable=SC2034
    YELLOW='\033[1;33m'
  else
    # shellcheck disable=SC2034
    NOFORMAT='' RED='' GREEN='' ORANGE='' BLUE='' PURPLE='' CYAN=''
    # shellcheck disable=SC2034
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
try_cmd() {
  local cmd retries_left wait_in_sec result output_file

  cmd="${1}"
  retries_left=${2:-0}
  wait_in_sec="${3:-1}"

  if [ "${HOPR_VERBOSE:-false}" = "true" ]; then
    log "Executing command: ${cmd}"
  fi

  if [ "${retries_left}" -le 0 ]; then
    # no retries left, so we just execute the command as is
    # shellcheck disable=SC2086
    eval ${cmd}
  else
    # the output needs to be captured to not mess up the return result
    # also exit on error needs to be disabled for execution of the command and re-enabled afterwards again
    output_file="$(mktemp -q)"
    rm -f "${output_file}"
    set +Eeo pipefail
    if eval "${cmd}" > "${output_file}"; then
      # command succeeded, return the output
      set -Eeo pipefail
      result="$(cat "${output_file}")"
      rm -f "${output_file}"
      echo "${result}"
    else
      # command failed, need to retry
      set -Eeo pipefail
      rm -f "${output_file}"
      ((retries_left--))
      if [ "${wait_in_sec}" -gt 0 ]; then
        sleep "${wait_in_sec}"
      fi
      log "Retrying command ${retries_left} more time(s)"
      try_cmd "${cmd}" ${retries_left} "${wait_in_sec}"
    fi
  fi
}

# $1 = file to monitor
# $2 = regexp to look for
# $3 = delay, defaults to 1.0 (seconds)
function wait_for_regex {
  local file regex delay res

  file="${1}"
  regex="${2}"
  delay="${delay:-1.0}"

  while true; do
    if [ -f "${file}" ]; then
      res=$(grep -E "${regex}" "${file}" || echo "")
      if [ -n "${res}" ]; then
        break
      fi
    fi
    sleep "${delay}"
  done

  echo "${res}"
}

function find_tmp_dir() {
  local tmp

  tmp="/tmp"

  if [ ! -d "${tmp}" ] || [ ! -w "${tmp}" ]; then
    tmp="/var/tmp"
  fi

  if [ ! -d "${tmp}" ] || [ ! -w "${tmp}" ]; then
    msg "Neither /tmp or /var/tmp can be used for writing logs";
    exit 1;
  fi

  echo ${tmp}
}

# encode API token
# $1 = api token
encode_api_token(){
  local api_token

  api_token=${1}

  # ideally we would use base64's option -w but it's not available in all envs
  echo -n "$api_token" | base64 | tr -d \\n
}

# $1 = optional: endpoint, defaults to http://localhost:3001
get_native_address(){
  local endpoint url cmd

  endpoint="${1:-localhost:3001}"
  url="${endpoint}/api/v3/account/addresses"
  cmd="$(get_authenticated_curl_cmd "${url}")"

  try_cmd "${cmd}" 30 5 | jq -r ".native"
}

# $1 = optional: endpoint, defaults to http://localhost:3001
get_hopr_address() {
  local endpoint url cmd

  endpoint="${1:-localhost:3001}"
  url="${endpoint}/api/v3/account/addresses"
  cmd="$(get_authenticated_curl_cmd "${url}")"

  try_cmd "${cmd}" 30 5 | jq -r ".hopr"
}

# $1 = endpoint
# $2 = api token
validate_hopr_address() {
  local hopr_address endpoint api_token valid

  endpoint="${1}"
  api_token="${2}"

  hopr_address="$(get_hopr_address "${api_token}@${endpoint}")"
  if [[ -z "${hopr_address}" ]]; then
    log "-- could not derive hopr address from endpoint ${endpoint}"
    exit 1
  fi

  valid="$(node -e "(import('@hoprnet/hopr-utils')).then(pId => console.log(pId.hasB58String('${hopr_address}')))")"

  if ! [[ $valid == "true" ]]; then
    log "Node returns an invalid hopr address: ${hopr_address} derived from endpoint ${endpoint}"
    exit 1
  fi

  echo "valid hopr address: ${hopr_address}"
}

# $1 = endpoint
# $2 = api token
validate_native_address() {
  local native_address endpoint api_token

  endpoint="${1}"
  api_token="${2}"

  native_address="$(get_native_address "${api_token}@${endpoint}")"
  if [ -z "${native_address}" ]; then
    log "-- could not derive native address from endpoint ${endpoint}"
    exit 1
  fi

  if [ -z "$(echo "${native_address}" | sed -nE "/0x[0-9a-fA-F]{40}/p")" ]; then
    log "Node returns an invalid native address: ${native_address} derived from endpoint ${endpoint}"
    exit 1
  fi

  echo "valid native address: ${native_address}"
}

# $1 = endpoint
get_authenticated_curl_cmd() {
  # the following checks must handle endpoints like:
  #   - myendpoint.com
  #   - myendpoint.com:3001
  #   - apitoken@myendpoint.com
  #   - apitoken@myendpoint.com:3001
  #   - http(s)://myendpoint.com
  #   - http(s)://myendpoint.com:3001
  #   - http(s)://apitoken@myendpoint.com
  #   - http(s)://apitoken@myendpoint.com:3001

  # trim whitespaces which are not allowed anywhere in the url
  local full_endpoint="${1// /}"

  # set default protocol if none was found
  local protocol="http://"
  # extract protocol prefix incl. separator ://
  if [[ "${full_endpoint}" =~ "://" ]]; then
    protocol="$(echo "${full_endpoint}" | sed -e's,^\(.*://\).*,\1,g')"
  fi

  # remove protocol from endpoint
  local endpoint_wo_protocol="${full_endpoint#"$protocol"}"

  # extract host:port/url portion of endpoint
  local host_w_port="${endpoint_wo_protocol#*@}"

  # extract auth portion of endpoint
  local api_token="${endpoint_wo_protocol%@*}"

  # re-create endpoint with correct protocol
  local endpoint="${protocol}${host_w_port}"

  # set up base curl command
  local cmd="curl --silent --max-time 5 ${endpoint}"

  # add auth info if token was found previously
  if [ -n "${api_token}" ]; then
    cmd+=" --header \"X-Auth-Token: ${api_token}\""
  fi

  # return full command
  echo "${cmd}"
}

# $1 - target file
# $2 - source file
# $2 - source_network network name of source e.g. anvil-localhost
# $3 - destination_network network name of destination e.g. anvil-localhost2
update_protocol_config_addresses() {
  local target_file source_file source_network destination_network source_data

  target_file="${1}"
  source_file="${2}"
  source_network="${3}"
  destination_network="${4}"

  log "updating contract addresses in protocol configuration"

  # copy all the fields except for the `stake_season`
  source_data="$(jq -r ".networks.\"${source_network}\"" "${source_file}" | jq "{environment_type: .environment_type, indexer_start_block_number: .indexer_start_block_number, addresses: .addresses}")"
  jq --argjson inputdata "${source_data}" ".networks.\"${destination_network}\" += \$inputdata" "${target_file}" > "${target_file}.new"
  mv "${target_file}.new" "${target_file}"

  log "contract addresses are updated in protocol configuration"
}

get_eth_block_number() {
}

setup_colors
