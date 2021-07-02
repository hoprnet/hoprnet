#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

declare branch
declare mydir
declare npm_package
declare version_type

# ensure local copy is up-to-date with origin
branch=$(git rev-parse --abbrev-ref HEAD)
git pull origin "${branch}" --rebase

# get package info
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
npm_package=$(${mydir}/get-npm-package-info.sh)

# identify version type
test -n "${npm_package}" && version_type="preminor" || version_type="prerelease"

# ensure the build is up-to-date
yarn
yarn build

# create new version in each package, and tag in Git
npx lerna version "${version_type}" --yes --exact --no-push --no-changelog \
  --preid next -m "chore(release): publish %s"

# only make remote changes if running in CI
if [ "${CI:-}" = "true" ] && [ -z "${ACT:-}" ]; then
  # push changes back onto origin including new tag
  git push origin "${branch}" --tags

  # publish version to npm
  npx lerna publish from-package --yes
fi
