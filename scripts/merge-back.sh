#!/usr/bin/env bash

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

usage() {
  echo ""
  echo "Usage: $0 <release_name>"
  echo ""
  echo "$0 providence"
  echo
}

# return early with help info when requested
{ [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; } && { usage; exit 0; }

# set mydir
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)

if [ $# -lt 1 ]; then
  usage
  exit 1
fi

release_name=${1} # Can be any of these values Build | ReleaseCandidate | Patch | Minor | Major


if [  -z "$(git status --porcelain)" ]; then 
  git checkout release/${release_name}
  git pull
  git checkout master
  git pull
  git checkout -b merge-back-release-${release_name}
  git merge release/${release_name} > /tmp/merge.log || true
  cat /tmp/merge.log

  # Resolve conflicts for package.json
  packages=(core-ethereum core hoprd real utils)
  for package in "${packages[@]}"; do
    if [ `git diff packages/$package/package.json | wc -l` -gt "0" ]; then 
      if [ `git diff packages/$package/package.json  | grep @@ | wc -l` -eq "1" ]; then
        git checkout --theirs packages/$package/package.json
        git add packages/$package/package.json
      else
        echo "[ERROR] Review changes manully for ./packages/$package/package.json"
        exit 1
      fi
    fi
  done


  # Check if there are conflicts
  conflicts=$(grep "CONFLICT" /tmp/merge.log | grep -v package.json | wc -l)
  if [[ "$conflicts" -gt "0" ]];
  then
    echo "[ERROR] There are conflicts in source code files"
    echo "[ERROR] You will need to continue manually on branch 'merge-back-release-${release_name}' or abort 'git merge --abort'"
    exit 1
  fi

  
  git commit -am "Merge branch 'master' into merge-back-release-${release_name}"
  git push --set-upstream origin merge-back-release-${release_name}
  echo "[INFO] Created remote branch merge-back-release-${release_name}"
  today=`date +%Y-%m-%d`
  echo "Creating github pull request using github cli"
  gh pr create --title "Merge back from ${release_name} - ${today}" --base master --label merge-back --reviewer hoprnet/hopr-development --body "The scope of this PR is to merge back to master all the bug fixing found in release ${release_name}"
else 
  echo "[ERROR] Clean your workspace before"
  exit 1
fi
