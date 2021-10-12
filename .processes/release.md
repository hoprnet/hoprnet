# Release Processes

The HOPR Association _tech_ members and [Project Owner](./development.md#legend) have agreed on the following release processes to streamline the releases of HOPRd.

- [Release Processes](#release-processes)
  - [Types of release](#types-of-release)
    - [Internal release](#internal-release)
    - [Public release](#public-release)
  - [When release](#when-release)
    - [Milestone based releases](#milestone-based-releases)
    - [Deadline based releases](#deadline-based-releases)
  - [Testing phases](#testing-phases)
    - [Tech team testing](#tech-team-testing)
    - [HOPR team testing](#hopr-team-testing)
    - [Ambassador testing](#ambassador-testing)
  - [Release promotion](#release-promotion)
  - [On a new release](#on-a-new-release)
    - [Release Cycle](#release-cycle)
    - [Actions](#actions)
      - [Pre-release Version Bump (`feature` -> `master` = `x.y.z-0.next.*` -> `x.y.z-0.next.* + 1`)](#pre-release-version-bump-feature---master--xyz-0next---xyz-0next--1)
      - [Release Version Bump (`master` -> `release/**` = `x.y.z-0.next.*` -> `x.y.0`)](#release-version-bump-master---release--xyz-0next---xy0)
      - [Minor Version Bump (`release/**` -> `master` = `x.y.*` -> `x.y+1.0-next.0`)](#minor-version-bump-release---master--xy---xy10-next0)
    - [Deployment checklist](#deployment-checklist)
      - [Per $release](#per-release)
    - [Per $chain](#per-chain)
    - [Scripts](#scripts)
      - [`cover-traffic` deployment script](#cover-traffic-deployment-script)
      - [`topology` deployment script](#topology-deployment-script)
  - [Migrating between releases](#migrating-between-releases)

## Types of release

| Type     | Goal                                                   | Ambassadors | Public |
| -------- | ------------------------------------------------------ | ----------- | ------ |
| Internal | Test new features and bug fixes                        | MAYBE       | NO     |
| Public   | New HOPRd version, showcase new features and bug fixes | YES         | YES    |

### Internal release

- Internal releases are more frequent and their goal is to test new features and bug fixes.
- There should be no public involvement unless the [internal release](#internal-release) is promoted to a [public release](#public-release).
- All releases start by being an [internal release](#internal-release) release and may be promoted to a [public release](#public-release), see [release promotion](#release-promotion).

### Public release

- Public releases are less frequent and their goal is to showcase new features and bug fixes.
- A [public release](#public-release) release occurs when an [internal release](#internal-release) release is promoted to a [public release](#public-release), see [release promotion](#release-promotion).

## When release

### Milestone based releases

At the end of a sprint, if a sufficient amount of features / bug fixes were implemented since last release, the [Trifecta](./development.md#trifecta) may queue and prioritize a new release for the **upcoming** sprint, this happens during [epic prioritization](./development.md#epic-prioritization).
This new release is considered an [internal release](#internal-release) and may be [promoted](#release-promotion) to a [public release](#public-release) if [testing phases](#testing-phases) are succesful.

### Deadline based releases

Deadline based releases are releases which have a defined deadline and scope decided by the [Trifecta](./development.md#trifecta).
These releases should be of a large scope and occur only on occasions where a release is necessary.

## Testing phases

Testing phases occur only when a release is queued and prioritized during [epic prioritization](./development.md#epic-prioritization).

| Phase name         | Description                                                  |
| ------------------ | ------------------------------------------------------------ |
| tech team testing  | First phase, testing by tech team members only               |
| HOPR team testing  | Second phase, testing by available HOPR team members         |
| ambassador testing | Third (optional) phase, testing with the help of ambassadors |

### Tech team testing

- Capture low-hanging bugs which are easily detectable.
- Test new features and bug fixes included in the release.
- Avoid taking resources from the HOPR team members.

### HOPR team testing

- Occurs after [Tech team testing](#tech-team-testing) is succesful.
- With the help of a [changelog](#release-cycle), test HOPRd.

### Ambassador testing

A third and final phase of testing is to include ambassadors.
This is optional in the possibility we want to gather more data points and/or a specific feature requires larger network topology.

- May occur after [HOPR team testing](#hopr-team-testing) is succesful.
- Launch and fund HOPRd, then turn on `promiscuous` mode.

## Release promotion

- All releases start by being internal releases.
- An [internal release](#internal-release) may be promoted to a [public release](#public-release) when all [testing phases](#testing-phases) are succesful.

## On a new release

To release, we do a code-freeze in our codebase by branching out a specific
release by naming the branch `release/*`. New patches are then merged on that
particular branch to deploy on every change.

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

### Deployment checklist

#### Per $release

The following are a series of manual tasks that are needed to be executed for the launch of a release. Ideally, we automate these entirely and delete this document in the future, but in the meantime, we'll have this document to keep track of these.

- [ ] Deploy a set of `$release` CT nodes for our testnet using our **`cover-traffic` deployment script**.

- [ ] Deploy a set of `$release` cloud nodes for our testnet to support DEADR and be used as relayers (to be removed after https://github.com/hoprnet/hoprnet/issues/2537 is fixed).

- [ ] Deploy a set of `$release` cloud nodes for our testnet with a full topology connected to feed event data for our leaderboard using our **`topology` deployment script**.

- [ ] Tag a distribution manually $release on npm and on Docker Hub.

- [ ] Create a DNS alias for each node (cloud, cover-traffic, topology), to be accessed via our `hoprnet.link` domain (e.g. ct-1-$release.hoprnet.link)

- [ ] Verify the $release smart contract in the explorer platform.

### Per $chain

- [ ] Deploy HOPR token on $chain and mint 130M HOPR tokens for our Development Address `0x2402da10A6172ED018AEEa22CA60EDe1F766655C`.

- [ ] Transfer 1M HOPR token and 1 native $chain to our funding wallet in our CI/CD.

- [ ] Transfer 1M HOPR token and 20 native $chain to our leaderboard wallet in network.hoprnet.org.

### Scripts

#### `cover-traffic` deployment script

```
CT_PRIV_KEY=14e6...a6a5 \
HOPRD_INFURA_KEY=51d4...caf6 \
HOPRD_PROVIDER=https://provider-proxy.hoprnet.workers.dev/matic_rio \
./scripts/setup-ct-gcloud-cluster.sh cover-traffic-node-01
```

#### `topology` deployment script

```
HOPRD_PERFORM_CLEANUP=false \
FUNDING_PRIV_KEY=0xa77a...21b8 \
HOPRD_INFURA_KEY=51d4...caf6 \
HOPRD_PROVIDER=https://polygon.infura.io/v3/51d4...caf6 \
HOPRD_TOKEN_CONTRACT="0x6F80d1a3AB9006548c2fBb180879b87364D63Bf7" \
HOPRD_SHOW_PRESTART_INFO=true \
./scripts/setup-gcloud-cluster.sh matic-testnet-01 gcr.io/hoprassociation/hoprd:latest `pwd`/scripts/topologies/full_interconnected_cluster.sh
```

## Migrating between releases

At the moment we DO NOT HAVE backward compatibility between releases.
We attempt to provide instructions on how to migrate your tokens between releases.

1. Set your automatic channel strategy to `MANUAL`.
2. Close all open payment channels.
3. Once all payment channels have closed, withdraw your funds to an external
   wallet.
4. Withdraw all funds from last release to a secure wallet.
5. Run `info` and take note of the **network name**.
6. Once funds are confirmed to exist in a different wallet, backup `.hopr-identity` and `.db` folders.
7. Launch new `HOPRd` instance using latest release, this will create new `.hopr-identity` and `.db` folders, observe the account address.
8. Only tranfer funds to new `HOPRd` instance if `HOPRd` operates on the **same network** as last release, you can compare the two networks using `info`.
