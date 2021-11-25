#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return > /dev/null 2>&1)
test "$?" -eq "0" && {
  echo "This script should only be executed." >&2
  exit 1
}

declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &> /dev/null && pwd -P)

# set log id and use shared log function for readable logs
declare HOPR_LOG_ID="hopr-connect-test"

# Setup environment and load useful utils
source "${mydir}/../utils.sh"

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

usage() {
  msg
  msg "Usage: $0"
  msg
}

# here go the tests
"${mydir}/reconnect-test.sh"
"${mydir}/relay-slots-test.sh"
