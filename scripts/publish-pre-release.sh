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
mydir=$(dirname $(readlink -f $0))
npm_package=$(${mydir}/get-npm-package-info.sh)

# identify version type
test -n "${npm_package}" && version_type="preminor" || version_type="prerelease"

# create new version in each package, and tag in Git
npx lerna version "${version_type}" --yes --exact --no-push --no-changelog --preid next

# only make remote changes if running in CI
if [ -n "${HOPR_IN_CI:-}" ]; then
  # push changes back onto origin including new tag
  git push origin "${branch}" --tags

  # publish version to npm
  npx lerna publish from-package --yes
fi
