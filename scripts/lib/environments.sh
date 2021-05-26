#!/usr/bin/env bash

# prevent execution of this script, only allow sourcing
$(return >/dev/null 2>&1)
test "$?" -eq "0" || { echo "This script should only be sourced."; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -euo pipefail

# don't source this file twice
test -z "${ENVIRONMENTS_SOURCED:-}" && ENVIRONMENTS_SOURCED=1 || exit 0

# set log id and use shared log function for readable logs
declare HOPR_LOG_ID="env"
declare mydir
mydir=$(dirname $(readlink -f $0))
source "${mydir}/utils.sh"

# These will be cleaned up and machines stopped
OLD_RELEASES='zurich zug luzern larnaca queretaro basodino saentis debug-dbg nightly internal integration-test'

# ===== Load env variables for the current github ref =====
# Takes:
# - GITHUB_REF
# - RELEASE
# Sets:
# - RELEASE_NAME
# - RELEASE_IP deprecated
# - VERSION_MAJ_MIN
get_environment() {
  BRANCH=$(echo "$GITHUB_REF" | sed -e "s#refs/heads/##g") # Removing `refs/heads`

  if [[ "$BRANCH" == 'master' ]] || [[ "$BRANCH" == debug-deploy/* ]]; then
    RELEASE_NAME='master'
    RELEASE_IP='34.65.102.152'
    VERSION_MAJ_MIN='prerelease'
    return
  fi

  case "$BRANCH" in release/*)
    VERSION_MAJ_MIN=$(get_version_maj_min $RELEASE)

    if [ "$VERSION_MAJ_MIN" == '1.70' ]; then
      RELEASE_NAME='jungfrau'
      return
    fi
    if [ "$VERSION_MAJ_MIN" == '1.69' ]; then
      RELEASE_NAME='bienne'
      return
    fi
    if [ "$VERSION_MAJ_MIN" == '1.68' ]; then
      RELEASE_NAME='neuchatel'
      return
    fi
    if [ "$VERSION_MAJ_MIN" == '1.67' ]; then
      RELEASE_NAME='emmenbrucke'
      return
    fi
    if [ "$VERSION_MAJ_MIN" == '1.64' ]; then
      RELEASE_NAME='morat'
      return
    fi
    if [ "$VERSION_MAJ_MIN" == '1.62' ]; then
      RELEASE_NAME='titlis'
      return
    fi
    if [ "$VERSION_MAJ_MIN" == '1.61' ]; then
      RELEASE_NAME='nyc'
      return
    fi
    if [ "$VERSION_MAJ_MIN" == '1.60' ]; then
      RELEASE_NAME='mainz'
      # Released by mistake: https://github.com/hoprnet/hoprnet/pull/893#issuecomment-750318579
      return
    fi
    if [ "$VERSION_MAJ_MIN" == '1.59' ]; then
      RELEASE_NAME='mainz'
      # From this release on, RELEASE_IP is deprecated
      return
    fi

    if [ "$VERSION_MAJ_MIN" == '1.58' ]; then
      RELEASE_NAME='queretaro'
      RELEASE_IP='34.65.207.39'
      return
    fi

    if [ "$VERSION_MAJ_MIN" == '1.57' ]; then
      RELEASE_NAME='larnaca'
      RELEASE_IP='unknown'
      return
    fi
    if [ "$VERSION_MAJ_MIN" == '1.56' ]; then
      RELEASE_NAME='luzern'
      RELEASE_IP='34.65.5.42'
      return
    fi
    if [ "$VERSION_MAJ_MIN" == '1.55' ]; then
      RELEASE_NAME='zug'
      RELEASE_IP='34.65.158.118'
      return
    fi
    if [ "$VERSION_MAJ_MIN" == '1.54' ]; then
      RELEASE_NAME='zurich'
      RELEASE_IP='unknown'
      return
    fi
    log "Unknown version: $VERSION_MAJ_MIN"
  esac

  log "Unknown release / environment: '$BRANCH'"
  exit 1
  RELEASE_NAME='debug'
  RELEASE_IP='34.65.56.229'
  VERSION_MAJ_MIN='dbg'
}
