#!/usr/bin/env bash

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

usage() {
  echo ""
  echo "Usage: $0 <release_type[Build|ReleaseCandidate|Patch|Minor|Major]> <current_version> <BuildNumber>"
  echo ""
  echo "$0 Build 2.0.0-rc.2 1234"
  echo "$0 ReleaseCandidate 2.0.0-rc.2"
  echo "$0 Patch 2.0.0-rc.2"
  echo "$0 Minor 2.0.0-rc.2"
  echo "$0 Major 2.0.0-rc.2"
  echo
}

# return early with help info when requested
{ [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; } && { usage; exit 0; }

# set mydir
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)


release_type=${1} # Can be any of these values Build | ReleaseCandidate | Patch | Minor | Major
build=${2:-}
current_version=$(jq -r '.version' "${mydir}/../packages/hoprd/package.json")

# Set dash as the delimiter to read current_version to get release candidate
IFS='-'
read -a current_version_splitted <<< "$current_version"

current_version_prefix=${current_version_splitted[0]}
if [ "${#current_version_splitted[*]}" == "2" ] 
then
  # Get Release Candidate Number
  pre_release=${current_version_splitted[1]/rc\.}
elif [ "${release_type}" == "ReleaseCandidate" ]
then
  # Reset the release candidate for new release name
  pre_release=0
fi

# Set dot as the delimiter to read parts of the version format
IFS='.'
read -a current_version_splitted <<< "$current_version_prefix"
major_version=${current_version_splitted[0]}
minor_version=${current_version_splitted[1]}
patch_version=${current_version_splitted[2]}

# Unset delimiter
unset IFS

# Increase version according to Release Type
case "$release_type" in
  ReleaseCandidate)
    pre_release=$((pre_release+1))
    ;;
  Patch)
    if [ -z ${pre_release+x} ]; then
      patch_version=$((patch_version+1))
    fi
    unset pre_release
    ;;
  Minor)
    patch_version=0
    minor_version=$((minor_version+1))
    ;;
  Major)
    patch_version=0
    minor_version=0
    major_version=$((major_version+1))
    ;;
esac

# Sets default version
new_version=${major_version}.${minor_version}.${patch_version}

# Adds release candidate information if needed
if [ ! -z ${pre_release+x} ]
then
  new_version=${new_version}-rc.${pre_release}
fi

# Adds Build information if needed
if [ "$release_type" == "Build" ]
then
    new_version=${new_version}+pr.${build}
fi

# Print results
echo $new_version