# Release Processs

The purpose of this document is to streamline the releases of hoprd.

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
  - [Create Release](#create-release)
  - [Create Patch](#create-patch)
  - [Merge Back](#merge-back)

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

- A _Public release_ occurs when an \_Internal release is promoted, see [Release promotion](#release-promotion).
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

### Create Release

1. Create a [release](https://github.com/hoprnet/hoprnet/issues/new?assignees=hoprnet%2Fhopr-tech-representatives&labels=release&projects=&template=release.md&title=Release%3A+%5BRELEASE+NAME%5D) issue for tracking scope. Use previous issues as [templates](https://github.com/hoprnet/hoprnet/issues/4487)
2. Create new PR to merge into master to prepare the release and include the following changes in it
````
export RELEASE_NAME=providence
git checkout master
git pull
git checkout -b feature/prepare-release-${RELEASE_NAME}
git push --set-upstream origin feature/prepare-${RELEASE_NAME}
gh pr create --title "Prepare release ${RELEASE_NAME}" --base master --draft -w --body "The scope of this PR is to prepare the contents of the release"

````

The contents of the PR should be:

- Check that there is an entry in `packages/hoprd/releases.json` for the new release name.
   - If the release will run in its own network ($RELEASENAME == $NETWORK) then a new entry in `packages/core/protocol-config.json` should be created for the network.
   - If the release will run in a multinetwork network like `dufour` then update the file `packages/core/protocol-config.json` for the `dufour` entry to accept the new `version_range` of the new release.
- Change all occurences of the last release name to the new release name within documentation files and Docker files. Don't touch the `protocol-config.json` and `releases.json` files in this step.
- Update `CHANGELOG.md` with the new release's information.
- Release owner checks if docs are correctly updated by comparing with the changes in `CHANGELOG.md`.
- If the release will run in a new network then, check that the entry `networks` in `contracts-addresses.json`, contains its own network

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

3. Check before merging the PR

- Modify the [environment variables](https://github.com/hoprnet/hoprnet/settings/variables/actions) `RELEASE_PR` with the number of PR just created above
- Modify the [environment variables](https://github.com/hoprnet/hoprnet/settings/variables/actions) `RELEASE_CANDIDATE_NUMBER` (Number of the release candidate to create) accordingly.
- Wait for the docker images builds to finish.
- Get the approval from at least 2 members
- Merge the PR
- Wait until the merge pipeline finishes.

4. Create release branch:

```
git checkout master
git pull
git checkout -b release/${RELEASE_NAME}
git push --set-upstream origin release/${RELEASE_NAME}
git tag ${RELEASE_NAME}
git push origin ${RELEASE_NAME}
```

5. Create the cluster of hoprd nodes in GCP. Make sure that the nodes are monitored in Grafana

```
./scripts/create-identity.sh dufour 10
mv ./identities/identity-XX ./identities/identities-core-dufour-gcp
./scripts/setup-gcloud-cluster.sh dufour dufour-providence 10
./scripts/setup-hoprd-nodes.sh dufour-providence ./identities/identities-core-dufour-gcp
```

6. Create a release page in the wiki (Notion) at: https://www.notion.so/Testnets-e53255f7003f4c8eae2f1b6644a676e0
   You may use previous testnet pages as templates. Ensure all started nodes are documented.
7. Share the links to the release tracking issue, tracking PR and testnet wiki page in the `#release` Element channel.
   On the `#testing` channel, members are expected to run their own nodes (either AVADO or via their workstation) to participate in the release.


### Create Patch

Bug fixing will be done by creating PR pointing to branch `release/${RELEASE_NAME}`. Once there are enough bugs or there is a need to deliver a blocking issue fixed, the release patching process starts. Before starting the release patching the process of alpha testing should start. 
If the testing is successful, then follow the following steps:

- Create a PR to prepare the release patching
````
echo PATCH_NUMBER=2
git pull
git checkout release/${RELEASE_NAME}
git pull
git checkout -b feature/prepare-${RELEASE_NAME}-patch
./scripts/bump-version.sh release/providence ${PATCH_NUMBER}
git add .
git commit -m "Preparing the patch ${RELEASE_NAME}-rc.${PATCH_NUMBER}"
git push --set-upstream origin feature/prepare-${RELEASE_NAME}-patch
gh pr create --title "Preparing the patch ${RELEASE_NAME}-rc.${PATCH_NUMBER}" --base release/${RELEASE_NAME} -w --body "The scope of this PR is to prepare the contents of the patch"

````
- Wait for the PR pipeline checks to finish correctly. E2E tests included
- Get the approval
- Merge the PR
- Tag the release 

````
git tag ${RELEASE_NAME}-rc.${PATCH_NUMBER}
git push origin ${RELEASE_NAME}-rc.${PATCH_NUMBER}

````

### Merge Back

1. Perform the following steps
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
      git checkout --theirs packages/$package/package.json
      git add packages/$package/package.json
    else
      echo "Review changes manully for ./packages/$package/package.json"
    fi
  done
  echo "By default to resolve the confllicts in package.json files it would be choosing for the 'Accept incomming change' option"
  git status

  git commit -m "Merge branch 'master' into merge-back-release-${RELEASE_NAME}"
  git push --set-upstream origin merge-back-release-${RELEASE_NAME}
  gh pr create --title "Merge back from ${RELEASE_NAME} 1" --base master -w --body "The scope of this PR is to merge back to master all the bug fixing found in release ${RELEASE_NAME}"
```
Note: In case of conflicts in any chain specific file, changes from `master` have preference 

2. Modify the above created PR to add reviewers, and labels accordingly. 
3. Remind that the release must be merged-back every week (Friday) to minimise conflicts whenever we want to merge a hotfix back to master.
