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
  - [Close Release](#close-release)
  - [Publish dappNode](#publish-dappnode)
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

### Close release

The process of closing a release consists of building, tagging and publishing the given release of the branch.

1. Make sure that the [milestone](https://github.com/hoprnet/hoprnet/milestones) issues are all close. Do not start the next step until all the issues and PR are closed.
2. Execute the manual workflow named [Close Release](https://github.com/hoprnet/hoprnet/actions/workflows/close-release.yaml) selecting the branch where you want to close it.
3. A new PR will be created with the name `Close release <RELEASE_NUMBER>`. Follow the instructions on the PR and merge it.
4. Review the contents of the new [Github Release](https://github.com/hoprnet/hoprnet/releases) created and modify accordingly
5. Review that the Element notification has been sent
6. A new PR will be created with the name `Open release <RELEASE_NUMBER>`. Follow the instructions on the PR and merge it.
7. A new PR will be created with the name `Update to release <RELEASE_NUMBER>` on [DAppNode repository](https://github.com/dappnode/DAppNodePackage-Hopr/pulls). Review and merge it

### Publish DappNode

The Dappnode package is versioned differently in its own repository: https://github.com/dappnode/DAppNodePackage-Hopr
We maintain a fork of that repository at https://github.com/hoprnet/DAppNodePackage-Hopr and the previous pipeline creates an automatic pull request on each release.
There should be a correspondence between the dappNode version an the upstream version of hoprd

#### Prerequisites

- In order to publish a new version on Dappnode the deployer needs to configure in its own machine the VPN to connect to dappNode
  Follow this guide: https://welcome.dappnode.io/vpn and https://docs.dappnode.io/user/product-manual/vpn
- Configure the Metamask account for Dappnode. The credentials are stored in Bitwarden under the name: `Dappnode Repo Owner Wallet`
- In Site setting for site `https://dappnode.github.io/` make sure you allow Insecure content (it usually does an HTTP request to your Dappnode while itself being behind HTTPS).

1. Open Metamask and switch to the Dappnode account. Check also that is connected to the Ethereum network (Mainnet).
2. Turn on your Dappnode, and connect your laptop to its WiFi
3. Connect to your Dappnode VPN: `System Preferences` -> `Network` -> `dAppNode Wireguard`
4. Access to the recently published release https://github.com/dappnode/DAppNodePackage-Hopr/releases, and click the link that takes you to the pre-filled release signing form.
5. Set the public ethereum address of the Metamask account into the form field named `Developer address`
6. In the form, click on `Edit Settings` and set the field `IPFS API URLs` with the value `http://ipfs.dappnode:5001/`
7. Click `Connect MetaMask` in the form.
8. Click `Sign release` to sign the release (confirm in MM). New IPFS hash is created with the signed release, changes automatically in the Release hash field in the form.
9. Click `Publish release` to publish the signed release (confirm transaction in MM).
10. On the forked repo https://github.com/hoprnet/DAppNodePackage-Hopr GH page, do a Sync of the `main` branches from the Upstream repo.

### Promote release

The process of promoting the named release (bratislava, providence, etc) consists of creating or updating a given ${RELEASE_NAME} tag, branch and artifacts based on the recently closed released. This process should be executed after the closure of release candidates only or the first minor version (X.X.0).

1. Execute the manual workflow named [Promote Release](https://github.com/hoprnet/hoprnet/actions/workflows/promote-release.yaml) specifying the name of the release and the tag you want to bind it
2. Create a release page in the wiki (Notion) at: https://www.notion.so/Testnets-e53255f7003f4c8eae2f1b6644a676e0
   You may use previous testnet pages as templates. Ensure all started nodes are documented.

### Merge Back

The process of merge back consists of bringing all the changes made in the release branch back to the master branch.

1. Execute the manual workflow named [Merge Back](https://github.com/hoprnet/hoprnet/actions/workflows/merge-back.yaml)
2. Review the new PR created by this workflow and approve it if the changes provided are correct.
3. Remind that the release must be merged-back periodically to minimise conflicts whenever we want to merge a hotfix back to master.
