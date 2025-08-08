# GitHub Workflows

This repository uses several GitHub Actions workflows to automate tasks such as building, testing, and releasing the code. Here's a brief overview of each workflow:

## Pull request lifecycle

### Open

When a new pull request is created it is converted by a [workflow](./new-pr.yaml) into Draft mode to avoid executing PR workflows when there is still unfinished work.

### Ready for review

When the developer changes the PR status to **Ready for review**, then the following workflows start running to check compliance with acceptance criteria:

- [Build](./build.yaml)
- [Tests](./tests.yaml)
- [Checks](./checks.yaml)
- [Docs](./build-docs.yaml)

### Additional actions

There are some manual actions that a developer can trigger on a PR:

- [Deploy hoprd](./k8s.yaml): By attaching label **deploy_nodes**, the workflow will deploy this specific PR commit into a new created hoprd node.
- [Build binaries](./build-binaries.yaml): By attaching label **binary:??-??**, the workflow will generate a binary for that specific platform attached to the PR so it can be manually tested.

* [Build binaries](./build-binaries.yaml): By attaching label **binary:<platform>**, the workflow will generate a binary for that specific platform attached to the PR so it can be manually tested. For example, **binary:linux-amd64**.

- [Build dappnode](./build-dappnode.yaml): By attaching label **package:DAppNodePackage-Hopr** or **package:DAppNodePackage-Hopr-testnet**, the workflow will generate a dappNode package for that specific environment attached to the PR so it can be manually tested.

## Promote Release Workflow

The promote release workflow is manually triggered and is used to create or update a release tag, branch, and artifacts based on the recently closed release. This workflow is defined in [promote-release.yaml](./promote-release.yaml).

## Close Release Workflow

The close release workflow is manually triggered and is used to close a release. It ensures that all the issues in the milestone are closed and creates a new PR named `Close release <RELEASE_NUMBER>`. This workflow is defined in [close-release.yaml](./close-release.yaml).

## Open Release Workflow

The open release workflow is triggered after the close release workflow. It creates a new PR named `Open release <RELEASE_NUMBER>`. This workflow is defined in [open-release.yaml](./open-release.yaml).

## Update DAppNode Release Workflow

The update DAppNode release workflow is triggered after the open release workflow. It creates a new PR named `Update to release <RELEASE_NUMBER>` on the DAppNode repository. This workflow is defined in [generate-dappnode-pr.yaml](./generate-dappnode-pr.yaml).

Please refer to the individual workflow files for more detailed information about each workflow.

## Generate SDK Python Workflow

The Generate SDK Python workflow is triggered after the open release workflow. It generates a Python SDK for the current release and creates a new PR named `Update Python SDK for release <RELEASE_NUMBER>`. This workflow is defined in [generate-sdk.yaml](./generate-sdk.yaml).

Please refer to the individual workflow file for more detailed information about this workflow.

## Load Testing Workflow

The Load Testing workflow is triggered manually. It is used to run load tests against the current release to ensure that it can handle the expected load. This workflow is defined in [load-tests.yaml](./load-tests.yaml).

Please refer to the individual workflow file for more detailed information about this workflow.
