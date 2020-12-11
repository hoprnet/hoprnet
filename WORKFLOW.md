# Development workflow

## Introduction

The HOPR Association team members have agreed on the following development
workflow to streamline the process we use to implement the HOPR protocol.

## Principles

- **Automation-first**: Rely on automation as much as possible. **Tests & CI**:
- Tests should prevent bad deployments and API regressions. Continuous
- integration ensures all our code is tested before being merged into our
- baseline. **Releases & CD**: Every week, we do a code-freeze in our codebase
- by branching out a specific release, which we then deploy on every change.

## Rules

- All PR‘s have to be approved by a team member different from the one who
- created the PR. All PR‘s must pass all status checks before merging.
- Releases can be merged back to master, but not always necessary.

## Workflow

### Daily Development

1. A team-member or an external contributor writes code in a **feature branch**,
   or in their own fork. The feature branch describe its goal, and any ticket it is
   related to. (e.g. “fixes 341")

2. Soon after the creation of the branch, create a draft pull-request to
   stay up to date on progress. That way, every push will build the project and
   run the tests.

3. When the code is ready, mark the pull-request as ready to review, and request
   a code-review from a maintainer. In the case of team members, the approval of an
   additional team member is required. Externals require **two** maintainers to
   approve the change.

4. After a different team member reviews the code and indicates approval, the PR
   can be merged. If the history is messy, the PR can be squashed, otherwise, it is
   merged.

### Release Cycle

1. On Thursdays at 12 pm CEST, the PM Lead of the week will code-freeze `master`
   by creating a `release/**` branch tracking `master`. Release specific changes
   will be done in this branch to trigger this particular release.

2. The `CHANGELOG.md` file is updated with the GitHub pull requests merged
   during that milestone.

3. The instructions with the release, how to test and what bootstrap server to
   use, are then shared in our social media channels.

## Actions

We made active use of actions to automate tasks trivial to our workflow.

- **Pre-release Version Bump**. When a PR to `master` is merged, an action bumps
- the package.json pre-release version and commits that change to `master`.

- **Tag Release**. When a PR to `master` is merged, a tag is pushed specifying
- that feature on that version.

- **Release Version Bump**. On first build, a `release/**` bumps the
- package.json by a `minor`, clearing the `pre-release` tag. Subsequent commits
- on `release` branches bump the `patch` version.

## Branches

- `master`: In our case, `master` is a **prerelease** branch - tests _must_
  pass, and code _should_ be stable, but its _acceptable_ to have issues.

- `release/**`: Every week on Thursdays at 12 pm CEST, we cut a `release/**`
  branch, using an internal name to identify the release.
