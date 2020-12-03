# Development workflow

## Introduction

The HOPR Association team members have agreed on the following development workflow to streamline the process we use to implement the HOPR protocol.

## Principles

- **Automation-first**: Rely on automation as much as possible.
- **Tests & CI**: Tests should prevent bad deployments and API regressions. Continuous integration ensures all our code is tested before being merged into our baseline.
- **Releases & CD**: Every week, we do a code-freeze in our codebase by branching out a specific release, which we then deploy on every change.

## Rules

* All PR‘s should track to master.
* All PR‘s have to be approved by two team members.
* All PR‘s must pass all status checks before merging.
* `master` tracks `release/**`, but releases do not track `master`. i.e. we always merge `release/**` back to `master`, but we do not merge `master` back to a frozen `release`.

## Workflow

### Daily Development

1. A team-member or an external contributor writes code in a **feature branch**, or in their own fork. The feature branch should track an **issue**, and describe its role (e.g. “fixes 341")

2. Immediately after the creation of the branch, create a draft pull-request to stay up to date on progress. That way, every push on will build the project and runs the tests.

3. When the code is ready, mark the pull-request as ready to review, and request a code-review from a maintainer. In the case of team members, the approval of an additional team member is required. Externals require **two** maintainers to approve the change.

4. After a different team member reviews the code and indicates approval, the PR can be merged. If the history is messy, the PR can be squashed, otherwise, it is merged.

### Release Cycle

1. On Thursdays at 12 pm CEST, the PM Lead of the week will code-freeze `master` by creating a `release/**` branch tracking `master`. Release specific changes will be done in this branch to trigger this particular release then.

2. **(TEMPORAL, see #709)** After the `release/**` action has completed, we then crease a `cd/**` branch tracking `release/**`.

3. The instructions with the release, how to test and what bootstrap server to use, are then shared in our social media channels.

## Actions

We made active use of actions to automate tasks trivial to our workflow.

- **Patch Version Bump**. When a PR to `master` is merged, an action bumps the package.json by a `patch` and commits that change to `master`.

- **Tag Release**. When a PR to `master` is merged, a tag is pushed specifying that feature on that version.

- **Release Version Bump**. When a `release/**` branch is pushed with a new commit, an action bumps the package.json by a `minor` and commits that change to the `release` branch. Further pushes on `release` branches would then bump the version by a `patch`.

## Branches

- `master`: In our case, `master` is a **prerelease** branch - tests *must* pass, and code *should* be stable, but its *acceptable* to have issues.

- `release/**`: Every week on Thursdays at 12 pm CEST, we cut a `release/**` branch, using an internal name to identify the release.

- `hotfix/**/**`: Hotfixes to be PR's to release branches. The release branch should then be pulled back into master.

- `cd/**` **TEMPORAL, see (#709)**: After a successful release, we then branch out a `cd` branch which will deploy the artefacts based on the release builds from our CI.

## Additional Notes

Currently, we are relying on a separate step called `cd` to create the required servers for the release to work. This will be phased out after #709 and #627 are completed.
