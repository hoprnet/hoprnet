#!/usr/bin/env bash

# prevent execution of this script, only allow sourcing
$(return >/dev/null 2>&1)
test "$?" -eq "0" || { echo "This script should only be sourced."; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -euo pipefail

# don't source this file twice
test -z "${UTILS_SOURCED:-}" && UTILS_SOURCED=1 || exit 0

# $1=version string, semver
function get_version_maj_min() {
  get_version_maj_min_pat $1 | cut -d. -f1,2
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

# shared log function which adds a useful prefix to all messages
# $1=msg
function log() {
  local prefix="HOPR-SCRIPT"
  if [ -n "${HOPR_LOG_ID:-}" ]; then
    prefix="${prefix}:${HOPR_LOG_ID}"
  fi
  echo -e "[${prefix}] ${1:-}"
}
