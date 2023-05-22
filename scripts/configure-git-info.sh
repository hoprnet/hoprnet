#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

if [ "${CI:-}" = "true" ] && [ -z "${ACT:-}" ]; then
  git config user.email "tech@hoprnet.org"
  git config user.name "HOPR CI robot"
  git config pull.rebase false
fi
