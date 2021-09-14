# Development Workflow

The HOPR Association core team members have agreed on the following planning
an development processes to streamline the implementation of the HOPR protocol.

## Sprints

Sprints are finite timeframes which have defined task priorities which the
team tries to adhere to as much as possible. Outside of emergency requests
the priorities stay the same within a sprint's timeframe.

*Length:* 2 weeks
*Start:*  first day of the week, mostly Monday

The start and end of a sprint is defined by a set of activities in which
mostly the whole team is involved in:

1. Task Grooming
2. Retrospective
3. Prioritization
4. Planning

## Task Grooming

On the last day of a sprint all team members shall spend time cleaning up
tasks and PRs on Github. This includes but is not limited to:

- responding to bug reports
- updating progress on tasks
- closing tasks which are actually done
- updating and/or closing PRs to ensure only in-progress PRs are active
- moving tasks on task boards to correct columns

*Who:*  all team members individually
*When:* first day of the sprint

## Retrospective

TODO

*Who:*  all team members within a meeting
*When:* first day of the sprint

## Prioritization

The target of prioritization are issues marked as
[epic](https://github.com/hoprnet/hoprnet/issues?q=is%3Aissue+is%3Aopen+label%3Aepic).
Priorities are captured on the [Roadmap](https://github.com/orgs/hoprnet/projects/9) which only contains epic issues.

The `Icebox` contains epics which require further specification or are specifically paused.
The `Backlog` contains epics which are well specified but haven't been given any
priority to be worked on during the current sprint.
The `Next` column contains epics which are given priority to be worked on during
the current sprint. The priorities are descending from top to bottom. Priorites
with hard deadlines must be marked with the label
[deadline](https://github.com/hoprnet/hoprnet/labels/deadline) with more
information on the deadline being available within the issue's description.
The `Acceptance` column contains epics which were completed but require
acceptance testing from an additional team member or outside person. When moving
issues into `Acceptance` the person who's input is required must be pinged
directly.
The `Done` column contains epics which were accepted. The column is cleaned up
as part of the `Task Grooming` phase.

*Who:*  3 team members which were selected by consensus, currently @nionis @tolbrino @SCBuergel
*When*: first day of a sprint

## Task Planning

The task planning follows the `Prioritization` and takes the priorities into
account as much as possible. Epic issues are further refined into tasks which
have clear definitions of work and done. Each sprint has its own taskboard, e.g.
[Sprint 36-37](https://github.com/orgs/hoprnet/projects/11). Ideally each team
member has only one task assigned which is `in progress`.

*Who:*  all team members within a meeting
*When:* first day of the sprint

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

## Releases

To release, we do a code-freeze in our codebase by branching out a specific
release by naming the branch `release/*`. New patches are then merged on that
particular branch to deploy on every change.

## Workflow

### Daily Development

```


   release/constantine          master          jjpa/solves-channels-1556

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

   hotfix/patch-constantine    release/constantine          master

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
