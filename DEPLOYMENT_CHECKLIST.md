# Deployment checklist

## Per $release

The following are a series of manual tasks that are needed to be executed for the launch of a release. Ideally, we automate these entirely and delete this document in the future, but in the meantime, we'll have this document to keep track of these.

- [ ] Deploy a set of `$release` CT nodes for our testnet using our **`cover-traffic` deployment script**.

- [ ] Deploy a set of `$release` cloud nodes for our testnet to support DEADR and be used as relayers (to be removed after https://github.com/hoprnet/hoprnet/issues/2537 is fixed).

- [ ] Deploy a set of `$release` cloud nodes for our testnet with a full topology connected to feed event data for our leaderboard using our **`topology` deployment script**.

- [ ] Tag a distribution manually $release on npm and on Docker Hub.

- [ ] Create a DNS alias for each node (cloud, cover-traffic, topology), to be accessed via our `hoprnet.link` domain (e.g. ct-1-$release.hoprnet.link)

- [ ] Verify the $release smart contract in the explorer platform.

## Per $chain

- [ ] Deploy HOPR token on $chain and mint 130M HOPR tokens for our Development Address `0x2402da10A6172ED018AEEa22CA60EDe1F766655C`.

- [ ] Transfer 1M HOPR token and 1 native $chain to our funding wallet in our CI/CD.

- [ ] Transfer 1M HOPR token and 20 native $chain to our leaderboard wallet in network.hoprnet.org.

# Scripts

## `cover-traffic` deployment script

```
CT_PRIV_KEY=14e6...a6a5 \
HOPRD_INFURA_KEY=51d4...caf6 \
HOPRD_PROVIDER=https://provider-proxy.hoprnet.workers.dev/matic_rio \
./scripts/setup-ct-gcloud-cluster.sh cover-traffic-node-01
```

## `topology` deployment script

```
HOPRD_PERFORM_CLEANUP=false \
FUNDING_PRIV_KEY=0xa77a...21b8 \
HOPRD_INFURA_KEY=51d4...caf6 \
HOPRD_PROVIDER=https://polygon.infura.io/v3/51d4...caf6 \
HOPRD_TOKEN_CONTRACT="0x6F80d1a3AB9006548c2fBb180879b87364D63Bf7" \
HOPRD_SHOW_PRESTART_INFO=true \
./scripts/setup-gcloud-cluster.sh matic-testnet-01 gcr.io/hoprassociation/hoprd:latest `pwd`/scripts/topologies/full_interconnected_cluster.sh
```
