# Release Process

The purpose of this process is to streamline the releases of HOPR packages.

- [Release Process](#release-process)
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
- All releases start as an [internal release](#internal-release) and may be promoted to a [public release](#public-release), see [release promotion](#release-promotion).
- When an internal release is created, one of the [representatives](./development.md#representatives) is assigned to oversee the release cycle, this includes:
  - notifies the HOPR team that the [internal release](#internal-release) has been created in element channel `releases`
  - [creating release](#release-cycle)
  - [testing release](#testing-phases)
  - [promoting release](#release-promotion)

### Public release

- Public releases are less frequent and their goal is to showcase new features and bug fixes.
- A [public release](#public-release) occurs when an [internal release](#internal-release) is promoted to a [public release](#public-release), see [release promotion](#release-promotion).

## When to release

### Milestone based releases

At the end of a sprint, if a sufficient amount of features / bug fixes were implemented since last release, the [Trifecta](./development.md#trifecta) may queue and prioritize a new release for the **upcoming** sprint, this happens during [epic prioritization](./development.md#epic-prioritization).
This new release is considered an [internal release](#internal-release) and may be [promoted](#release-promotion) to a [public release](#public-release) if [testing phases](#testing-phases) are successful.

### Deadline based releases

Deadline based releases are releases which have a defined deadline and scope agreed between the [Trifecta](./development.md#trifecta) and `founders`.
These releases should be of a large scope and occur only on occasions where a release is necessary.

## Testing phases

Testing phases occur only when a release is queued and prioritized during [epic prioritization](./development.md#epic-prioritization).

For every phase completed, release owner must update the release's PR with the current testing phase status.

| Phase name         | Description                                                                           |
| ------------------ | ------------------------------------------------------------------------------------- |
| CORE team testing  | First phase, testing by CORE team members only (see [Phase 1 testing](./testing.md) ) |
| HOPR team testing  | Second phase, testing by available HOPR team members                                  |
| ambassador testing | Third (optional) phase, testing with the help of ambassadors                          |

### Tech team testing

- Capture low-hanging bugs which are easily detectable.
- Test new features and bug fixes included in the release.
- Avoid taking resources from the HOPR team members.

### HOPR team testing

- Occurs after [Tech team testing](#tech-team-testing) is successful.
- With the help of a [changelog](#release-cycle), test HOPRd.

### Ambassador testing

A third and final phase of testing is to include ambassadors.
This is optional in the possibility we want to gather more data points and/or a specific feature requires larger network topology.

- May occur after [HOPR team testing](#hopr-team-testing) is successful.
- Launch and fund HOPRd, then follow release instructions.

## Release promotion

- All releases start by being internal releases.
- An [internal release](#internal-release) may be promoted to a [public release](#public-release) when all [testing phases](#testing-phases) are successful.
- Before promoting, release owner ensures that comm team actually needs this to be public.

An [internal release](#internal-release) is promoted to a [public release](#public-release) by tagging its binaries with the public facing release name. See [Deployment checklist](#deployment-checklist).

Once promoted, a release owner notifies the HOPR team that the [internal release](#internal-release) has been promoted to a [public release](#public-release):

- by commenting into the release's epic
- by writing in element channel `releases`

## On a new chain

Every chain has its own unique characteristics. While it may be EVM compatible, various other factors can make the chain incompatible with HOPR.
When an epic is prioritized to switch the default `HOPRd` chain to a new one, [this issue](../.github/ISSUE_TEMPLATE/new-chain-epic.md) must be added to the sprint as part of the new chain epic.

This requirement will become simpler once [#2140](https://github.com/hoprnet/hoprnet/issues/2140) is completed.

## On a new release

Before a new release is started, a release owner is picked by the [Tech Representatives](./development.md#representatives).

To release, we do a code-freeze in our codebase by branching out a specific
release by naming the branch `release/*`. New patches are then merged on that
particular branch to deploy on every change.

### Release Cycle

```
   hotfix/patch-riga    staging/riga         release/riga           master

         x                   x                       x  create new release  x
         x                   x                       x◄─────────────────────x 1.92.0-next.44
         x                   x   create new staging  x 1.92.0               x
         x                   │ ◄─────────────────────│                      x
         x                   │                       │   first merge-back   x
         x                   │                       │ ──────────────────►  x 1.93.0-next.0
         x                   │ 1.92.0-next.0         │                      x
         x                   │                       │                      x
         x  start hotfix     ▼                       │                      x
         ┌◄──────────────────x                       │                      x
         │                   x                       │                      x
         │  hotfix merge     x                       │                      x
         ▼──────────────────►┐ 1.92.0-next.1         │                      x
         x                   │                       │                      x
                             │   release upgrade     │                      x
         x                   ▼──────────────────────►┐ 1.92.1               x
         x                   x   staging upgrade     │                      x
         x                   x ◄─────────────────────│                      x
         x                   x 1.92.1-next.0         │                      x
         x                   x                       │   next merge-back    x
         x                   x                       │ ──────────────────►  x

```

1. Setup some environment variables:

- Give a name to the release. For instance : `export RELEASE_NAME=riga`.
- Give a name to the previous release: `export OLD_RELEASE_NAME=bogota`
- Give a name to the target environment of the release. For instance: `export ENVIRONMENT_NAME=monte_rosa`

2. Create a release tracking issue on GitHub. Use previous issues as [templates](https://github.com/hoprnet/hoprnet/issues/4487)
3. On the `master` branch, and before the creation of the release branch, there should be an entry in `packages/hoprd/releases.json` for the new release name.

- If the release will run in its own environment ($RELEASENAME == $ENVIRONMENT_NAME) then a new entry in `packages/core/protocol-config.json` should be created for the network.
- If the release will run in a multienvironment network like `monte_rosa` then update the file `packages/core/protocol-config.json` for the `monte_rosa` entry to accept the new `version_range` of the new release.
- Create a PR and merge it into master

4. On the `master` branch, create the release branch locally by executing `git checkout -b release/${RELEASE_NAME}`.
5. On the `release/${RELEASE_NAME}` branch, and before pushing the branch to GitHub, some release-specific changes should be applied to ensure the resulting CD artifacts actually are proper release artifacts.

   - Change all occurences of the last release name to the new release name within documentation files and Docker files. Don't touch the `protocol-config.json` and `releases.json` files in this step. Changes should be committed locally.
   - Update `CHANGELOG.md` with the new release's information. Changes should be committed locally.
   - Release owner checks if docs are correctly updated by comparing with the changes in `CHANGELOG.md`.
   - If the release will run in a new environment then, copy contract deployment files from the old environment by executing these commands:

   ```
   mkdir -p packages/ethereum/deployments/${RELEASE_NAME}/xdai
   cp packages/ethereum/deployments/${ENVIRONMENT_NAME}/xdai/* packages/ethereum/deployments/${RELEASE_NAME}/xdai/
   cp packages/ethereum/deployments/${ENVIRONMENT_NAME}/xdai/.chainId packages/ethereum/deployments/${RELEASE_NAME}/xdai/
   rm packages/ethereum/deployments/${RELEASE_NAME}/xdai/HoprChannels.json
   ```

   NOTE: Don't include the deployment of HoprChannels, because this will be re-deployed anyway by the CD system.
   Changes should be committed locally.

6. Delete the topology VM instances of the ${OLD_RELEASE_NAME}

- Check `gcloud compute instance-groups managed list` for a list and delete the instance groups using

```sh
HOPRD_PERFORM_CLEANUP=true ./scripts/setup-gcloud-cluster.sh "${ENVIRONMENT_NAME}" "" "${OLD_RELEASE_NAME}-topology-1-91"
```

7. On the `release/${RELEASE_NAME}` branch, check that everything is ready and push it to GitHub by executing : `git push origin`. Wait until the [deployment of the cluster](https://github.com/hoprnet/hoprnet/actions/workflows/deploy.yaml) has finished successfully.
8. Create a Pull Request for tracking the release changes against the `master` branch. Remark in the PR description that it should never be merged!. Also use the label `DO NOT MERGE`, `release` and `release/${RELEASE_NAME}`. As a reference take a look at https://github.com/hoprnet/hoprnet/pull/4311
9. On the `release/${RELEASE_NAME}` branch, create a topology cluster using the [script](./release.md#topology-deployment-script) mentioned at the end of this document.
10. Create a release testnet page in the wiki at: https://www.notion.so/Testnets-e53255f7003f4c8eae2f1b6644a676e0
    You may use previous testnet pages as templates. Ensure all started nodes are documented.
11. Share the links to the release tracking issue, tracking PR and testnet wiki page in the `#release` Element channel.
    On the `#testing` channel, members are expected to run their own nodes (either AVADO or via their workstation) to participate in the release.
12. For details how patches are applied to the release, see the `Release patching` section below.
13. Once the first release version has been built and is running, the release branch should be merged-back into `master` once to trigger version upgrades on `master`. See [the next](./release.md#release-merge-back) section for details.

Once the release testing has concluded, or if any significant amount of patches has been applied to the release branch, the release branch should be merged back into `master` again.

#### Release patching

Whenever a patch to a release `${RELEASE_NAME}` is needed, it first needs to be tested in the Staging deployment. The Staging deployment
is specific per each release and runs within the same network environment as the Release. Once the fix is
considered stable enough (after testing in the Staging deployment), it can be merged into the Release branch (this is called Release upgrade).
The developers are encourage to batch the hotfixes in the Staging branch and minimize the number of Release upgrades.

If `staging/${RELEASE_NAME}` does not exist yet:

1. (on `release/${RELEASE_NAME}`) if `staging/${RELEASE_NAME}` branch does not exist yet, create it: `git checkout -b staging/${RELEASE_NAME}`
2. (on `staging/${RELEASE_NAME}`) create and push empty commit to trigger deployment: `git commit --allow-empty -m "Deploy staging ${RELEASE_NAME}" && git push -u origin staging/${RELEASE_NAME}`

To create a hotfix:

1. (on `staging/${RELEASE_NAME}`) create a hotfix branch `hotfix/fix-bug`
2. (on `hotfix/fix-bug`) add commits to fix the bug
3. Once the fix needs to be tested, create a PR of `hotfix/fix-bug` to `staging/${RELEASE_NAME}` and merge once peer-reviewed & approved.

#### Release upgrade

The Release upgrade is done, when the Staging deployment is considered stable, so it can be merged back to the Release.
Once the upgraded release is deployed, the Staging deployment must be updated as well from the Release branch (this is called Staging upgrade).

1. (on `staging/${RELEASE_NAME}`) create branch `release-upgrade-${RELEASE_NAME}`: `git checkout -b release-upgrade-${RELEASE_NAME}`
2. (on `release-upgrade-${RELEASE_NAME}`) merge `release/${RELEASE_NAME}` into the branch: `git merge release/${RELEASE_NAME}`.
   In case of a merge conflict, the changes from the `release-upgrade-${RELEASE_NAME}` branch take precedence. For the conflict on the package.json version attribute, it should be taken the one comming from the `release/${RELEASE_NAME}` branch which does not have the suffix `-next.X`.
3. Create a PR of `release-upgrade-${RELEASE_NAME}` and target to `release/${RELEASE_NAME}` and ask for a peer review.
   Each of such PR merges will trigger a new release version, and re-build our infrastructure.
   ````
    git checkout release/${RELEASE_NAME}
    git pull
    git checkout staging/${RELEASE_NAME}
    git pull
    git checkout -b release-upgrade-${RELEASE_NAME}
    git merge release/${RELEASE_NAME}
   ````
4. Wait for the CI to deploy upgraded Release. Then perform the following steps for Staging upgrade.
5. (on `staging/${RELEASE_NAME}`) create branch `staging-upgrade-${RELEASE_NAME}`: `git checkout -b staging-upgrade-${RELEASE_NAME}`
6. (on `staging-upgrade-${RELEASE_NAME}`) merge `release/${RELEASE_NAME}` into the branch: `git merge release/${RELEASE_NAME}`.
   In case of a merge conflict, the changes from `release/${RELEASE_NAME}` take precedence.
7. Create a PR of `staging-upgrade-${RELEASE_NAME}` and ask for a peer review (should be straight-forward).
   The merge of the PR will trigger re-build of the Staging infrastructure.

#### Release merge-back

1. On the `master` branch update the latest changes by executing: `git pull`
2. On the `release/${RELEASE_NAME}` branch, create a PR branch of the release branch: `git checkout -b merge-back-release-${RELEASE_NAME}`
3. On the `merge-back-release-${RELEASE_NAME}` branch, merge `master` into the branch: `git merge master`.
   For an example of a merge-back, take a look at older releases: https://github.com/hoprnet/hoprnet/pull/2956
   In case of conflicts (which is expected) changes from `master` have preference in the following cases:
   1. Revert changes in `packages/avado/docker-compose.yml`
   2. Revert any chain specific changes.
   3. Revert changes made to Avado configuration files as part of the initial release creation.
      In regards to version naming convention for the merge-back:
   - If it is the first merge-back, then the version number to be used should be the one being used in the release branch which does not have the suffix `-next.XX`.
   - If it is other merge-back, then the version number to be used should be the one being used in the master branch which it has the suffix `-next.XX`.
  `````
    git checkout master
    git pull
    git branch -D merge-back-release-${RELEASE_NAME}
    git checkout release/${RELEASE_NAME}
    git pull
    git checkout -b merge-back-release-${RELEASE_NAME}
    git merge master

    packages=(connect core-ethereum core cover-traffic-daemon ethereum hoprd real utils)
    for package in "${packages[@]}"; do
      changes=$(git diff packages/$package/package.json  | grep @@ | wc -l)
      if [ "$changes" -eq "1" ]; then
        echo "git checkout --theirs packages/$package/package.json"
        echo "git add packages/$package/package.json"
      else
        echo "Review changes manully for ./packages/$package/package.json"
      fi
    done

    echo "Resolving clonficts on Vscode for the package.json it would be similar to perform a \"Accept incomming change\" "

    git status

    git commit -m "Merge branch 'master' into merge-back-release-${RELEASE_NAME}"
    git push --set-upstream origin merge-back-release-${RELEASE_NAME}
   ````
4. Modify the above created PR to add reviewers, and labels accordingly. Wait for the review before merge the `merge-back-release-${RELEASE_NAME}` branch to `master`.
5. If the release runs in a new environment, then redeploy `api.hoprnet.org` in Vercel to pickup release specific changes from the `protocol-config.json`.
6. Remind that the release must be merged-back every week (Friday) to minimise conflicts whenever we want to merge a hotfix back to master.

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
  ./scripts/setup-ct-gcloud-cluster.sh athens
```

#### `topology` deployment script

```

export PRIVATE_KEY=
export FAUCET_SECRET_API_KEY=
export HOPRD_PASSWORD=open-sesame-iTwnsPNg0hpagP+o6T0KOwiH9RQ0
export HOPRD_API_TOKEN=^binary6wire6GLEEMAN9urbanebetween1watch^


HOPRD_PERFORM_CLEANUP=false \
HOPRD_SHOW_PRESTART_INFO=true \
./scripts/setup-gcloud-cluster.sh monte_rosa `pwd`/scripts/topologies/full_interconnected_cluster.sh ${RELEASE_NAME}-topology-1-91 gcr.io/hoprassociation/hoprd:${RELEASE_NAME} 6 ${RELEASE_NAME}-topology-1-91 true


```
