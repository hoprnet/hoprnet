#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

# $1=version string, semver
function get_version_maj_min() {
  get_version_maj_min_pat "$1" | cut -d. -f1,2
}

# $1=version string, semver
function get_version_maj_min_pat() {
  # From https://github.com/cloudflare/semver_bash/blob/master/semver.sh
  # Fixed https://github.com/cloudflare/semver_bash/issues/4
  local RE='[^0-9]*\([0-9]*\)[.]\([0-9]*\)[.]\([0-9]*\)\([0-9A-Za-z-]*\).*'
  local MAJ
  local MIN
  local PAT
  # shellcheck disable=SC2001
  MAJ=$(echo "$1" | sed -e "s#$RE#\1#")
  # shellcheck disable=SC2001
  MIN=$(echo "$1" | sed -e "s#$RE#\2#")
  # shellcheck disable=SC2001
  PAT=$(echo "$1" | sed -e "s#$RE#\3#")
  echo "$MAJ.$MIN.$PAT"
}
