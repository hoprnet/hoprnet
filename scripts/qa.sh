#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

# Don't source this file twice
test -z "${QA_SOURCED:-}" && QA_SOURCED=1 || exit 0

# Smoke test a running node

#$1 command
#$2 admin host
cmd() {
  curl --data "$1" "$2/api/v1/command"
  echo ''
}

HOST=localhost:3001

# Info
cmd 'version' "$HOST"
cmd 'settings' "$HOST"
cmd 'info' "$HOST"
cmd 'balance' "$HOST"
cmd 'address' "$HOST"
cmd 'channels' "$HOST"
cmd 'peers' "$HOST"

# Open channel
