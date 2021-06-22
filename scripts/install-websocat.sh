#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="e2e-setup"
source "${mydir}/../scripts/utils.sh"

declare exit_code=0

websocat --version > /dev/null || exit_code=$?

if [ ${exit_code} -ne 0 ]; then
    log "websocat binary not found, trying to install Linux version"
    sudo apt-get update 
    sudo apt-get install curl -y 
    curl -sLO https://github.com/vi/websocat/releases/download/v1.8.0/websocat_1.8.0_newer_amd64.deb 
    sudo dpkg -i websocat_1.8.0_newer_amd64.deb
else
    log "websocat found"
fi

