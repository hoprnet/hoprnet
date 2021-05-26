#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -euo pipefail

usage() {
  echo >&2
  echo "Usage: $0" >&2
  echo >&2
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

# verify and set parameters
declare branch

branch=$(git rev-parse --abbrev-ref HEAD)

# do work

# create new version in each package, and tag in Git
npx lerna version patch --yes --exact --no-push --no-changelog

# only make remote changes if running in CI
if [ -n "${CI:-}" ]; then
  # push changes back onto origin including new tag
  git push origin "${branch}" --tags

  # publish version to npm
  npx lerna publish from-package --yes
fi
