# Development workflow

Principles:

- Rely on automation as much as possible.
- Automated tests should prevent bad deployments. Continuous integration,
  continuous deployment.

## Goal Workflow

- A team-member, or an external contributor writes code in a feature branch, or
  in their own fork.

- The PR should track master

- They create a draft pull-request while they work on the code so we can stay
  up to date on progress.

- Every push to that branch builds and runs the tests.

- When the code is ready, they mark the pull-request as ready to review.

- Ideally a team member reviews the code, and indicates approval. For external
  contributors, this is mandatory.

- PR's should not be merged until the status checks pass.

- If the history is messy, the PR can be squashed, otherwise it is merged.

- When the PR is merged, an action bumps the package.json and commits that
  change to master/develop.

- A tag is pushed with that code.

## Branches

- Master is a prerelease branch - tests _must_ pass, and code _should_ be stable,
  but we may have issues.

- When we want to do a real release, we cut a `release/` branch. This should
  bump the minor or major version (todo. workout tags)

- Pushes to release branches should trigger a `patch` release.

- Bugfixes to releases should be PR's to release branches. The release branch
  should then be pulled back into master.

- Master tracks releases, but releases do not track master.
