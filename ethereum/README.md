# HOPR Ethereum Package

This directory contains the Ethereum smart contracts and their bindings for the [HOPR protocol](https://github.com/hoprnet/hoprnet).
These contracts power HOPR's privacy-preserving incentive framework.

Main contracts are

```bash
├── Channels.sol                 # Uni-directional payment channel contract
├── Crypto.sol                   # Cryptographic utility functions and primitives
├── MultiSig.sol                 # Multisig modifiers to enforce Safe-based node operations
├── Announcements.sol            # Node announcement mechanism (independent of staking)
├── Ledger.sol                   # Snapshot-based index for HOPR Channels
├── NetworkRegistry.sol          # Network access gate (deprecated and slated for removal)
├── TicketPriceOracle.sol        # Oracle to update HOPR ticket price across the network
├── proxy                        # Adapters between the NetworkRegistry and staking modules
│   ├── DummyProxyForNetworkRegistry.sol
│   ├── SafeProxyForNetworkRegistry.sol
│   └── StakingProxyForNetworkRegistry.sol
├── interfaces                   # Solidity interfaces for contract interoperability
│   ├── IAvatar.sol
│   ├── INetworkRegistryRequirement.sol
│   ├── INodeManagementModule.sol
│   └── INodeSafeRegistry.sol
├── node-stake                  # Node staking system built on Safe's Account-Abstraction design
│   ├── NodeSafeRegistry.sol    # Registry mapping nodes to their Safe wallets
│   ├── NodeStakeFactory.sol    # Factory contract to deploy and initialize node Safes
│   └── permissioned-module     # Modules for Safe-based node management
│       ├── CapabilityPermissions.sol    # Defines capability-based permission rules
│       ├── NodeManagementModule.sol     # Main module to manage nodes via a Safe
│       └── SimplifiedModule.sol         # Lightweight version of the Node Management Module
├── utils                                # Shared utility libraries
│   ├── EnumerableStringSet.sol       # Enumerable set for strings
│   ├── EnumerableTargetSet.sol       # Enumerable set for targets (addresses with metadata)
│   └── TargetUtils.sol               # Helper functions for managing targets
└── static                            # Legacy contracts (archived and not actively maintained)
    ├── EnumerableStringSet.sol
    ├── ERC777
    │   └── ERC777Snapshot.sol
    ├── HoprDistributor.sol           # Contract for token distribution
    ├── HoprForwarder.sol             # Minimal forwarder for meta-transactions
    ├── HoprToken.sol                 # ERC20 token implementation for HOPR
    ├── HoprWrapper.sol               # Legacy wrapper contract
    ├── HoprWrapperProxy.sol          # Proxy for interacting with HoprWrapper
    ├── openzeppelin-contracts
    │   ├── ERC777.sol
    │   └── README.md
    └── stake                         # Legacy staking contracts by season
        ├── HoprBoost.sol
        ├── HoprStake.sol
        ├── HoprStake2.sol
        ├── HoprStakeBase.sol
        ├── HoprStakeSeason3.sol
        ├── HoprStakeSeason4.sol
        ├── HoprStakeSeason5.sol
        ├── HoprStakeSeason6.sol
        ├── HoprStakeSeason7.sol
        ├── HoprWhitehat.sol
        └── IHoprBoost.sol
```

## Installation

Please use the [Nix environment](../README.md#develop) or install the following packages

1. `rustup`
2. `foundryup`
3. `brew install lcov` (to install lcov for viewing coverage report)

If not using Nix, make sure to create a `foundry.toml` file based on `foundry.in.toml` and populate the solc version under "[profile.default]"

Create a file for environment variables:

```
cp ./contracts/.env.example ./contracts/.env
```

and populate the necessary variables.

## Test

### Unit tests

```
cd contracts && make sc-test
```

### Coverage

```
cd contracts && make sc-coverage
```

## Deployment and source-code verification

### Local

```
# run anvil as a daemon.
anvil & make anvil-deploy-all
```

```
# The anvil daemon can be killed with
lsof -i :8545 -s TCP:LISTEN -t | xargs -I {} -n 1 kill {}
```

### Staging

Staging environment is on the same chain as in production - Gnosis chain

```
source .env
```

```
// This verifies contract on sourcify
FOUNDRY_PROFILE=staging NETWORK=debug-staging forge script --broadcast --verify --verifier sourcify script/DeployAll.s.sol:DeployAllContractsScript

// This verifies contract on blockscout
FOUNDRY_PROFILE=staging NETWORK=debug-staging forge script --broadcast \
   --verify --verifier blockscout --verifier-url "https://gnosis.blockscout.com/api?" \
   --etherscan-api-key "${BLOCKSCOUT_API_KEY}" \
   --chain 100 --use 0.8.19 \
   script/DeployAll.s.sol:DeployAllContractsScript

// This deploys contract to staging environment and verifies contracts on Gnosisscan
FOUNDRY_PROFILE=staging NETWORK=debug-staging forge script --broadcast \
   --verify --verifier etherscan --verifier-url "https://api.etherscan.io/v2/api?chainid=100" \
   --delay 30 --chain 100 --etherscan-api-key "${ETHERSCAN_API_KEY}" \
   --use <specify_if_other_than_that_in_config> \
   script/DeployAll.s.sol:DeployAllContractsScript
```

### Production

use either of the command below

```
FOUNDRY_PROFILE=staging NETWORK=debug-staging forge script --broadcast --verify --verifier sourcify script/DeployAll.s.sol:DeployAllContractsScript
```

If contracts are not properly verified on explorers, please try with the manual verification. E.g.

```
# Verify HoprStakingProxyForNetworkRegistry contract on goerli
export ETHERSCAN_API_KEY=<gnosisscan_api_key>
forge verify-contract --verifier etherscan --verifier-url "https://api.gnosisscan.io/api" --chain gnosis --constructor-args $(cast abi-encode "constructor(address,address,uint256)" 0xA02Af160a280957A8881879Ee9239A614Ab47F0D 0x4fF4e61052a4DFb1bE72866aB711AE08DD861976 1000000000000000000000) 0xcA9B1bC189F977B2A9217598D0300d956b6a719f src/proxy/HoprStakingProxyForNetworkRegistry.sol:HoprStakingProxyForNetworkRegistry
```

To check the verification result, use

```
forge verify-check --chain-id <number> <GUID>
```

## Deployment

### Deploy legacy contracts

The following diagram illustrates the deployment order and dependencies among contracts in the `static/` folder.
Contracts at the bottom depend on those above them:

```
              +-----------------+
              | ERC1820Registry |
              +--------^--------+
                       |
                       |
                       |
                +------+------+       +------------+        +-----------+
                |  HoprToken  |       | xHoprToken |        | HoprBoost |
                +^---^--^---^-+       +-^----^-----+        +-----^-----+
                 |   |  |   |           |    |                    |
                 |   |  |   |           |    |                    |
                 |   |  |   |           |    |                    |
                 |   |  |   |           |    |                    |
+----------------++  |  | +-+-----------+-+  |                    |
| HoprDistributor |  |  | |  HoprWrapper  |  |                    |
+-----------------+  |  | +---------------+  |                    |
                     |  |                    |                    |
                     |  |                    |                    |
                     |  +-----------------+  |   +----------------+
                     |                    |  |   |
                     |                 +--+--+---+-+
                     |                 | HoprStake |
                     |                 +-----^-----+
                     |                       |
                     |                       |
                     |                       |
             +-------+------+    +-----------+----------+
             | HoprChannels |    | NetworkRegistryProxy |
             +--------------+    +-----------^----------+
                                             |
                                             |
                                             |
                                  +----------+----------+
                                  | HoprNetworkRegistry |
                                  +---------------------+
```

Note that in local deployment, the deployment for `HoprDistributor` and `HoprWrapper` are skipped.

### Deployed contracts

The `./contracts/contracts-address.json` file defines contract deployments across multiple networks, including local, staging, and production environments.
For each network, it specifies deployed contract addresses, the environment type, and the starting block number for indexers.
It serves as a centralized reference for frontend, indexer, and integration scripts to locate on-chain components.

This file is automatically populated with the latest deployment addresses.

## Note

1. Compared to the versions used in actual deployments (see list below), some contracts in this repository have updated solc versions. This is due to the limitation of Foundry's lack of multi-version Solidity compiler support.

- Solc v0.4: Used for the PermittableToken, which is the base implementation of the deployed xHOPR token. The source code has been extracted from the deployed contract, with the only modification being the loosening of the pragma solidity directive to support compilation.
- Solc v0.6: Used for the deployed HoprToken.
- Solc v0.8: Used for all newer contracts.

2. Most of the libraries are locked to specific commit or version

- Audited Safe contracts at commit [eb93dbb0f62e2dc1b308ac4c110038062df0a8c9](https://github.com/safe-global/safe-contracts/blob/main/docs/audit_1_4_0.md)
- Audited Zodiac Modifier Roles v1 contracts at commit [454be9d3c26f90221ca717518df002d1eca1845f](https://github.com/gnosis/zodiac-modifier-roles-v1/tree/main) After importing the contracts, adjust the pragma for two contracts; and manually imported their imports from Gnosis Safe, e.g. Enum.sol
- Audited Zodiac Base contract at commit [8a77e7b224af8004bd9f2ff4e2919642e93ffd85](https://github.com/gnosis/zodiac/tree/8a77e7b224af8004bd9f2ff4e2919642e93ffd85)

```
forge install safe-global/safe-contracts@eb93dbb0f62e2dc1b308ac4c110038062df0a8c9 \
   gnosis/zodiac-modifier-roles-v1@454be9d3c26f90221ca717518df002d1eca1845f \
   gnosis/zodiac@8a77e7b224af8004bd9f2ff4e2919642e93ffd85 \
   OpenZeppelin/openzeppelin-contracts-upgradeable \
   --no-git --no-commit
```

3. `SafeSuiteSetupScript` deploys basic Safe suites. We deploy all the contracts with `main-suite` tag, in a deterministic way

|                              | l2  | l2-suite | main-suite | accessors | factory | handlers | libraries | singleton |
| ---------------------------- | --- | -------- | ---------- | --------- | ------- | -------- | --------- | --------- |
| SimulateTxAccessor           |     | x        | x          | x         |         |          |           |           |
| SafeProxyFactory             |     | x        | x          |           | x       |          |           |           |
| TokenCallbackHandler         |     | x        | x          |           |         | x        |           |           |
| CompatibilityFallbackHandler |     | x        | x          |           |         | x        |           |           |
| CreateCall                   |     | x        | x          |           |         |          | x         |           |
| MultiSend                    |     | x        | x          |           |         |          | x         |           |
| MultiSendCallOnly            |     | x        | x          |           |         |          | x         |           |
| SignMessageLib               |     | x        | x          |           |         |          | x         |           |
| SafeL2                       | x   | x        |            |           |         |          |           |           |
| Safe                         |     |          | x          |           |         |          |           | x         |

4. Deployment starts with a singleton contract. Singleton's deployment details are saved under https://github.com/safe-global/safe-singleton-factory/tree/main/artifacts
   Specifically, for "anvil-deploy-safe-singleton" target, it follows instruction from [safe-global/safe-singleton-factory/artifacts/31337/deployment.json](https://github.com/safe-global/safe-singleton-factory/blob/6700a7c90ececc8cb9e1a4d97fd70fea1ee4670d/artifacts/31337/deployment.json)

   When running `make run-anvil`, it also deploys the SafeSingleton which is used as a deployer factory in deterministic deployment.

4. In the "Dufour" network, node-staking safes use the implementation of `Safe.sol` v1.3 and node-staking modules use an undeclared version of the `NodeManagementModule.sol`.
As the module proxies were created with a minimal proxy, where the implementation address was supplied at the deployment, and due to the fact that the module contract does not allow delegatecalls, it is not possible to migrate existing NodeManagementModules to a different implementation.
The desired workflow should be that the owner of a module (i.e. the Safe contract to which the module is attached) MAY call a `migrate` function at its own will to change the implementation contract address to a different one.
As a result, the `NodeSafeMigration` contract is created as a supporting contract to faciliate process of:
   - creating a new NodeManagementModule proxy instance that uses `NodeManagementModule.sol` v2.0.0
   - initiate the basic targets on the module instance (e.g. for Channels, Token, Announcement, Send). Channels, Tokens, and Announcement contracts should be already deployed and supplied in the `NodeSafeMigration` contract.
   - include nodes into the new module.
   - set the owner back to the creator (caller) address.
Optionally, the `NodeSafeMigration` contract also contains a function to migrate Safe implementation to a different one.
Node runners should ensure that all the tickets are redeemed and all the channels are closed before the migration. Then turn off the node.
Execute the migration process using multicall:
   - migrate module
   - optionally remove the previous module from the Safe
   - optionally upgrade the Safe implementation


## Change log
### HoprNodeManagementModule

1. `isHoprNodeManagementModule` variable is removed.
2. Added `VERSION` ("2.0.0") for the implementation of node management module. This function is also included in the interface.

### HoprChannels

1. Include the indexed `channelId` topic in the `ChannelOpened` event.