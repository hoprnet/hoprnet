# GitHub Workflows

This repository uses several GitHub Actions workflows to automate tasks such as building, testing, and releasing the code. Here's a brief overview of each workflow:

## Pull request lifecycle

### Open

When pull request are created they are analyzed by a [Open PR workflow](./open-pr.yaml) to assess basic convention.

On every commit of the PR, then the following workflows start running to check compliance with acceptance criteria:

- [Build](./build.yaml)
- [Tests](./tests.yaml)
- [Checks](./checks.yaml)

### Additional actions

There are some manual actions that a developer can trigger on a PR:

- [Build dappnode](./build-dappnode.yaml): By attaching label **package:DAppNodePackage-Hopr** or **package:DAppNodePackage-Hopr-testnet**, the workflow will generate a dappNode package for that specific environment attached to the PR so it can be manually tested.

## Close Release Workflow

The close release workflow is manually triggered and is used to close a release. This workflow is defined in [release.yaml](./release.yaml).

## Promote Release Workflow

The promote release workflow is manually triggered and is used to create or update a release tag, branch, and artifacts based on the recently closed release. This workflow is defined in [promote-release.yaml](./promote-release.yaml).

## Update DAppNode Release Workflow

The update DAppNode release workflow is triggered after the close release workflow. It creates a new PR named `Update to release <RELEASE_NUMBER>` on the DAppNode repository. This workflow is defined in [generate-dappnode-pr.yaml](./generate-dappnode-pr.yaml).

Please refer to the individual workflow files for more detailed information about each workflow.
