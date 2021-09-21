# Development Processes

The HOPR Association _tech_ members have agreed on the following development processes to streamline the implementation of the HOPR protocol.

- [Development Processes](#development-processes)
  - [Legend](#legend)
    - [Ambassadors](#ambassadors)
    - [Trifecta](#trifecta)
  - [Daily updates](#daily-updates)
    - [Start of work](#start-of-work)
      - [Text Update](#text-update)
      - [Video call](#video-call)
    - [End of working day](#end-of-working-day)
    - [Absence](#absence)
  - [Sprints](#sprints)
    - [Task Grooming](#task-grooming)
      - [External issues](#external-issues)
    - [Sprint Retrospective](#sprint-retrospective)
    - [Epic Prioritization](#epic-prioritization)
      - [The Roadmap columns](#the-roadmap-columns)
    - [Sprint Planning](#sprint-planning)
    - [Fire alarm](#fire-alarm)
  - [Development](#development)
    - [Principles](#principles)
    - [Issue creation](#issue-creation)
    - [Rules](#rules)
    - [Releases](#releases)
    - [Workflow](#workflow)
      - [Daily Development](#daily-development)
      - [Release Cycle](#release-cycle)
    - [Actions](#actions)
      - [Pre-release Version Bump (`feature` -> `master` = `x.y.z-0.next.*` -> `x.y.z-0.next.* + 1`)](#pre-release-version-bump-feature---master--xyz-0next---xyz-0next--1)
      - [Release Version Bump (`master` -> `release/**` = `x.y.z-0.next.*` -> `x.y.0`)](#release-version-bump-master---release--xyz-0next---xy0)
      - [Minor Version Bump (`release/**` -> `master` = `x.y.*` -> `x.y+1.0-next.0`)](#minor-version-bump-release---master--xy---xy10-next0)
    - [Branches](#branches)
    - [Labels](#labels)

## Legend

| Name          | Description                                                                   |
| ------------- | ----------------------------------------------------------------------------- |
| Project Owner | _TODO_                                                                        |
| Ambassadors   | Two tech members elected to be the bridge between tech team and Project Owner |
| Trifecta      | Ambassadors + Project Owner                                                   |
| Issue tracker | The sole issue tracker is [GitHub](https://github.com/hoprnet)                |

### Ambassadors

Elected ambassadors are rotated every `2 months`.

- bridge between tech team and Project Owner

_Current:_ [Tino](https://github.com/tolbrino), [Steve](https://github.com/nionis)
_Expiry:_ 5/11/21

### Trifecta

Trifecta is made up by the Project Owner and the ambassadors.

- bi-weekly [epic priorities](#epic-prioritization) check
- task duration estimation done before [Sprint Planning](#sprint-planning)

_Current:_ [Tino](https://github.com/tolbrino), [Steve](https://github.com/nionis), [Sebastian](https://github.com/SCBuergel)

## Daily updates

### Start of work

#### Text Update

For every working day, all members are required to write an update on what they will be working on throughout the day. Ideally, the list should be accompanied by github issues or PRs.

_Required:_ `True`

_When:_ Every working day, at start of work

_Where:_ In element channel `tech - daily`

#### Video call

For every working day except Mondays, all members are required to join the video call with camera. The call is an opportunity for everybody to catch up, call out on things that are blocking them and any other unexpected issues.

_Required:_ `True`

_When:_ Every working day except Mondays, time determined by google meet invite

_Where:_ Google meet

### End of working day

At the end of every working day, members may write an update on what they have accomplished during the day.

_Required:_ `False`

_When:_ Every day, at end of work

_Where:_ In element channel `tech - eod`

### Absence

Every Team Member is responsible for letting the rest of the team know about illness/absence/vacation.

- unexpected absence needs to be reported ASAP through email

_TODO: add info about planned absences_

## Sprints

Sprints are finite timeframes which have defined task priorities which the
team tries to adhere to as much as possible. Outside of emergency requests
the priorities stay the same within a sprint's timeframe.

_Length:_ 2 weeks

_Start:_ first day of the week, mostly Monday

The start and end of a sprint is defined by a set of activities in which
mostly the whole team is involved in:

1. Task Grooming
2. Sprint Retrospective
3. Epic Prioritization
4. Task Planning

_TODO: define a small guide how the meetings are carried out_

### Task Grooming

On the last day of a sprint all members shall spend time cleaning up
tasks and PRs on Github. This includes but is not limited to:

- updating progress on tasks
- closing tasks which are actually done
- updating and/or closing PRs to ensure only in-progress PRs are active
- moving tasks on sprint boards to correct columns
- [responding to issues created by externals](#external-issues)

_Who:_ all members individually

_When:_ first day of the sprint or before a vacation

#### External issues

Newly created issues may not all have been replied to, the [Ambassadors](#ambassadors) may reply or assign a tech member to follow up on the issue.

- closing duplicates
- issues are properly labelled

### Sprint Retrospective

In `Retrospective` we aim to summurize the results of the last spring, in order to help us identify and fix issues in our processes and company culture.

- what went better than expected
- what went worse than expected
- lessons learned

_Who:_ all members within a meeting

_When:_ first day of the sprint

### Epic Prioritization

The target of prioritization are issues marked as
[epic](https://github.com/hoprnet/hoprnet/issues?q=is%3Aissue+is%3Aopen+label%3Aepic).
Priorities are captured on the [Roadmap](https://github.com/orgs/hoprnet/projects/9) which only contains epic issues.

- closed epics are moved to `Acceptance`
- epics within `Acceptance` column are accepted by [Trifecta](#trifecta) prior to the meeting
- ensure newly created epics are well created _TODO: define_
- adapt epic priorities
- create new [Sprint](#sprint-planning) project

#### The Roadmap columns

- `Icebox` contains epics which require further specification or are specifically paused.
- `Backlog` contains epics which are well specified but haven't been given any
  priority to be worked on during the current sprint.
- `Next` column contains epics which are given priority to be worked on during
  the current sprint. The priorities are descending from top to bottom. Priorites
  with hard deadlines must be marked with the label
  [deadline](https://github.com/hoprnet/hoprnet/labels/deadline) with more
  information on the deadline being available within the issue's description.
- `Acceptance` column contains epics which were completed but require
  acceptance testing from an additional team member or outside person. When moving
  issues into `Acceptance` the person who's input is required must be pinged
  directly.
- `Done` column contains epics which were accepted. The column is cleaned up
  as part of the `Task Grooming` phase.

_Who:_ [Trifecta](#trifecta)

_When_: first day of a sprint

### Sprint Planning

The task planning follows the `Prioritization` and takes the priorities into
account as much as possible.

- Each sprint has its own taskboard, e.g. [Sprint 36-37](https://github.com/orgs/hoprnet/projects/11).
- Epic issues are further refined into tasks which have clear definitions of work and done.
- Ideally each team member has only one task assigned which is `in progress`.
- members which are coming back from vacation should already have tasks assigned to them. _TODO: whos job?_

_Who:_ all members within a meeting

_When:_ first day of the sprint

### Fire alarm

It's possible that throughout the planned sprint, we encounter a bag of issues that need to be resolved ASAP.

1. As soon issue is detected, an [ambassador](#ambassadors) needs to take up the task of coordinating how the issue is tackled
2. Member appointed then is responsible for finding the right member within the tech team that has the most knowledge on the current issue, let's say that's Alice
3. Alice investigates issue and may ask other members for help
4. Alice patches issue

## Development

### Principles

- Context is the enemy. Be obvious, not clever.
- Simple is always better. Short, concise.
- Code for maintainability, not perfection.
- Automation-first. Whenever possible, rely on automation.
- Test early, test often. They prevent bad deployments and API regressions.

### Issue creation

_TODO: epics, other_

### Rules

- All PR‘s **have to be approved** by at least two members.
- Respect our CI pipeline. If you break it, you fix it.
- All PR‘s must pass all status checks/tests before merging.
- Releases can be merged back to `master`, but not always necessary.
- When in conflict, chat and engage with the team.

### Releases

To release, we do a code-freeze in our codebase by branching out a specific
release by naming the branch `release/*`. New patches are then merged on that
particular branch to deploy on every change.

### Workflow

#### Daily Development

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
   a code-review from a maintainer. In the case of members, the approval of an
   additional team member is required. Externals require **two** maintainers to
   approve the change.

4. After a different team member reviews the code and indicates approval, the PR
   can be merged. If the history is messy, the PR can be squashed, otherwise, it is
   merged. Use common sense to decide when you should do which one.

#### Release Cycle

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
   then shared within our #release channel. On the #testing channel, members are expected
   to run their own nodes (either AVADO or via their workstation) to participate in the release.
3. Patches to the release are created via `hotfix/**` branches. Each of these merges will trigger
   a new release version, and re-build our infrastructure for that version. Upon successfullly
   testing a release, merge it back to trigger a new pre-release version via our actions.

### Actions

We made active use of actions to automate tasks trivial to our workflow.

#### Pre-release Version Bump (`feature` -> `master` = `x.y.z-0.next.*` -> `x.y.z-0.next.* + 1`)

When a PR to `master` is merged, an action bumps the package.json pre-release
version and commits that change to `master`. When a PR to `master` is merged,
a tag on GitHub is pushed specifying that feature on that version.

#### Release Version Bump (`master` -> `release/**` = `x.y.z-0.next.*` -> `x.y.0`)

On first build, a `release/**` bumps the package.json by a `minor`, clearing
the `pre-release` tag. Subsequent commits on `release` branches bump the
`patch` version.

#### Minor Version Bump (`release/**` -> `master` = `x.y.*` -> `x.y+1.0-next.0`)

After testing a release, we can merge it back to `master` to trigger a bump on the
package.json by a `minor`, and restoring the `pre-release` tag, to keep our normal
daily workflow as it was before.

### Branches

- `master`: In our case, `master` is a **prerelease** branch - tests _must_
  pass, and code _should_ be stable, but its _acceptable_ to have issues.

- `release/**`: On every [Milestone](https://github.com/hoprnet/hoprnet/milestones)
  marked with a **PUBLIC RELEASE**, we cut a `release/**`
  branch, using an internal name to identify the release. Official milestones
  use a specific name with the codename of a mountain in Switzerland.

<!--
TODO:

- fix various TODOs in document
- emailing process
-->

### Labels

All hopr repositories need to support these labels.

| Name    | Description                           | Color                                           |
| ------- | ------------------------------------- | ----------------------------------------------- |
| epic    | An issue which tracks multiple issues | <span style="background:#2473b7">#2473b7</span> |
| bug     | Something isn't working               | <span style="background:#d73a4a">#d73a4a</span> |
| feature | New feature or request                | <span style="background:#d73a4a">#a2eeef</span> |

_TODO: extend_
