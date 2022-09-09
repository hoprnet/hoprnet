# Network Registry

To test HOPR protocol and develop dApps on top of HOPR at a resonable scale, nodes are only allowed to join the network (sending messages) if they are registered on a "Network Registry" smart contract.

This restriction on the access guarded by the "Network Registry" is only enabled in the staging or production environment by default. If you are running a cluster of HOPR nodes locally in the hardhat network, the "Network Registry" is not enabled.

There are two ways of registering a node:

- By the node runner itself, providing the node runner is eligible; or
- By the owner of the `HoprNetworkRegistry` smart contract

Relevant smart contracts are listed below, per environment **(to be updated)**:

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

- A node runner's Ethereum account is staking in the HOPR stake program for a minimum stake of 1000 xHOPR token and one of the NFTs from the "allowed list"
- A node runner's Ethereum account is staking a "HOPR Boost NFT" of type `Network_registry`

#### Stake xHOPR tokens in staging environment

To stake xHOPR tokens, you can interact directly with the staking contract of the environment your HOPR node is running on. For production network, there is even a web application for such a purpose.

For the <mark>staging environment</mark>, please call the following function where the `PRIVATE_KEY` is the private key of the node runner's account. This call can only succeed if the caller (i.e. the `PRIVATE_KEY` or the node runner) has enough xHOPR (on goerli staging environment).

```
PRIVATE_KEY=<private key of "account"> make stake-funds environment=master-goerli network=goerli
```

If there's not enough xHOPR token, please use "Dev Bank" account to transfer some to the node runner's account.

#### Stake Network_registry NFT in staging environment

<mark>When not in production</mark>, CI/CD will mint "Network_registry" NFTs to `CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[1]` and `CLUSTER_NETWORK_REGISTERY_LINKED_ADDRESSES[3]` on deployment.

There are 6 "Network_registry" NFTs (3 "developer" rank and 3 of "community" rank) being minted to the "Dev Bank" account per deployment, where you can transfer some tokens from.

For the <mark>staging environment</mark>, please call the following function where the `PRIVATE_KEY` is the private key of the node runner's account. This call can only succeed if the caller (i.e. the `PRIVATE_KEY` or the node runner) has "Network_registry" NFT (on goerli staging environment).

```
PRIVATE_KEY<private key of "account"> make stake-nrnft environment=master-goerli network=goerli nftrank=<rank of "Network_registry" nft>
```

### Register the peer ID

An eligible node runner can call `selfRegister(string[] hoprPeerIds)` method from `HoprNetworkRegistry` smart contract to register one or multiple HOPR node(s). The number of nodes one account is allowed to register is subject to the `rank` of the "Network Registry" NFT the account has staked.

For the <mark>staging environment</mark>, please call the following function where the `PRIVATE_KEY` is the private key of the node runner's account. This call can only succeed if the caller (i.e. the `PRIVATE_KEY` of the node runner) is eligible (having enough stake or a "Network_registry" NFT).

```
PRIVATE_KEY=<private key of “account”> make self-register-node environment=master-goerli network=goerli peer-ids=<peerId1,peerId2,peerId3>
```

## Deregister a node

A node runner can call `selfDeregister(string[] hoprPeerIds)` method from `HoprNetworkRegistry` smart contract to remove previously registered HOPR nodes.

For the <mark>staging environment</mark>, please call the following function where the `PRIVATE_KEY` is the private key of the node runner's account.

```
PRIVATE_KEY=<private key of “account”> make self-deregister-node environment=master-goerli network=goerli peer-ids=<peerId1,peerId2,peerId3>
```

## Register a node by the Network Registry contract owner

### Eligibility

Owner can register any account for any node. The eligibility of an account is not going to be checked unless a `sync` method for that account gets called.

### Register the peer ID

Owner can call `ownerRegister(address[] accounts, string[] hoprPeerIds)` method from `HoprNetworkRegistry` smart contract to register a list of HOPR nodes for a list of accounts respectively. Note that this registration can overwrite existing entries.

```
make register-nodes environment=master-goerli network=goerli --native-addresses=<address1,address2,address3,address4> --peer-ids=<peerid1,peerid2,peerid3,peerid4>
make force-eligibility-update environment=master-goerli network=goerli --native-addresses=<address1,address2,address3,address4> --eligibility=<true,false,true,true, etc>
```

## Deregister a node

