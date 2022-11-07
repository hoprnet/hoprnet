# Phase 1 testing

This document describes the process of Phase 1 testing in detail.
For the overall overview of individual testing phases, please refer to the 
[release process](release.md).

## Basic overview

- Phase 1 Testing involves the *CORE* team only.
- *Release Person (RP)* = *CORE* team member responsible for the build & releasing of the new release

1. Meeting is scheduled by the RP for 1 hour max and consists of two Stages (30 mins each):

   - **Stage 1 (fixed form)**: always the same set of Basic Tests (see below)

   - **Stage 2 (free form)**: devoted to test various new features/bug-fixes of the release and should be defined before the meeting (by RP).

2. Whoever encounters a bug during testing is encouraged to create a bug report (including log output) after the meeting is over.

3. If both testing Stages are successful, the team decides to proceed the release to Phase 2.

4. If any test in any Stage fails, the team decides whether the session is over and bug fixes are needed first to proceed.

5. As soon as bug fixes are done, a new testing session is scheduled by the RP and testing should start from the beginning, regardless in which Stage it failed last time.

## Preparation

### Release Person
- Ensures GCP nodes are running and up-to-date (public nodes, NAT nodes, topology nodes)
- Allocates timeframe of 1 hour for team testing
invites all CORE members to calendar event.
- Adds agenda to event, i.e. defines what the Stage 2 of the meeting will be (i.e. what new features/bugfixes need testing)
- Everybody in CORE team is involved in testing (including RP)

### Every tester
1. start & fund at least 1 NAT node
2. (optional) also start & fund a public node (if they have the possibility)
3. document all nodes on the Notion page
4. wait some time before the node synchronizes with the network. Therefore everyone should finish their preparations and have the node started at least 15-20 mins before the meeting.

## Basic tests
Basic tests are performed Stage 1 to verify basic functionalities of the product. If any of these tests fail at any stage, the team decides if the testing is over until the bugs are fixed.

1. Pick a public relay node and ping it
2. Open channel to the public relay node
3. Send 1-hop message to a public relay (e.g. `"Hi from X to public node!"`) and verify arrival
4. Send 1-hop message back from the public relay node
5. Ping NAT nodes of all other team members
6. Send 1-hop message to all other NAT nodes of team members (e.g. `"Hi from X to Y!"`,...)

### Channel testing

1. Decide on a closed loop of team members

2. Open 1 channel to 1 other team member (Upstream Buddy). The goal is to create a closed loop:

Example: `X -> Y -> Z -> A -> B -> X`

So for `X`:
- 1st Upstream Buddy is `Y`, 2nd Upstream Buddy is `Z`
- 1st Downstream Buddy is `A`, 2nd Downstream Buddy is `B`

Send 1-hop message to your 1st Upstream Buddy:
- `X` sends: `"Hi from X to Y"`
- `Y` sends: `"Hi from Y to Z"`


Send 2-hop message over your 1st Upstream Buddy to your 2nd Upstream Buddy, e.g.:
- `X` sends: `"Hi from X to Z (via Y)"`
- `Y` sends: `"Hi from Y to A (via Z)"`

Send 3-hop message to your 3rd level Buddy over your 1st and 2nd level Buddy, e.g.:
- `X` sends: `"Hi from X to A (via Y, Z)"`
- `Y` sends: `"Hi from Y to B (via Z, A)"`

At the end of Channel testing, everyone should have 2 messages from their Downstream Buddies:
So X should see:

`"Hi from A to X (via B)"`
`"Hi from Z to X (via A, B)"`