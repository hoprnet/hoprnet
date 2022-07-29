# Network Registry

To test HOPR protocol and develop dApps on top of HOPR at a resonable scale, nodes are only allowed to join the network (sending messages) if they are registered on a "Network Registry" smart contract.

This restriction on the access guarded by the "Network Registry" is only enabled in the staging or production environment by default. If you are running a cluster of HOPR nodes locally in the hardhat network, the "Network Registry" is not enabled.

There are two ways of registering a node:

- By the node runner itself, providing the node runner is eligible; or
- By the owner of the `HoprNetworkRegistry` smart contract

Relevant smart contracts are listed below, per environment:

| Contract                 | Staging                                                                                                                      | Production                                                                                                                           |
| ------------------------ | ---------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| xHOPR                    | [0xe8aD2ac347dA7549Aaca8F5B1c5Bf979d85bC78F](https://goerli.etherscan.io/token/0xe8aD2ac347dA7549Aaca8F5B1c5Bf979d85bC78F)   | [0xD057604A14982FE8D88c5fC25Aac3267eA142a08](https://blockscout.com/xdai/mainnet/address/0xD057604A14982FE8D88c5fC25Aac3267eA142a08) |
| HOPR Boost               | [0xd7ECa0E90cD85b08875E7d10d4D25B274C6CC549](https://goerli.etherscan.io/token/0xd7eca0e90cd85b08875e7d10d4d25b274c6cc549)   | [0x43d13D7B83607F14335cF2cB75E87dA369D056c7](https://blockscout.com/xdai/mainnet/address/0x43d13D7B83607F14335cF2cB75E87dA369D056c7) |
| HOPR Stake (Season 3)    | [0x0d4Ec37e692BcD36FE7dDcB37a14358d7F44d72C](https://goerli.etherscan.io/address/0x0d4Ec37e692BcD36FE7dDcB37a14358d7F44d72C) | [0xae933331ef0bE122f9499512d3ed4Fa3896DCf20](https://blockscout.com/xdai/mainnet/address/0xae933331ef0bE122f9499512d3ed4Fa3896DCf20) |
| HoprNetworkRegistry      | [0x3e5AA27125C90686444b2d093BFe9b843E82D2F5](https://goerli.etherscan.io/address/0x3e5AA27125C90686444b2d093BFe9b843E82D2F5) |                                                                                                                                      |
| HoprNetworkRegistryProxy | [0x3ee6e1eaE59C44EC30bc5e8FEeE587f95C9F2626](https://goerli.etherscan.io/address/0x3ee6e1eaE59C44EC30bc5e8FEeE587f95C9F2626) |                                                                                                                                      |

## Register a node by the runner

### Eligibility

A node can be registered by its runner if the runner is eligible. There are two ways to become an eligible account:

- A node runner's Ethereum account is staking in the HOPR stake program for a minimum stake of 1000 xHOPR token
- A node runner's Ethereum account is staking a "HOPR Boost NFT" of type `Dev`

#### Stake xHOPR tokens in staging environment

To stake xHOPR tokens, you can interact directly with the staking contract of the environment your HOPR node is running on. For production network, there is even a web application for such a purpose.

For the <mark>staging environment</mark>, please call the following function where the `privatekey` is the private key of the node runner's account. This call can only succeed if the caller (i.e. the `privatekey` or the node runner) has enough xHOPR (on goerli staging environment).

```
make stake-funds environment=master-goerli network=goerli privkey=<private key of “account”>
```

If there's not enough xHOPR token, please use "Dev Bank" account to transfer some to the node runner's account.

#### Stake Dev NFT in staging environment

<mark>When not in production</mark>, CI/CD will mint "Dev" NFTs to `CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[1]` and `CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[3]` on deployment.

There are 10 "Dev" NFTs being minted to the "Dev Bank" account per deployment, where you can transfer some tokens from.

For the <mark>staging environment</mark>, please call the following function where the `privatekey` is the private key of the node runner's account. This call can only succeed if the caller (i.e. the `privatekey` or the node runner) has "Dev" NFT (on goerli staging environment).

```
make stake-devnft environment=master-goerli network=goerli privkey=<private key of “account”>
```

### Register the peer ID

An eligible node runner can call `selfRegister(string hoprPeerId)` method from `HoprNetworkRegistry` smart contract to register its HOPR node. Note that only one node per account is allowed for registration. If a node has been registered by the caller, the caller must deregister the old peerId before registering a new one.

For the <mark>staging environment</mark>, please call the following function where the `privatekey` is the private key of the node runner's account. This call can only succeed if the caller (i.e. the `privatekey` or the node runner) is eligible (having enough stake or a "Dev" NFT).

```
make self-register-node environment=master-goerli network=goerli privkey=<private key of “account”> peer-id <peer id>
```

## Deregister a node

A node runner can call `selfDeregister()` method from `HoprNetworkRegistry` smart contract to de-register an old HOPR node.

For the <mark>staging environment</mark>, please call the following function where the `privatekey` is the private key of the node runner's account.

```
yarn workspace @hoprnet/hopr-ethereum hardhat register:self --network goerli --task remove --privatekey 0x6789...
```

## Register a node by the Network Registry contract owner

### Eligibility

Owner can register any account for any node. The eligibility of an account is not going to be checked unless a `sync` method for that account gets called.

### Register the peer ID

Owner can call `ownerRegister(address[] accounts, string[] hoprPeerIds)` method from `HoprNetworkRegistry` smart contract to register a list of HOPR nodes for a list of accounts respectively. Note that this registration can overwrite existing entries.

For the <mark>staging environment</mark>, please call the following function where the `privatekey` is the private key of the owner.

```
yarn workspace @hoprnet/hopr-ethereum hardhat register --network goerli --task add --nativeAddresses <address1,address2,address3,address4> --peerIds <peerid1,peerid2,peerid3,peerid4>
```

## Deregister a node

Owner can call `ownerDeregister(address[] accounts)` method from `HoprNetworkRegistry` smart contract to de-register for a list of accounts.

For the <mark>staging environment</mark>, please call the following function where the `privatekey` is the private key of the owner.

```
yarn workspace @hoprnet/hopr-ethereum hardhat register --network goerli --task remove --nativeAddresses <address1,address2,address3,address4>
```

## Enable and disable globally

As mentioned in the beginning, by default, Network Registry is enabled for staging envirionment and disabled in the local network.
To toggle the network registry, the following method can be called

```
yarn workspace @hoprnet/hopr-ethereum hardhat register --network goerli --task disable
```

or

```
yarn workspace @hoprnet/hopr-ethereum hardhat register --network goerli --task enable
```

## Internal NR testing - obtain a dev NFT and register your node on NR

1. Create a MetaMask wallet (note as “account”)
2. Send 1 Goerli ETH from “DevBank” to the “account”
3. Request a Dev NFT from "DevBank" with `make request-dev-nft environment=master-goerli network=goerli privkey=<private key of "DevBank"> recipient <"account">`.
4. Stake NFT `make stake-devnft environment=master-goerli network=goerli privkey=<private key of “account”>` It detects if the “account” needs to stake Dev NFT and stake
5. Start your Hopr node and copy node’s “Peer ID”
6. Register "Peer ID" with `make self-register-node environment=master-goerli network=goerli privkey=<private key of “account”> peer-id <peer id>`. If the "account" has registered with a different "Peer ID", a deregister needs to be done with `make self-deregister-node environment=master-goerli network=goerli privkey=<private key of “account”>`
7. Fund the HOPR node as usual

Or

1. Create a MetaMask wallet (note as “account”)
2. Fund 1 Goerli ETH (from “DevBank” or from the faucet) to the “account”
3. Start your local HOPR node
4. Run `HOPRD_API_TOKEN=<hoprd_api_token> ACCOUNT_PRIVKEY=<account_private_key> DEV_BANK_PRIVKEY==<dev_bank_private_key> ./scripts/register-node.sh 0x35A3e15A2E2C297686A4fac5999647312fdDfa3f <hopr_api>`. Provide `<hopr_api>` when it's different from `localhost:3001`
