#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

test -z "${HOPR_GIT_MSG:-}" && (echo "Missing environment variable HOPR_GIT_MSG"; exit 1)
test -z "${HOPR_GITHUB_REF:-}" && (echo "Missing environment variable HOPR_GITHUB_REF"; exit 1)

# only do work when there are actual changes
if [ -n "$(git status --porcelain)" ]; then
  git pull origin "${HOPR_GITHUB_REF}"
  git add .
  git commit -m "${HOPR_GIT_MSG}"
  git push origin "${HOPR_GITHUB_REF}"
fi
