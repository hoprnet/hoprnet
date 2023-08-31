# Release Processs

The purpose of this document is to streamline the releases of hoprd.



# Release Process

The purpose of this process is to streamline the releases of HOPR packages.

  - [Release Types](#types-of-release)
    - [Internal release](#internal-release)
    - [Public release](#public-release)
    - [Milestone based releases](#milestone-based-releases)
    - [Deadline based releases](#deadline-based-releases)
  - [Testing phases](#testing-phases)
    - [Alpha testing](#alpha-testing)
    - [Beta testing](#beta-testing)
    - [Release candidate testing](#release-candidate-testing)
    - [User acceptance testing](#user-acceptance-testing)
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

## Release Types

### Internal release

- All releases start as an _Internal release_ and may be promoted to a _Public release_, see [Release promotion](#release-promotion).
- Internal releases are more frequent and their goal is to test new features and bug fixes.
- In an _Internal release_ there should be no public involvement.
- When an internal release is created, one member of the development team is assigned to oversee the launch of release, this includes:
  - Notifies the HOPR team that the _Internal release_ has been created in element channel `releases`
  - [Creating release](#release-cycle)
  - [Testing phases](#testing-phases)
  - [Release promotion](#release-promotion)

### Public release

- A _Public release_ occurs when an _Internal release is promoted, see [Release promotion](#release-promotion).
- Public releases are less frequent and their goal is to showcase new features and bug fixes.

### Milestone based releases

At the end of a sprint, if a sufficient amount of features / bug fixes were implemented since last release, the [Trifecta](./development.md#trifecta) may queue and prioritize a new release for the **upcoming** sprint, this happens during [epic prioritization](./development.md#epic-prioritization).
This new release is considered an [Internal release](#internal-release) and may be [promoted](#release-promotion) to a [Public release](#public-release) if [testing phases](#testing-phases) are successful.

### Deadline based releases

Deadline based releases are releases which have a defined deadline and scope agreed between the [Trifecta](./development.md#trifecta) and `founders`.
These releases should be of a large scope and occur only on occasions where a release is necessary.

## Testing phases


### Alpha testing

These are the keypoints of this testing phase:
- This testing effort is owned by the Core team, meaning that all the Core team members should participate in the testing
- Identify those bugs that are considered blockers and would prevent the other testing phases to even start or progress.
- Start the testing before an _Internal release_ is created to avoid unnecessary releases.
- Capture low-hanging bugs which are easily detectable.
- Test new features and bug fixes included in the release.
- Asses that the `latest` docker image published in the Docker registry is stable enough to be released.
- The release owner schedules a meeting where only the Core team members are invited
- During the online testing sesion, the Core team executes a manual regresion suite of test cases.
- Depending on the results of the session, a decission is taken to release the code as it is or not.
- In case some blocking bugs are found, the Core team should adapt the sprint plan and include them. A new alpha testing session must be needed once they are fixed

### Beta testing

These are the keypoints of this testing phase:
- This testing effort is owned by the Engineering team, meaning that all the engineering team members should participate in the testing.
- It always occurs after a successful alpha testing phase and when the release owner has created the _Internal release_.
- The release owners compiles the change log of the release that will help the testers to understand the new features and bug fixes that are bundled with it.
- The devops team needs to deploy the new release in the required environments.
- Once it is deployed, the Engineering team might start using this release to assess its stability.
- A coordinator must be chosen to lead the tests and collect all the issues.
- If the Engineering team identify relevant issues, it might require the Core team to create a new patch before continuing with more testing.

### Release candidate testing

These are the keypoints of this testing phase:
- This testing effort is owned by the Com Team
- It always occurs after a successful beta testing phase.
- A coordinator must be chosen to lead the tests and collect all the issues.
- The Com Team members install the release in their own devices
- The Com Team must asses if this release is stable enough or not.
  - Stable: The _Internal release_ is promoted to become _Public release_.
  - Unstable: Requests the Core Team to patch the internal release with the issues found.

### User acceptance testing

These are the keypoints of this testing phase:
- This testing effort is owned by the Com Team
- It always occurs after a successful beta or release candidate testing phase.
- A coordinator must be chosen to lead the tests and collect all the issues.
- The scope of this testing effort is to test the release with the devices owned by the ambassadors

## Release promotion

- All releases start by being internal releases.
- An [internal release](#internal-release) may be promoted to a [public release](#public-release) when all [testing phases](#testing-phases) are successful.
- Before promoting, release owner ensures that comm team actually needs this to be public.
- An [internal release](#internal-release) is promoted to a [public release](#public-release) by tagging its binaries with the public facing release name.
- Once promoted, a release owner notifies the HOPR team that the [internal release](#internal-release) has been promoted to a [public release](#public-release):
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
   hotfix/bug-2         hotfix/bug-1         release/providence           master

         x                   x                       x  create new release     x
         x                   x                       x◄─────────────────────   x 2.0.0
         x                   x   start hotfix bug-1  x 2.0.0-rc1 / providence  x 2.0.1
         x                   x ◄─────────────────────│                         x
         x                   │                       │                         x
         x                   │   hotfix merge  1     │  backport pr bug-1      x 
         x                   ▼ ─────────────────────►x ──────────────────►     x
         x                   x                       │                         x
         x  start fix bug-2  x                       │                         x
         x◄──────────────────x───────────────────────│                         x
         │                   x                       │                         x
         │  hotfix merge 2   x                       │  backport pr bug-1      x
         ▼───────────────────x──────────────────────►│ ───────────────────►    x
         x                   x                       │                         x
         x                   x                       │                         x
         x                   x                       │                         x
         x                   x                       x 2.0.0-rc2/providence    x
         x                   x                       │                         x
         x                   x                       │                         x
         x                   x                       │                         x
         x                   x                       │                         x

```

1. Setup some environment variables:

- Give a name to the release. For instance : `export RELEASE_NAME=providence`.
- Give a name to the previous release: `export OLD_RELEASE_NAME=bratislava`
- Give a name to the target network of the release. For instance: `export NETWORK=dufour`

2. Create a release tracking issue on GitHub. Use previous issues as [templates](https://github.com/hoprnet/hoprnet/issues/4487)
3. On the `master` branch, and before the creation of the release branch, there should be an entry in `packages/hoprd/releases.json` for the new release name.

- If the release will run in its own network ($RELEASENAME == $NETWORK) then a new entry in `packages/core/protocol-config.json` should be created for the network.
- If the release will run in a multinetwork network like `dufour` then update the file `packages/core/protocol-config.json` for the `dufour` entry to accept the new `version_range` of the new release.
- Create a PR and merge it into master

4. On the `master` branch, create the release branch locally by executing `git checkout -b release/${RELEASE_NAME}`.
5. On the `release/${RELEASE_NAME}` branch, and before pushing the branch to GitHub, some release-specific changes should be applied to ensure the resulting CD artifacts actually are proper release artifacts.

   - Change all occurences of the last release name to the new release name within documentation files and Docker files. Don't touch the `protocol-config.json` and `releases.json` files in this step. Changes should be committed locally.
   - Update `CHANGELOG.md` with the new release's information. Changes should be committed locally.
   - Release owner checks if docs are correctly updated by comparing with the changes in `CHANGELOG.md`.
   - If the release will run in a new network then, copy create a network entry under the `networks` in `contracts-addresses.json`, like

   ```
    "new_network": {
      "boost_contract_address": "",
      "channels_contract_address": "",
      "environment_type": "production",
      "indexer_start_block_number": 0,
      "network_registry_contract_address": "",
      "network_registry_proxy_contract_address": "",
      "stake_contract_address": "",
      "stake_season": 7,
      "token_contract_address": "",
      "xhopr_contract_address": ""
    }
   ```

   NOTE: Don't include the deployment of HoprChannels, because this will be re-deployed anyway by the CD system.
   Changes should be committed locally.
6. On github UI, clone the `master` branch to the `release/${RELEASE_NAME}` branch
7. On the `release/${RELEASE_NAME}` branch, check that everything is ready and push it to GitHub by executing : `git push origin`. Create a PR to be merge to `release/${RELEASE_NAME}`. Wait until the PR merge workflow is finished.
8. Create a Pull Request for tracking the release changes against the `master` branch. Remark in the PR description that it should never be merged!. Also use the label `DO NOT MERGE`, `release` and `release/${RELEASE_NAME}`. As a reference take a look at https://github.com/hoprnet/hoprnet/pull/4311
9. Create the cluster of hoprd nodes in GCP
````
./scripts/create-identity.sh dufour 10
mv ./identities/identity-XX ./identities/identities-core-dufour-gcp
./scripts/setup-gcloud-cluster.sh dufour dufour-providence 10
./scripts/setup-hoprd-nodes.sh dufour-providence ./identities/identities-core-dufour-gcp
````
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
   ```
    git checkout release/${RELEASE_NAME}
    git pull
    git checkout staging/${RELEASE_NAME}
    git pull
    git checkout -b release-upgrade-${RELEASE_NAME}
    git merge release/${RELEASE_NAME}
   ```
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

```
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

  echo "Resolving clonficts on Vscode for the package.json it would be similar to perform a 'Accept incomming change'"

  git status

  git commit -m "Merge branch 'master' into merge-back-release-${RELEASE_NAME}"
  git push --set-upstream origin merge-back-release-${RELEASE_NAME}
```

4. Modify the above created PR to add reviewers, and labels accordingly. Wait for the review before merge the `merge-back-release-${RELEASE_NAME}` branch to `master`.
5. If the release runs in a new environment, then redeploy `api.hoprnet.org` in Vercel to pickup release specific changes from the `protocol-config.json`.
6. Remind that the release must be merged-back every week (Friday) to minimise conflicts whenever we want to merge a hotfix back to master.

