#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

test -z "${HOPR_GITHUB_REF:-}" && (echo "Missing environment variable HOPR_GITHUB_REF"; exit 1)

# ensure local copy is up-to-date with origin
git pull origin "${HOPR_GITHUB_REF}"

# ensure the build is up-to-date
yarn
yarn build

# create new version in each package, and tag in Git
npx lerna version patch --yes --exact --no-push --no-changelog \
  -m "chore(release): publish %s"

# only make remote changes if running in CI
if [ "${CI:-}" = "true" ] && [ -z "${ACT:-}" ]; then
  # push changes back onto origin including new tag
  git push origin "${HOPR_GITHUB_REF}" --tags

  # publish version to npm
  npx lerna publish from-package --yes
fi
