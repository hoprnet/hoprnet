#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="install-jq"
source "${mydir}/utils.sh"

declare exit_code=0

which jq > /dev/null || exit_code=$?

if [ "${exit_code}" = "0" ]; then
  log "jq binary found"
  exit 0
fi

apt-get update
apt-get intall jq -y