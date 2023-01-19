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
declare HOPR_LOG_ID="publish-version"
# shellcheck disable=SC1091
source "${mydir}/utils.sh"

usage() {
  msg
  msg "Usage: $0 <version_type>"
  msg
  msg "\t<version_type> must be one of patch or prerelease"
  msg
}

cleanup() {
  # Remove lock files due to conflicts with workspaces
  rm -f \
    "${mydir}/../packages/hoprd/npm-shrinkwrap.json"

  # Don't commit changed package.json files as package resolutions are
  # supposed to interfer with workspaces according to https://yarnpkg.com/configuration/manifest#resolutions
  git restore packages/hoprd/package.json

  # delete default environments
  rm -f \
    "${mydir}/../packages/hoprd/default-environment.json"
}

# return early with help info when requested
{ [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; } && { usage; exit 0; }

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

# define packages
declare -a versioned_packages=( utils connect ethereum core-ethereum core real hoprd )

# ensure local copy is up-to-date with origin
branch=$(git rev-parse --abbrev-ref HEAD)
git pull origin "${branch}" --rebase --tags

# get package info
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)

# ensure the build is up-to-date
make -j -C "${mydir}/.." deps && make -j -C "${mydir}/.." build

declare current_version
current_version="$("${mydir}/get-package-version.sh")"
if [[ "${version_type}" = "prerelease" && "${branch}" != "staging/"* ]]; then
  # turn prerelease into preminor if the current version is not a prerelease
  # already, thus update from a patch version to a new minor prerelease
  # This applies only to non-staging branches
  if [[ "${current_version}" != *"-next."* ]]; then
    version_type="preminor"
  fi
fi

log "get default environment id"

declare environment_id
environment_id="$("${mydir}/get-default-environment.sh")"

log "using version template ${current_version} + ${version_type}"
declare new_version
new_version=$(npx semver --preid next -i "${version_type}" "${current_version}")
log "creating new version ${new_version}"

# create new version in each package
for p in "${versioned_packages[@]}"; do
  cd "${mydir}/../packages/${p}"
  jq ".version = \"${new_version}\"" package.json > package.json.new
  mv package.json.new package.json
  cd "${mydir}/.."
done

# commit changes and create Git tag
git add packages/*/package.json
git commit -m "chore(release): publish ${new_version}"

# in the meantime new changes might have come in which we need to rebase on before pushing
git pull origin "${branch}" --rebase --tags

# now tag and proceed
git tag "v${new_version}"

# only make remote changes if running in CI
if [ "${CI:-}" = "true" ] && [ -z "${ACT:-}" ]; then
  # we push changes back onto origin
  git push origin "${branch}"
  # we only push the tag if we succeeded to push the changes onto master
  git push origin tag "v${new_version}"

  # publish each workspace package to npm
  if [ -n "${NODE_AUTH_TOKEN:-}" ]; then
    yarn config set npmAuthToken "${NODE_AUTH_TOKEN:-}"
  fi

  # pack and publish packages
  yarn workspaces foreach -piv --topological-dev \
    --exclude hoprnet --exclude hopr-docs \
    --exclude @hoprnet/hoprd \
    npm publish --access public

  "${mydir}/wait-for-npm-package.sh" utils
  "${mydir}/wait-for-npm-package.sh" connect
  "${mydir}/wait-for-npm-package.sh" ethereum
  "${mydir}/wait-for-npm-package.sh" core-ethereum
  "${mydir}/wait-for-npm-package.sh" core
  "${mydir}/wait-for-npm-package.sh" real

  trap cleanup SIGINT SIGTERM ERR

  # set default environments
  log "adding default environments to packages"
  echo "{\"id\": \"${environment_id}\"}" > "${mydir}/../packages/hoprd/default-environment.json"

  # special treatment for end-of-chain packages
  # to create lockfiles with resolution overrides
  "${mydir}/build-lockfiles.sh" hoprd

  yarn workspace @hoprnet/hoprd npm publish --access public

  cleanup
fi