Owner can call `ownerDeregister(string[] hoprPeerIds)` method from `HoprNetworkRegistry` smart contract to remove a list of nodes.

```
make deregister-nodes environment=master-goerli network=goerli --peer_ids=<peerId1,peerId2,peerId3,peerId4>
```

## Enable and disable globally

As mentioned in the beginning, by default, Network Registry is enabled for staging envirionment and disabled in the local network.
To toggle the network registry, the following method can be called

```
make disable-network-registry environment=master-goerli network=goerli --peer_ids=<peerId1,peerId2,peerId3,peerId4>
```

or

```
make enable-network-registry environment=master-goerli network=goerli --peer_ids=<peerId1,peerId2,peerId3,peerId4>
```

## Internal NR testing

### Staging

To register an eligible node in the NR, there are two options:

- obtain a "Network_registry" NFT and register your node on NR
- stake tokens and register your node on NR

The procedure for both options are very similar, which only some differences in the last step.

#### Procedure

1. Create a MetaMask wallet (note as “account”)
2. Fund 1 Goerli ETH (from “DevBank” or from the faucet) to the “account”
3. Start your local HOPR node
4. Save private keys (`ACCOUNT_PRIVKEY` and `DEV_BANK_PRIVKEY`) into `.env` file

```
export ACCOUNT_PRIVKEY=<account_private_key>
export DEV_BANK_PRIVKEY=<dev_bank_private_key>
```

and

```
source .env
```

5. Run either command. In both cases, provide `<hoprd_endpoint>` when it's different from `localhost:3001`

- Option 1: obtain a "Network_registry" NFT (with nftrank of "developer" or "community") and register your node on NR

  ```
  make register-node-with-nft endpoint=<hoprd_endpoint> nftrank=<"Network_registry" NFT Rank> account=<staking_account> environment=master-goerli network=goerli
  ```

- Option 2: stake tokens and register your node on NR

  ```
  make register-node-with-stake endpoint=<hoprd_endpoint> account=<staking_account> environment=master-goerli network=goerli
  ```

### [DEPRECATED] Production (with dummy proxy)

Before "Network_registry" NFT gets minted in production environment, "Dummy proxy" is used to faciliate the process.
Deployer wallet in the CI/CD registers node and its peerId when calling `make register-nodes` (followed by more flags and arguments). Developers must follow these steps to register their node in the registry:

1. Create a MetaMask wallet (note as “account”)
2. Start your local HOPR node
3. Save private keys (`CI_DEPLOYER_PRIVKEY`) into `.env` file

```
export CI_DEPLOYER_PRIVKEY=<CI deployer account private key>
```

and

```
source .env
```

4. Run command

```
make register-node-when-dummy-proxy endpoint=<hoprd_endpoint> account=<staking_account> environment=monte_rosa network=xdai
```

e.g.

```
make register-node-when-dummy-proxy endpoint="localhost:3001" api_token="^MYtoken4testing^" account=0x35A3e15A2E2C297686A4fac5999647312fdDfa3f environment=monte_rosa network=xdai
```

### Production

For nodes running in the upcoming "monte rosa" environment, only wallets with one "Network_registry" HoprBoost NFT (be `developer` or `community` rank) staked in the staking program are eligible to spin up HOPR nodes.

""
To register one (`community` rank) or many (`developer` rank) eligible node in the NR, please follow:

#### Procedure

1. Create a MetaMask wallet (note as “account”)
2. Fund “account” with some xDAI (e.g 0.1 xDAI)
3. Start your local HOPR node
4. Save private keys (`ACCOUNT_PRIVKEY` and `DEV_BANK_PRIVKEY`) into `.env` file

   ```
   export ACCOUNT_PRIVKEY=<account_private_key>
   ```

   and

   ```
   source .env
   ```

5. Request "Network_registry" NFT. Either by requesting from TECH (or COM) team, or by obtaining directly from "Dev Bank". For the latter, add the following line into `.env` file

   ```
   export DEV_BANK_PRIVKEY=<dev_bank_private_key>
   ```

   and

   ```
   source .env
   ```

   run

   ```
   make register-node-with-nft endpoint=<hoprd_endpoint> nftrank=<"Network_registry" NFT Rank ("developer" or "community")> account=<staking_account> environment=<release_name> network=xdai
   ```

   Note: please provide `endpoint=<hoprd_endpoint>` when the node exposes port different from `localhost:3001`.
