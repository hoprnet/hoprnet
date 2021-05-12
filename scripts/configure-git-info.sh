#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

if [ -n "${HOPR_IN_CI:-}" ]; then
  git config user.email "noreply@hoprnet.org"
  git config user.name "HOPR CI robot"
  git config pull.rebase false
fi
