# Release Processes

The HOPR Association _tech_ members and [Project Owner](./development.md#legend) have agreed on the following release processes to streamline the releases of HOPRd.

- [Release Processes](#release-processes)
  - [Types of release](#types-of-release)
    - [Internal release](#internal-release)
    - [Public release](#public-release)
  - [When to release](#when-to-release)
    - [Milestone based releases](#milestone-based-releases)
    - [Deadline based releases](#deadline-based-releases)
  - [Testing phases](#testing-phases)
    - [Tech team testing](#tech-team-testing)
    - [HOPR team testing](#hopr-team-testing)
    - [Ambassador testing](#ambassador-testing)
  - [Release promotion](#release-promotion)
  - [On a new chain](#on-a-new-chain)
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

## Types of release

| Type     | Goal                                                   | Ambassadors | Public |
| -------- | ------------------------------------------------------ | ----------- | ------ |
| Internal | Test new features and bug fixes                        | MAYBE       | NO     |
| Public   | New HOPRd version, showcase new features and bug fixes | YES         | YES    |

### Internal release

- Internal releases are more frequent and their goal is to test new features and bug fixes.
- There should be no public involvement unless the [internal release](#internal-release) is promoted to a [public release](#public-release).
- All releases start as an [internal release](#internal-release) release and may be promoted to a [public release](#public-release), see [release promotion](#release-promotion).
- When an internal release is created, one of the [representatives](./development.md#representatives) is assigned to oversee the release cycle, this includes:
  - notifies the HOPR team that the [internal release](#internal-release) has been created in element channel `releases`
  - [creating release](#release-cycle)
  - [testing release](#testing-phases)
  - [promoting release](#release-promotion)

### Public release

- Public releases are less frequent and their goal is to showcase new features and bug fixes.
- A [public release](#public-release) release occurs when an [internal release](#internal-release) release is promoted to a [public release](#public-release), see [release promotion](#release-promotion).

## When to release

### Milestone based releases

At the end of a sprint, if a sufficient amount of features / bug fixes were implemented since last release, the [Trifecta](./development.md#trifecta) may queue and prioritize a new release for the **upcoming** sprint, this happens during [epic prioritization](./development.md#epic-prioritization).
This new release is considered an [internal release](#internal-release) and may be [promoted](#release-promotion) to a [public release](#public-release) if [testing phases](#testing-phases) are succesful.

### Deadline based releases

Deadline based releases are releases which have a defined deadline and scope agreed between the [Trifecta](./development.md#trifecta) and `founders`.
These releases should be of a large scope and occur only on occasions where a release is necessary.

## Testing phases

Testing phases occur only when a release is queued and prioritized during [epic prioritization](./development.md#epic-prioritization).

For every phase completed, release owner must update the release's PR with the current testing phase status.

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
- Launch and fund HOPRd, then follow release instructions.

## Release promotion

- All releases start by being internal releases.
- An [internal release](#internal-release) may be promoted to a [public release](#public-release) when all [testing phases](#testing-phases) are succesful.
- Before promoting, release owner ensures that community team actually needs this to be public.

An [internal release](#internal-release) is promoted to a [public release](#public-release) by tagging it's binaries with the public facing release name. See [Deployment checklist](#deployment-checklist).

Once promoted, a release owner notifies the HOPR team that the [internal release](#internal-release) has been promoted to a [public release](#public-release):

- by commenting into the release's epic
- by writing in element channel `releases`

## On a new chain

Every chain has it's own unique characteristics. While it may be EVM compatible, various other factors can make the chain incompatible with HOPR.
When an epic is prioritized to switch the default `HOPRd` chain to a new one, [this issue](../.github/ISSUE_TEMPLATE/new-chain-epic.md) must be added to the sprint as part of the new chain epic.

This requirement will become simpler once [#2140](https://github.com/hoprnet/hoprnet/issues/2140) is completed.

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

1. Create a release tracking issue on Github. Use previous issues as templates: https://github.com/hoprnet/hoprnet/issues/3044
2. (on `master`) As a preparation for a release there should be a respective entry in `packages/hoprd/releases.json` and if needed in `packages/core/protocol-config.json`. If the entries are missing, create and merge them before starting the actual release process. This step can be done way in advance to plan environments and releases, too.
3. (on `master`) Now create the release branch locally. E.g. doing `git checkout -b release/${RELEASE_NAME}`.
4. (on `release/${RELEASE_NAME}`) Before pushing the branch to Github, some release-specific changes should be applied to ensure the resulting CD artifacts actually are proper release artifacts.

   1. Change all occurences of the last release name to the new release name within documentation files and Docker files. Don't touch the `protocol-config.json` and `releases.json` files in this step. Changes should be committed locally.
   2. Change use of `master-goerli` in `packages/avado/Dockerfile` to the new release name. Changes should be committed locally.
   3. Update `CHANGELOG.md` with the new release's information. Changes should be committed locally.
   4. Copy contract deployment files from the old release. This can be done doing

   ```
   mkdir -p packages/ethereum/deployments/${RELEASE_NAME}/xdai
   cp packages/ethereum/deployments/${OLD_RELEASE_NAME}/xdai/* packages/ethereum/deployments/${RELEASE_NAME}/xdai/
   cp packages/ethereum/deployments/${OLD_RELEASE_NAME}/xdai/.chainId packages/ethereum/deployments/${RELEASE_NAME}/xdai/
   rm packages/ethereum/deployments/${RELEASE_NAME}/xdai/HoprChannels.json
   
   ```

   NOTE: Don't include the deployment of HoprChannels, because this will be re-deployed anyway by the CD system.

   Changes should be committed locally.

5. (on `release/${RELEASE_NAME}`) Now everything is ready and can be pushed to Github: `git push origin`. Wait until the deployment of the basic cluster has completed by the CD.
6. Create a release tracking PR which can be used to follow CD builds. However, the PR should never be merged! As a reference take a look at https://github.com/hoprnet/hoprnet/pull/3048
7. (on `release/${RELEASE_NAME}`) Start a topology cluster using the script mentioned at the end of this document.
8. Create a release testnet page in the wiki at: https://www.notion.so/Testnets-e53255f7003f4c8eae2f1b6644a676e0
   You may use previous testnet pages as templates. Ensure all started nodes are documented.
9. Share the links to the release tracking issue, tracking PR and testnet wiki page in the `#release` Element channel.
   On the `#testing` channel, members are expected to run their own nodes (either AVADO or via their workstation) to participate in the release.
10. Patches to the release are created via `hotfix/RELEASE_NAME/**` branches.
    Each of these merges will trigger a new release version, and re-build our infrastructure
    for that version. Upon successfullly testing a release
11. Once the first release version has been built and is running, the release branch should be merged-back into `master` once to trigger version upgrades on `master`. See the next section for details.

Once the release testing has concluded, or if any signifant amount of patches were applied to the release branch, the release branch should be merged back into `master` again.

#### Release merge-back

1. (on `release/${RELEASE_NAME}`) Create a PR branch off of the release branch: `git checkout -b merge-back-release-${RELEASE_NAME}`
2. (on `merge-back-release-${RELEASE_NAME}`) Merge `master` into the branch: `git merge master`.
   For an example of a merge-back, take a look at older releases: https://github.com/hoprnet/hoprnet/pull/2956
   In case of conflicts (which is expected) changes from `master` have preference in the following cases:
   1. Revert changes in `packages/avado/docker-compose.yml`
   2. Revert any chain specific changes.
   3. Revert changes made to Avado configuration files as part of the initial release creation.
3. merge the `merge-back-release-${RELEASE_NAME}` PR back to `master` (ask someone to review first)

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
- [ ] Deploy a set of `$release` cloud nodes for our testnet with a full topology connected to feed event data for our leaderboard using our **`topology` deployment script**.
- [ ] Verify the $release smart contract in the explorer platform, see [verification guide](../VERIFICATION_GUIDE.md).

##### Optional

- [ ] Create a DNS alias for each node (cloud, cover-traffic, topology), to be accessed via our `hoprnet.link` domain (e.g. ct-1-$release.hoprnet.link)
- [ ] Tag a distribution manually $release on npm and on Docker Hub.
  - `npm login`, `npm dist-tag add @hoprnet/hoprd@$version $mountain/$city`
  - `docker login`, `docker tag gcr.io/hoprassociation/hoprd:latest hopr/hoprd:$city/$mountain`

#### Per $chain

- [ ] Deploy HOPR token on $chain and mint 130M HOPR tokens for our Development Address `0x2402da10A6172ED018AEEa22CA60EDe1F766655C`.

- [ ] Transfer 1M HOPR token and 1 native $chain to our funding wallet in our CI/CD.

- [ ] Transfer 1M HOPR token and 20 native $chain to our leaderboard wallet in network.hoprnet.org.

### Scripts

#### `cover-traffic` deployment script

```
CT_PRIV_KEY=14e6...a6a5 \
  ./scripts/setup-ct-gcloud-cluster.sh prague
```

#### `topology` deployment script

```
HOPRD_PERFORM_CLEANUP=false \
FUNDING_PRIV_KEY=0xa77a...21b8 \
HOPRD_SHOW_PRESTART_INFO=true \
  ./scripts/setup-gcloud-cluster.sh prague `pwd`/scripts/topologies/full_interconnected_cluster.sh
```
