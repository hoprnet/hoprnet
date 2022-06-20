#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
# shellcheck disable=SC2091
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
# shellcheck disable=SC2034
declare HOPR_LOG_ID="build-lockfiles"
# shellcheck disable=SC1091
source "${mydir}/utils.sh"

usage() {
  msg
  msg "Usage: $0 <pkg>"
  msg
  msg "\t<pkg> must be one of packages under packages/, e.g. 'hoprd'"
  msg
}

# return early with help info when requested
{ [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; } && { usage; exit 0; }

if [ -z "${1:-}" ]; then
  msg "Missing package name"
  usage
  exit 1
fi

declare package_name="${1}"

declare build_dir
build_dir="$(find_tmp_dir)/hopr_lockfile_generation"

log "Creating temporary build directory ${build_dir} (will be removed)"
mkdir "${build_dir}"

declare package_dir="${mydir}/../packages/${package_name}"

restore() {
  log "Restoring previous state"
  rm -Rf "${build_dir}"
  rm -f "${package_dir}/npm-shrinkwrap.json"
  rm -f "${package_dir}/package.tgz"
}

# Remove temporary build dir if failed
trap restore SIGINT SIGTERM ERR

log "Install workspace package in temporary directory"

declare package_npm_name
package_npm_name=$(jq -r '.name' "${package_dir}/package.json")

# Resolve workspace links by packing a NPM package
yarn workspace "${package_npm_name}" pack

cd "${build_dir}"

# Decompress NPM package to get `package.json` file
tar -xf "${package_dir}/package.tgz"

# Turn yarn `resolutions` from workspace root into npm `overrides` for selected package
jq -s '.[1] * (.[0].resolutions | { "overrides": . ,"resolutions": . })' "${mydir}/../package.json" "${build_dir}/package/package.json" > "${build_dir}/package.json"

# # NPM package is now useless
rm "${package_dir}/package.tgz"
rm -R "${build_dir}/package"

log "Creating NPM lock file for ${package_name} package"

# Create NPM lockfile but do not download (or build) entire package
npm install --package-lock-only

# Create shrinkwrap version of package.json
npm shrinkwrap

# Copy also npm-shrinkwrap.json, which will take precedence over package-lock.json while publishing
mv "${build_dir}/npm-shrinkwrap.json" "${package_dir}"

# No need for package.json anymore
rm "${build_dir}/package.json"

log "Patching resolution overrides in ${package_dir}/package.json for Yarn and NPM"

# Create package.json file without resolved versions version but with resolution overrides
jq -s '.[1] * (.[0].resolutions | { "overrides": . ,"resolutions": . })' "${mydir}/../package.json" "${package_dir}/package.json" > "${build_dir}/package.json"
cp "${build_dir}/package.json" "${package_dir}"

log "Successfully copied lockfile to ${package_dir}/"

log "Removing temporary build dir"
rm -Rf "${build_dir}"
