#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="install-jq"
# shellcheck disable=SC1090
source "${mydir}/utils.sh"

declare exit_code=0

which jq > /dev/null || exit_code=$?

if [ "${exit_code}" = "0" ]; then
  log "jq already installed"
  exit 0
fi

declare kernel 

kernel=$(uname -s)

if [ "${kernel}" = "Linux" ]; then
  exit_code=0
  which apt-get > /dev/null || exit_code=$?
  if [ "${exit_code}" != "0" ]; then
    log "⛔️ apt-get not found"
    exit 1
  fi
  sudo apt-get update
  sudo apt-get install jq -y
elif [ "${kernel}" = "Darwin" ]; then
  exit_code=0
  which brew > /dev/null || exit_code=$?
  if [ "${exit_code}" != "0" ]; then
    log "⛔️ Homebrew not found. Please install Homebrew manually first"
    exit 1
  fi
  log "Installing jq..."
  brew install jq
else
  log "⛔️ cannot install jq binary for unsupported platform ${kernel}"
  exit 1
fi

log "Checking jq is installed..."
which jq > /dev/null
log "jq succesfully installed"