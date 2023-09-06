# Release Processs

The purpose of this document is to streamline the releases of hoprd.

- [Release Types](#types-of-release)
  - [Internal release](#internal-release)
  - [Public release](#public-release)
  - [Deadline based releases](#deadline-based-releases)
- [Testing phases](#testing-phases)
  - [Pre-release testing](#pre-release-testing)
  - [Release testing](#release-testing)
  - [Promotion testing](#promotion-testing)
  - [User acceptance testing](#user-acceptance-testing)
- [Release promotion](#release-promotion)
- [On a new chain](#on-a-new-chain)
- [On a new release](#on-a-new-release)
  - [Release Cycle](#release-cycle)
  - [Open Release](#open-release)
  - [Close Release](#close-release)
  - [Promote release](#promote-release)
  - [Create Patch](#create-patch)
  - [Merge Back](#merge-back)

## Release Types

The following types of releases describe the different characteristics that they have:
- A release can be _Internal release_ or _Public release_
- A release can be deadline based or not

### Internal release

- All releases start as an _Internal release_ and may be promoted to a _Public release_, see [Release promotion](#release-promotion).
- Internal releases are more frequent and their goal is to test new features and bug fixes.
- In an _Internal release_ there should be no public involvement.
- When an internal release is created, one member of the development team is assigned to oversee the launch of release

### Public release

- A _Public release_ occurs when an \_Internal release is promoted, see [Release promotion](#release-promotion).
- Public releases are less frequent and their goal is to showcase new features and bug fixes.

### Deadline based releases

Deadline based releases are releases which have a defined deadline and scope agreed between the [Trifecta](./development.md#trifecta) and `founders`.
These releases should be of a large scope and occur only on occasions where a release is necessary.

## Testing phases

### Pre-release testing

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
- In case some blocking bugs are found, the Core team should adapt the sprint plan and include them. A new pre-release testing session must be needed once they are fixed

### Release testing

These are the keypoints of this testing phase:

- This testing effort is owned by the Engineering team, meaning that all the engineering team members should participate in the testing.
- It always occurs after a successful pre-release testing phase and when the release owner has created the _Internal release_.
- The devops team needs to deploy the new release in the required environments.
- Once it is deployed, the Engineering team might start using this release to assess its stability.
- A coordinator must be chosen to lead the tests and collect all the issues.
- If the Engineering team identify relevant issues, it might require the Core team to create a new patch before continuing with more testing.

### Promotion testing

These are the keypoints of this testing phase:

- This testing effort is owned by the Com Team
- It always occurs after a successful release testing phase.
- A coordinator must be chosen to lead the tests and collect all the issues.
- The Com Team members install the release in their own devices
- The Com Team must asses if this release is stable enough or not.
  - Stable: The _Internal release_ is promoted to become _Public release_.
  - Unstable: Requests the Core Team to patch the internal release with the issues found.

### User acceptance testing

These are the keypoints of this testing phase:

- This testing effort is owned by the Com Team
- It always occurs after a successful release or promotion testing phase.
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

### Open release

The process of opening a release should be done at the begining of the release. Ideally should be the first task to do right after the clousure of the previous release, so the team can start contributing to this new release by adding their issues and PR to the given milestone.

1. Execute the manual workflow named [Open Release](https://github.com/hoprnet/hoprnet/actions/workflows/open-release.yaml) specifying the type of new release that you want to create and selecting the branch where you want to create it.
2. A new PR will be created with the name `Create release <RELEASE_NUMBER>`. Review, approve and merge it.

### Close release

The process of closing a release consists of building, tagging and publishing the given release of the branch. It does not perform any other action like bumping the version to the next release.

1. Make sure that the [milestone](https://github.com/hoprnet/hoprnet/milestones) issues are all close. Do not start the next step until all the issues and PR are closed.
2. Execute the manual workflow named [Close Release](https://github.com/hoprnet/hoprnet/actions/workflows/close-release.yaml) selecting the branch where you want to close it.
3. A new PR will be created with the name `Close release <RELEASE_NUMBER>`. Follow the instructions on the PR until merge it.
4. Review the contents of the new [Github Release](https://github.com/hoprnet/hoprnet/releases) created and modify accordingly
5. Share the links to the  Github release and testnet wiki page in the `#release` Element channel.

### Promote release

The process of promoting the named release (bratislava, providence, etc) consists of creating or updating a given ${RELEASE_NAME} tag and artifacts to the newly closed released. This process should be executed after the closure of release.

1. Update the named release
```
echo RELEASE_NAME=providence
echo RELEASE_NUMBER=2.0.0-rc.3
git pull
git checkout ${RELEASE_NUMBER}
git checkout -b release/${RELEASE_NAME}
git push --set-upstream origin release/${RELEASE_NAME}
git tag ${RELEASE_NAME}
git push origin ${RELEASE_NAME}
```

2. Open [GCP Console](https://console.cloud.google.com/artifacts/docker/hoprassociation/europe-west3/docker-images?orgonly=true&project=hoprassociation&supportedpurview=organizationIdhttps://console.cloud.google.com/artifacts/docker/hoprassociation/europe-west3/docker-images?orgonly=true&project=hoprassociation&supportedpurview=organizationId) and tag the images with the new tags ${RELEASE_NUMBER} with the tag ${RELEASE_NAME}

3. Create a release page in the wiki (Notion) at: https://www.notion.so/Testnets-e53255f7003f4c8eae2f1b6644a676e0
   You may use previous testnet pages as templates. Ensure all started nodes are documented.

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
