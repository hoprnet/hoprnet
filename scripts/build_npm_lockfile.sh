#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)

source "${mydir}/utils.sh"

usage() {
  msg
  msg "Usage: $0 <pkg>"
  msg
  msg "\t<pkg> must be one of packages under packages/, e.g. 'hoprd'"
  msg
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

if [ -z "${1:-}" ]; then
  msg "Missing package name"
  usage
  exit 1
fi

declare packageName="${1}"

declare build_dir="$(find_tmp_dir)/hopr_build_npm"

log "Creating temporary build directory ${build_dir} (will be removed)"
mkdir "${build_dir}"

declare package_dir="${mydir}/../packages/${packageName}"

log "Install workspace package in temporary directory"

declare package_npm_name="$(jq -r '.name' ${package_dir}/package.json)"
# Resolve workspace links
yarn workspace ${package_npm_name} pack > /dev/null

cd "${build_dir}"

tar -xf ${package_dir}/package.tgz

# Turn yarn `resolutions` from workspace root into npm `overrides` for selected package
jq -s '.[1] * (.[0].resolutions | { overrides: . })' ${mydir}/../package.json ${build_dir}/package/package.json > ${build_dir}/package.json

rm -R ${build_dir}/package

log "Create package-lock.json file for ${packageName}"

npm install --package-lock-only

cp ${build_dir}/package-lock.json ${package_dir}

rm ${build_dir}/package.json

log "Patching resolution overrides in ${package_dir}/package.json"
jq -s '.[1] * (.[0].resolutions | { overrides: . })' ${mydir}/../package.json ${package_dir}/package.json > ${build_dir}/package.json
cp ${build_dir}/package.json ${package_dir}


log "Successfully copied package-lock.json file to ${package_dir}/package.json"

rm -Rf "${build_dir}"
