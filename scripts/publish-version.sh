#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="publish-version"
source "${mydir}/utils.sh"

usage() {
  msg
  msg "Usage: $0 <version_type>"
  msg
  msg "\t<version_type> must be one of patch or prerelease"
  msg
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

# verify and set parameters
declare branch version_type

# We don't try to be smart about the version type to be published.
# The user must provide that.
version_type="${1:-}"
if [ ! "${version_type}" = "patch" ] &&
   [ ! "${version_type}" = "prerelease" ]; then
  usage
  exit 1
fi

# ensure local copy is up-to-date with origin
branch=$(git rev-parse --abbrev-ref HEAD)
git pull origin "${branch}" --rebase

# get package info
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)

# ensure the build is up-to-date
yarn
yarn build

declare current_version
current_version=$(${mydir}/get-package-version.sh)
if [[ "${version_type}" = "prerelease" ]]; then
  # turn prerelease into preminor if the current version is not a prerelease
  # already, thus update from a patch version to a new minor prerelease
  if [[ "${current_version}" != *"-next."* ]]; then
    version_type="preminor"
  fi
fi

echo "Creating new version ${current_version} + ${version_type}"

# create new version in each package
yarn workspaces foreach -piv --no-private --topological-dev exec -- npm version ${version_type} --preid=next
declare new_version
new_version=$(${mydir}/get-package-version.sh)

# commit changes and create Git tag
git add packages/*/package.json
git commit -m "chore(release): publish ${new_version}"
git tag v${new_version}

# only make remote changes if running in CI
if [ "${CI:-}" = "true" ] && [ -z "${ACT:-}" ]; then
  # push changes back onto origin including new tag
  git push origin "${branch}" --tags

  # publish each workspace package to npm
  if [ -n "${NODE_AUTH_TOKEN:-}" ]; then
    yarn config set npmAuthToken "${NODE_AUTH_TOKEN:-}"
  fi
  yarn workspaces foreach -piv --no-private --topological-dev npm publish --access public
fi
