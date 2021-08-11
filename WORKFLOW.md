# Development workflow

## Introduction

The HOPR Association team members have agreed on the following development
workflow to streamline the process we use to implement the HOPR protocol.

## Principles

- Context is the enemy. Be obvious, not clever.
- Simple is always better. Short, concise.
- Code for maintainability, not perfection.
- Automation-first. Whenever possible, rely on automation.
- Test early, test often. They prevent bad deployments and API regressions.

## Rules

- All PR‘s **have to be approved** by at least two team members.
- Respect our CI pipeline. If you break it, you fix it.
- All PR‘s must pass all status checks/tests before merging.
- Releases can be merged back to `master`, but not always necessary.
- When in conflict, chat and engage with the team.
- An Epic must contain a problem, a solution, and a list of sub-issues. It must also state a DONE-criteria that can be checked before the epic is marked complete.
- An epic's solution **have to be approved** by at least two team members before implementation starts.

## Releases

To release, we do a code-freeze in our codebase by branching out a specific
release by naming the branch `release/*`. New patches are then merged on that
particular branch to deploy on every change.

## Workflow

### Daily Development

```


   release/moscow          master          jjpa/solves-channels-1556

         x                   │                       x
         x                   │                       x
         x                   │                       x
         x                   └──────────────────────►┐
         x                   x                       │
         x                   x                       │
         x                   x                       │
         x                   ◄───────────────────────┘
         x                   x                       x
         x                   x                       x
         x                   x                       x
         ◄────────────────────                       x
         x                   x                       x
         x                   x                       x
         x                   x                       x

```

1. A team-member or an external contributor writes code in a **feature branch**,
   or in their own fork. The feature branch describe its goal, and any issue it is
   related to, or whom it's working on it. (e.g. “jjperezaguinaga/fixes-341")

2. Soon after the creation of the branch, create a draft pull-request to
   stay up to date on progress. That way, every push will build the project and
   run the tests on every single push.

3. When the code is ready, mark the pull-request as ready to review, and request
   a code-review from a maintainer. In the case of team members, the approval of an
   additional team member is required. Externals require **two** maintainers to
   approve the change.

4. After a different team member reviews the code and indicates approval, the PR
   can be merged. If the history is messy, the PR can be squashed, otherwise, it is
   merged. Use common sense to decide when you should do which one.

### Release Cycle

```

   hotfix/patch-moscow    release/moscow          master

         x                   x                       x 1.74.0-next.44
         x                   │ ◄─────────────────────x
         x                   │                       x
         x                   │ 1.74.0                x
         x                   │                       x
         x                   ▼                       x
         ┌◄──────────────────x                       x
         │                   x                       x
         │                   x                       x
         │                   x                       x
         ▼──────────────────►┐ 1.74.1                x
         x                   │                       x
         x                   ▼──────────────────────►x 1.75.0-next.0
         x                   x                       x
         x                   x                       x

```

1. On every public release agreed as a [Milestone](https://github.com/hoprnet/hoprnet/milestones),
   the PM Lead of the week will code-freeze `master` by creating a `release/**` branch
   tracking `master`. Release specific changes will be done in this branch to trigger
   this particular release, which requires to insert name and release version of the new milestone
   in the file `scripts/environment.sh` as well as `packages/avado/Dockerfile` and add an entry to `CHANGELOG.md`.

2. The information about the release, how to test and what commands to run, are
   then shared within our #release channel. On the #testing channel, team members are expected
   to run their own nodes (either AVADO or via their workstation) to participate in the release.
3. Patches to the release are created via `hotfix/**` branches. Each of these merges will trigger
   a new release version, and re-build our infrastructure for that version. Upon successfullly
   testing a release, merge it back to trigger a new pre-release version via our actions.

## Actions

We made active use of actions to automate tasks trivial to our workflow.

### Pre-release Version Bump (`feature` -> `master` = `x.y.z-0.next.*` -> `x.y.z-0.next.* + 1`)

When a PR to `master` is merged, an action bumps the package.json pre-release
version and commits that change to `master`. When a PR to `master` is merged,
a tag on GitHub is pushed specifying that feature on that version.

### Release Version Bump (`master` -> `release/**` = `x.y.z-0.next.*` -> `x.y.0`)

On first build, a `release/**` bumps the package.json by a `minor`, clearing
the `pre-release` tag. Subsequent commits on `release` branches bump the
`patch` version.

### Minor Version Bump (`release/**` -> `master` = `x.y.*` -> `x.y+1.0-next.0`)

After testing a release, we can merge it back to `master` to trigger a bump on the
package.json by a `minor`, and restoring the `pre-release` tag, to keep our normal
daily workflow as it was before.

## Branches

- `master`: In our case, `master` is a **prerelease** branch - tests _must_
  pass, and code _should_ be stable, but its _acceptable_ to have issues.

- `release/**`: On every [Milestone](https://github.com/hoprnet/hoprnet/milestones)
  marked with a **PUBLIC RELEASE**, we cut a `release/**`
  branch, using an internal name to identify the release. Official milestones
  use a specific name with the codename of a mountain in Switzerland.
