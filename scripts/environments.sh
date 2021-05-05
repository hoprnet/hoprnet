#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

source scripts/utils.sh

# These will be cleaned up and machines stopped
# shellcheck disable=SC2034
OLD_RELEASES='zurich zug luzern larnaca queretaro basodino saentis debug-dbg nightly internal'

# ===== Load env variables for the current github ref =====
# Takes:
# - GITHUB_REF
# - RELEASE
# Sets:
# - RELEASE_NAME
# - VERSION_MAJ_MIN
get_environment() {
  BRANCH=${GITHUB_REF#refs/heads} # Removing `refs/heads`

  if [ "$BRANCH" == 'master' ]; then
    RELEASE_NAME='master'
    VERSION_MAJ_MIN='prerelease'
    return
  fi

  case "$BRANCH" in release/*)
    VERSION_MAJ_MIN=$(get_version_maj_min "$RELEASE")

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
      return
    fi

    if [ "$VERSION_MAJ_MIN" == '1.58' ]; then
      RELEASE_NAME='queretaro'
      return
    fi

    if [ "$VERSION_MAJ_MIN" == '1.57' ]; then
      RELEASE_NAME='larnaca'
      return
    fi
    if [ "$VERSION_MAJ_MIN" == '1.56' ]; then
      RELEASE_NAME='luzern'
      return
    fi
    if [ "$VERSION_MAJ_MIN" == '1.55' ]; then
      RELEASE_NAME='zug'
      return
    fi
    if [ "$VERSION_MAJ_MIN" == '1.54' ]; then
      # shellcheck disable=SC2034
      RELEASE_NAME='zurich'
      return
    fi
    echo "Unknown version: $VERSION_MAJ_MIN"
  esac

  echo "Unknown release / environment: '$BRANCH'"
  exit 1
}
