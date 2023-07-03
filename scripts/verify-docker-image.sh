#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
# shellcheck disable=SC2091
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
# shellcheck disable=SC2034
declare HOPR_LOG_ID="verify-docker-image"
# shellcheck disable=SC1091
source "${mydir}/utils.sh"

usage() {
  msg
  msg "Usage: $0 <image_name> <image_version> <package_version>"
  msg
  msg "\t<image_name> is the Docker image name which should be run"
  msg "\t<image_version> is the Docker image tag which should be run"
  msg "\t<package_version> is the NPM package version which is verified"
  msg
}

# return early with help info when requested
{ [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; } && { usage; exit 0; }

# verify and set parameters
declare image_version package_version image_name

image_name="${1?"parameter <image_name> missing"}"
image_version="${2?"parameter <image_version> missing"}"
package_version="${3?"parameter <package_version> missing"}"

image="gcr.io/hoprassociation/${image_name}:${image_version}"

log "Verifying docker image ${image} has bundled package version ${package_version}"

declare version
version="$(docker run -v /var/run/docker.sock:/var/run/docker.sock "${image}" --version | sed -En '/^[0-9]+\.[0-9]+\.[0-9]+(-next\.[0-9]+)?$/p')"

# The version returned by hoprd has the following format: hoprd v1.2.3
# Any suffix after the patch version is omitted.
# Therefore, we check that the package version starts with the reported version
# as a prefix.

if [[ "${version#hoprd v}*" = "${package_version}" ]]; then
  log "Docker image ${image} has bundled package version ${version}, expected ${package_version}"
  exit 1
fi

log "Docker image ${image} has bundled correct package version ${version}"
