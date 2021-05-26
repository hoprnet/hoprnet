#!/usr/bin/env bash

# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -euo pipefail

usage() {
  echo >&2
  echo "Usage: $0 <git-ref> <git-commit-msg>" >&2
  echo >&2
}

# return early with help info when requested
([ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]) && { usage; exit 0; }

# verify and set parameters
[ -z "${1:-}" ] && { echo "Missing parameter <git-ref>" >&2; usage; exit 1; }
[ -z "${2:-}" ] && { echo "Missing parameter <git-commit-msg>" >&2; usage; exit 1; }

declare git_ref
declare git_commit_msg

git_ref="$1"
git_commit_msg="$2"

# do work

# only do work when there are actual changes
if [ -n "$(git status --porcelain)" ]; then
  git add .
  git commit -m "${git_commit_msg}"
  git push origin "${git_ref}"
fi
