This guide aim to explain the process of publishing a new HOPR package in the Dappnode public repository:

The Dappnode package is versioned differently in its own repository: https://github.com/dappnode/DAppNodePackage-Hopr
We maintain a fork of that repository at https://github.com/hoprnet/DAppNodePackage-Hopr
Each Dappnode package version is tied to a certain upstream version in our Monorepo (As of now: 1.0.8 -> 1.93.0)

The current process is as follows when a new version needs to be published:

1. Clone or pull the HOPR Dappnode repo from https://github.com/hoprnet/DAppNodePackage-Hopr
2. Checkout the `develop` branch and make sure `main` is merged into it.
3. Make changes to `dappnode_package.json`: bump the `version` and change to the desired `upstreamVersion`.
4. Bump the versions also in `docker-compose.yml`
5. Commit the changes to the `develop` branch and push the changes.
6. Create a PR from the `develop` branch of https://github.com/hoprnet/DAppNodePackage-Hopr to the `main` branch of https://github.com/dappnode/DAppNodePackage-Hopr
7. Wait until the PR is approved and merged.
8. Connect to your Dappnode VPN following this guide: https://welcome.dappnode.io/vpn and https://docs.dappnode.io/user/product-manual/vpn
9. On GH page of https://github.com/dappnode/DAppNodePackage-Hopr, go to the newly created release and click the link that takes you to the pre-filled release signing form.
10. In Site setting for this page make sure you allow Insecure content (it usually does an HTTP request to your Dappnode while itself being behind HTTPS).
11. Make sure the developer address in the form corresponds to the `Dappnode HOPR Repository Owner`, change it if not.
10. In MetaMask, load the `Dappnode HOPR Repository Owner` wallet on Mainnet and click `Connect MetaMask` in the form.
11. Click `Sign release` to sign the release (confirm in MM). New IPFS hash is created with the signed release, changes automatically in the Release hash field in the form.
12. Click `Publish release` to publish the signed release (confirm transaction in MM).
13. On the forked repo https://github.com/hoprnet/DAppNodePackage-Hopr GH page, do a Sync of the `main` branches from the Upstream repo.
