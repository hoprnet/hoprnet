This guide aim to explain the process of publishing a new HOPR package in the Dappnode public repository:

The Dappnode package is versioned differently in it's own repository: https://github.com/dappnode/DAppNodePackage-Hopr
Each Dappnode package version is tied to a certain upstream version in our Monorepo (As of now: 1.0.0 -> 1.90.12)

The current process is as follows when a new version needs to be published:

1. Clone the HOPR Dappnode repo from https://github.com/dappnode/DAppNodePackage-Hopr
2. Connect to your Dappnode VPN.
3. Make changes to `dappnode_package.json`: bump the `version` and change to the desired `upstreamVersion`.
4. Do a Dappnode build using: `dapnnodesdk build` This will build a new unsigned Dappnode package and push it to your IPFS node.
5. Commit all the changes in a new branch (e.g. `bump-upstream-1.90.x`) and create a new PR in the `DAppNodePackage-Hopr` repo.
6. Wait until the PR is approved and merged.
7. On GH page of the `DAppNodePackage-Hopr`, go to the newly created release and click the link that takes you to the pre-filled release signing form.
8. In Site setting for this page make sure you allow Insecure content (it usually does an HTTP request to your Dappnode while itself being behind HTTPS).
9. Make sure the developer address in the form corresponds to the `Dappnode HOPR Repository Owner`, change it if not.
10. In MetaMask, load the `Dappnode HOPR Repository Owner` wallet on Mainnet and click `Connect MetaMask` in the form.
11. Click `Sign release` to sign the release (confirm in MM). New IPFS hash is created with the signed release, changes automatically in the Release hash field in the form.
12. Click `Publish release` to publish the signed release (confirm transaction in MM).
