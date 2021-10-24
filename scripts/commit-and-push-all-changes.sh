#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

test -z "${HOPR_GIT_MSG:-}" && (echo "Missing environment variable HOPR_GIT_MSG"; exit 1)
test -z "${HOPR_GITHUB_REF:-}" && (echo "Missing environment variable HOPR_GITHUB_REF"; exit 1)

# only do work when there are actual changes
if [ -n "$(git status --porcelain)" ]; then
  git add .
  git commit -m "${HOPR_GIT_MSG}"
fi

# must get the latest version of the branch from origin before pushing
git pull origin "${HOPR_GITHUB_REF}" --rebase --strategy-option recursive -X ours # NB! when pull rebasing, ours is the incoming change (see https://stackoverflow.com/a/3443225)

git push origin "${HOPR_GITHUB_REF}"
