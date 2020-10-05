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

- They add a label indicating whether the code is a major, minor or patch
  release. The default, if no label is given, is an alpha release.
  If you prefer a different release version, use a tag on the pull request to
  specify, `Major version`, `Minor version` or `Patch version`

- Ideally a team member reviews the code, and indicates approval. For external
  contributors, this is mandatory.

- PR's should not be merged until the status checks pass.

- If the history is messy, the PR can be squashed, otherwise it is merged.

- When the PR is merged, an action bumps the package.json and commits that
  change to master/develop.

- A tag is pushed with that code.
