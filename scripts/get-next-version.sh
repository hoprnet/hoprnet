#!/usr/bin/env bash

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

usage() {
  echo ""
  echo "Usage: $0 <release_type[Build|ReleaseCandidate|Patch|Minor|Major]> <build_number>"
  echo ""
  echo "$0 Build 1234"
  echo "$0 ReleaseCandidate"
  echo "$0 Patch"
  echo "$0 Minor"
  echo "$0 Major"
  echo
}

# return early with help info when requested
{ [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; } && { usage; exit 0; }

# set mydir
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)

release_type="${1:-}"
build="${2:-}"

if [ -z "${release_type:-}" ]; then
  echo "Error: Mandatory parameter <release_type> missing"
  usage
  exit 1
fi

if [[ ! "${release_type}" =~ (Build|ReleaseCandidate|Patch|Minor|Major) ]]; then
  echo "Error: Parameter <release_type> contains unsupported value"
  usage
  exit 1
fi

if [ "${release_type}" = "Build" ] && [ -z "${build}" ]; then
  echo "Error: Parameter <build_number> missing"
  usage
  exit 1
fi

current_version="$(jq -r '.version' "${mydir}/../packages/hoprd/package.json")"

# Set dash as the delimiter to read current_version to get release candidate
IFS='-'
read -a current_version_splitted <<< "${current_version}"

current_version_prefix="${current_version_splitted[0]}"
if [ "${#current_version_splitted[*]}" = "2" ]; then
  # Get Release Candidate Number
  pre_release="${current_version_splitted[1]/rc\.}"
elif [ "${release_type}" = "ReleaseCandidate" ]; then
  # Reset the release candidate for new release name
  pre_release=0
fi

# Set dot as the delimiter to read parts of the version format
IFS='.'
read -a current_version_splitted <<< "${current_version_prefix}"
major_version="${current_version_splitted[0]}"
minor_version="${current_version_splitted[1]}"
patch_version="${current_version_splitted[2]}"

# Unset delimiter
unset IFS

# Increase version according to Release Type
case "${release_type}" in
  ReleaseCandidate)
    pre_release=$((pre_release+1))
    ;;
  Patch)
    if [ -z ${pre_release} ]; then
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
new_version="${major_version}.${minor_version}.${patch_version}"

# Adds release candidate information if needed
if [ -n "${pre_release:-}" ]; then
  new_version="${new_version}-rc.${pre_release}"
fi

# Adds Build information if needed
if [ "${release_type}" = "Build" ]; then
    new_version="${new_version}+pr.${build}"
fi

# Print results
echo "${new_version}"
