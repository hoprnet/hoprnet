# Development Processes

The HOPR Association _tech_ members and [Project Owner](#legend) have agreed on the following development processes to streamline the implementation of the HOPR protocol.

- [Development Processes](#development-processes)
  - [Legend](#legend)
    - [Representantives](#representantives)
    - [Trifecta](#trifecta)
  - [Daily updates](#daily-updates)
    - [Start of work](#start-of-work)
      - [Text update](#text-update)
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
    - [Labels](#labels)
    - [Rules](#rules)
    - [Workflow](#workflow)
      - [Daily Development](#daily-development)
      - [Releases](#releases)
    - [Branches](#branches)

## Legend

| Name                | Description                                                                                                |
| ------------------- | ---------------------------------------------------------------------------------------------------------- |
| Project Owner       | A HOPR member which ensures epics prioritized are in line with both the short and long term vision of HOPR |
| Process Facilitator | A HOPR member which ensures agreed processes are followed                                                  |
| Representantives    | Two tech members elected to be the bridge between tech team and Project Owner                              |
| Trifecta            | Representantives + Project Owner                                                                           |
| Issue tracker       | The sole issue tracker is [GitHub](https://github.com/hoprnet)                                             |

### Representantives

Each individual representantive is elected for a period of `3 months`. The periods are overlapped to create a staggered schedule during which new representatives can be guided by their co-representatives who have already served half of their election period. Previous representatives may be re-elected.

Responsibilities:

- to be the bridge between tech team and Project Owner
- daily check for community created tasks in [hopr-devrel](https://github.com/hoprnet/hopr-devrel)
- daily check for new [unreleased tokens](../UNRELEASED_TOKENS.md) emails send to `tech@hoprnet.org`

_Current:_

- [Tino](https://github.com/tolbrino) until 05/02/2022
- [Steve](https://github.com/nionis) until 20/12/2021

### Trifecta

Trifecta is made up by the [Representantives](#representantives) and Project Owner.

Responsibilities:

- bi-weekly [epic priorities](#epic-prioritization) check
- task duration estimation done before [Sprint Planning](#sprint-planning)
- [file alarm](#fire-alarm) management
- [release](./release.md#internal-release) management

_Current:_ [Representantives](#representantives), [Sebastian](https://github.com/SCBuergel)

## Daily updates

### Start of work

#### Text update

For every working day, all tech members are required to write an update on what they will be working on throughout the day. Ideally, the list should be accompanied by github issues or PRs.

_Required:_ `True`

_When:_ Every working day, at start of work

_Where:_ In element channel `tech - daily`

#### Video call

For every working day except Mondays, all tech members are required to join the video call with camera. The call is an opportunity for everybody to catch up, call out on things that are blocking them and any other unexpected issues.

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

### Task Grooming

On the last day of a sprint all tech members shall spend time cleaning up
tasks and PRs on Github. This includes but is not limited to:

- updating progress on tasks
- closing tasks which are actually done
- updating and/or closing PRs to ensure only in-progress PRs are active
- moving tasks on sprint boards to correct columns
- [responding to issues created by externals](#external-issues)
- delete stale/old branches which are not attached to a PR

_Who:_ all tech members individually

_When:_ last day of the sprint or before a vacation

#### External issues

Newly created issues ([labelled as](#labels): `new issue`) may not all have been replied to, the [Representantives](#representantives) may reply or assign a tech member to follow up on the issue.

- closing duplicates
- issues are properly labelled

### Sprint Retrospective

In `Retrospective` we aim to summarize the results of the last spring, in order to help us identify and fix issues in our processes and company culture.

- what went better than expected
- what went worse than expected
- lessons learned

_Who:_ all tech members within a meeting

_When:_ first day of the sprint

### Epic Prioritization

The target of prioritization are issues marked as
[epic](https://github.com/hoprnet/hoprnet/issues?q=is%3Aissue+is%3Aopen+label%3Aepic).
Priorities are captured on the [Roadmap](https://github.com/orgs/hoprnet/projects/15) which only contains epic issues.

- epics are issues which encapsulate the work that need to be done for X feature/bug
- epics are not bound by sprints and they can be long lived
- closed epics are moved to `acceptance`
- epics within `acceptance` column are accepted by [Trifecta](#trifecta) prior to the meeting
  - unaccepted epics are moved back to `next` column with a comment explaining why it wasn't accepted
- ensure newly created [epics](#issue-creation) are well created
  - must use the [epic issue template](../.github/ISSUE_TEMPLATE/epic.md)
- adapt epic priorities
- make an epic if necessary for issues created by community in [hopr-devrel](https://github.com/hoprnet/hopr-devrel)

#### The Roadmap columns

- `icebox` contains epics which require further specification or are specifically paused.
- `backlog` contains epics which are well specified but haven't been given any
  priority to be worked on during the current sprint.
- `next` column contains epics which are given priority to be worked on during
  the current sprint. The priorities are descending from top to bottom. Priorites
  with hard deadlines must be marked with the label
  [deadline](https://github.com/hoprnet/hoprnet/labels/deadline) with more
  information on the deadline being available within the issue's description.
- `acceptance` column contains epics which were completed but require
  acceptance testing from an additional team member or outside person. When moving
  issues into `acceptance` the person who's input is required must be pinged
  directly.
- `done` column contains epics which were accepted. The column is cleaned up
  as part of the `Task Grooming` phase.

_Who:_ [Trifecta](#trifecta)

_When_: first day of a sprint

### Sprint Planning

The task planning follows the `Prioritization` and takes the priorities into
account as much as possible.

- Tasks and PRs which are actively worked on are gathered on the [Taskboard](https://github.com/orgs/hoprnet/projects/15/views/10).
- Epic issues are further refined into tasks which have clear definitions of work and done.
- Ideally each team member has only one task assigned which is in status `next`.
- members which are coming back from vacation should already have tasks assigned to them.

_Who:_ all tech members within a meeting

_When:_ first day of the sprint

### Fire alarm

It's possible that throughout the planned sprint, we encounter a bag of issues that need to be resolved ASAP.

1. As soon issue is detected, a [representantive](#representantives) needs to take up the task of coordinating how the issue is tackled
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
- All PRs must include the relevant tests required to ensure updated code works as expected.
- All PRs must include the documentation changes which will reflect the updated codebase.

### Issue creation

All HOPR repositories must support at most these issue templates, more templates may be added which make sense for each specific repository.

- [Epic Template](./.github/ISSUE_TEMPLATE/epic.md)
- [Bug Template](./.github/ISSUE_TEMPLATE/bug.md)
- [Feature Template](./.github/ISSUE_TEMPLATE/feature.md)
- [Custom Template](./.github/ISSUE_TEMPLATE/custom.md)

- A _tech_ member, or an external contributor creates an issue using one of our templates, these templates may add some labels by default.
- All new issues created by external contributors will contain a `new issue` label.
- A _tech_ member may review the new issue and label it accordingly, see [#labels](#labels) for descriptions on the standard labels.
  - Some repositories may contain their own unique labels.
- Once a new issue is reviewed, the label 'new issue' shall be removed and the issue may be updated with any other relevant labels.

### Labels

All HOPR repositories must support at least these labels, more labels may be added which make sense for each specific repository.
Since PRs should link to an issue, it's not necessary to label PRs. However, some automation tools like `dependabot` doesn't create issues, in cases like this, it will label it's PRs.

| Name              | Description                                             | Color                                           |
| ----------------- | ------------------------------------------------------- | ----------------------------------------------- |
| epic              | An issue which tracks multiple issues                   | <span style="background:#2473b7">#2473b7</span> |
| bug               | Something isn't working                                 | <span style="background:#d73a4a">#d73a4a</span> |
| feature           | New feature request                                     | <span style="background:#a2eeef">#a2eeef</span> |
| needs information | Further information is requested                        | <span style="background:#d876e3">#d876e3</span> |
| new issue         | **Automatic**, this is a new issue                      | <span style="background:#000000">#000000</span> |
| dependencies      | **Automatic** for **PRs**, this PR updates a dependancy | <span style="background:#0366d6">#0366d6</span> |

### Rules

- All PR‘s **have to be approved** by at least two members.
- Respect our CI pipeline. If you break it, you fix it.
- All PR‘s must pass all status checks/tests before merging.
- Releases can be merged back to `master`, but not always necessary.
- When in conflict, chat and engage with the team.

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

#### Releases

See [Release Processes](./release.md)

### Branches

- `master`: In our case, `master` is a **prerelease** branch - tests _must_
  pass, and code _should_ be stable, but its _acceptable_ to have issues.

- `release/**`: On new internal release, we cut a `release/**`
  branch, using an internal **city** name to identify the release.
  See [Release Processes](./release.md) for more info.
