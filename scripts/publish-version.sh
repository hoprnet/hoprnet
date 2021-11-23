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
git pull origin "${branch}" --rebase --tags

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

log "get default environment id"

declare environment_id
for git_ref in $(cat "${mydir}/../packages/hoprd/releases.json" | jq -r "to_entries[] | .value.git_ref" | uniq); do
  if [[ "${branch}" =~ ${git_ref} ]]; then
    environment_id=$(cat "${mydir}/../packages/hoprd/releases.json" | jq -r "to_entries[] | select(.value.git_ref==\"${git_ref}\" and .value.default==true) | .key")
    # if no default is set we take the first entry
    if [ -z "${environment_id}" ]; then
      environment_id=$(cat "${mydir}/../packages/hoprd/releases.json" | jq -r "to_entries[] | select(.value.git_ref==\"${git_ref}\") | .key" | sed q)
    fi
    break
  fi
done
: ${environment_id:?"Could not read value for default environment id"}

log "creating new version ${current_version} + ${version_type}"

# create new version in each package
yarn workspaces foreach -piv --topological-dev --exclude hoprnet exec -- npm version ${version_type} --preid=next
declare new_version
new_version=$(${mydir}/get-package-version.sh)

# commit changes and create Git tag
git add packages/*/package.json
git commit -m "chore(release): publish ${new_version}"

# in the meantime new changes might have come in which we need to rebase on before pushing
git pull origin "${branch}" --rebase --tags

# now tag and proceed
git tag v${new_version}

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

  # set default environments
  echo "{\"id\": \"${environment_id}\"}" > "${mydir}/../packages/hoprd/default-environment.json"
  echo "{\"id\": \"${environment_id}\"}" > "${mydir}/../packages/cover-traffic-daemon/default-environment.json"

  # pack and publish packages
  yarn workspaces foreach -piv --topological-dev --exclude hoprnet npm publish --access public

  # delete default environments
  rm -f \
    "${mydir}/../packages/hoprd/default-environment.json" \
    "${mydir}/../packages/cover-traffic-daemon/default-environment.json"
fi
