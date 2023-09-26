# Dappnode Release process 

This guide aim to explain the process of publishing a new HOPR package in the Dappnode public repository:

The Dappnode package is versioned differently in its own repository: https://github.com/dappnode/DAppNodePackage-Hopr
We maintain a fork of that repository at https://github.com/hoprnet/DAppNodePackage-Hopr
Each Dappnode package version is tied to a certain upstream version in our Monorepo (As of now: 1.0.8 -> 1.93.0)


# Prerequisites

- In order to publish a new version on Dappnode the deployer needs to configure in its own machine the VPN to connect to dappNode
Follow this guide: https://welcome.dappnode.io/vpn and https://docs.dappnode.io/user/product-manual/vpn
- Configure the Metamask account for Dappnode. The credentials are stored in Bitwarden under the name: `Dappnode Repo Owner Wallet`
- In Site setting for site `https://dappnode.github.io/` make sure you allow Insecure content (it usually does an HTTP request to your Dappnode while itself being behind HTTPS).
# Process


The current process is as follows when a new version needs to be published:

1. Clone or pull the HOPR Dappnode repo from https://github.com/hoprnet/DAppNodePackage-Hopr
2. Execute the following commands
````
RELEASE_NUMBER=2.0.0-rc.7

docker_tag=$(gcloud artifacts docker tags list europe-west3-docker.pkg.dev/hoprassociation/docker-images/hoprd --filter=tag:${RELEASE_NUMBER} --format="csv[no-heading](tag,version)" 2&> /dev/null | grep -v "cache" | grep -v "\-pr" | sed 's/,/@/')
cd DAppNodePackage-Hopr
git checkout main
git fetch upstream
git pull
git checkout -b update/${RELEASE_NUMBER}
dappnode_patch_version=$(jq -r '.version' dappnode_package.json | sed 's/.*\.//')
dappnode_minor_version=$(jq -r '.version' dappnode_package.json | sed 's/\.'${dappnode_patch_version}'//')
dappnode_patch_bumped_patch_version=$((dappnode_patch_version + 1))
dappnode_version="${dappnode_minor_version}.${dappnode_patch_version}"
dappnode_bumped_version="${dappnode_minor_version}.${dappnode_patch_bumped_patch_version}"

cat <<< $(jq --arg dappnode_bumped_version ${dappnode_bumped_version} ' . |= .+ { "version": $dappnode_bumped_version}' dappnode_package.json) > dappnode_package.json
cat <<< $(jq --arg dappnode_bumped_version ${dappnode_bumped_version} ' . |= .+ { "version": $dappnode_bumped_version}' dappnode_package.json) > dappnode_package.json
yq -i e ".services.node.image |= \"node.hopr.public.dappnode.eth:${dappnode_bumped_version}\"" docker-compose.yml
git add dappnode_package.json docker-compose.yml
git commit -m "Bumping to version ${RELEASE_NUMBER}"
git push --set-upstream origin update/${RELEASE_NUMBER}
````
3. Create a PR from [here](https://github.com/hoprnet/DAppNodePackage-Hopr) the to the point to https://github.com/dappnode/DAppNodePackage-Hopr
4. Wait until the PR is approved and merged.
5. Open Metamask and switch to the Dappnode account. Check also that is connected to the Ethereum network (Mainnet).
6. Turn on your Dappnode
7. Connect to your Dappnode VPN: `System Preferences` -> `Network` -> `dAppNode Wireguard`
8. Access to the recently published release https://github.com/dappnode/DAppNodePackage-Hopr/releases, and click the link that takes you to the pre-filled release signing form.
9. Set the public ethereum address of the Metamask account into the form field named `Developer address`
10. Click `Connect MetaMask` in the form.
11. Click `Sign release` to sign the release (confirm in MM). New IPFS hash is created with the signed release, changes automatically in the Release hash field in the form.
12. Click `Publish release` to publish the signed release (confirm transaction in MM).
13. On the forked repo https://github.com/hoprnet/DAppNodePackage-Hopr GH page, do a Sync of the `main` branches from the Upstream repo.


