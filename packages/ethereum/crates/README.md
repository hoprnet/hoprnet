# HOPR Ethereum Package

Draft readme, for rust migration

## Installation

1. `rustup`
2. `foundryup`
3. `brew install lcov` (to install lcov for viewing coverage report)

## Contracts

```
cd contracts
```

### Test

```
make sc-test
```

### Run Coverage

```
make sc-coverage
```

### Deployment and verfy deployed contracts

#### Local

```
anvil
make anvil-deploy-erc1820
FOUNDRY_PROFILE=development ENVIRONMENT_NAME=localhost forge script --broadcast script/DeployAll.s.sol:DeployAllContractsScript
```

#### Staging

Apply for api keys for goerli etherscan and enter in `.env` file

```
source .env
```

```
// This verifies contract on sourcify
FOUNDRY_PROFILE=staging ENVIRONMENT_NAME=debug-goerli forge script --broadcast --verify --verifier sourcify script/DeployAll.s.sol:DeployAllContractsScript

// This deploys contract to goerli testnet and verifies contracts on etherscan
FOUNDRY_PROFILE=staging ENVIRONMENT_NAME=debug-goerli forge script --broadcast --verify --verifier etherscan --chain 5 script/DeployAll.s.sol:DeployAllContractsScript
```

#### Production

```
FOUNDRY_PROFILE=staging ENVIRONMENT_NAME=debug-goerli forge script --broadcast --verify --verifier sourcify script/DeployAll.s.sol:DeployAllContractsScript
```

### Note

1. Three solc versions are needed

- 0.4: Permittable token, implementation of xHOPR. The permittable token implementation is extracted from the deployed xHOPR token. The only alternative done on the contract is to keep `pragma solidity` with the least version
- 0.6: Deployed Hoprtoken
- 0.8: More recent contracts

2. Dependencies are vendored directly into the repo. Some of them are locked to a specific version

```
forge install foundry-rs/forge-std \
openzeppelin-contracts=OpenZeppelin/openzeppelin-contracts@v4.4.2 \
--no-git --no-commit
```

3. If `forge coverage` is not found in as a command, or error in using `writeJson`, update `foundryup` to the [latest nightly release](https://github.com/foundry-rs/foundry/releases) may solve the issue.
   E.g.

```
foundryup --version nightly-64cbdd183e0aae99eb1be507196b6b5d640b3801
```

4. `forge coverage` may run into `Error: Function has no kind` when compiler has multiple versions. Opened an issue https://github.com/foundry-rs/foundry/issues/3519

<!-- 5. To move faster on the migration of toolchain and to avoid constantly running into foundry's error during the compilation `Error: Discovered incompatible solidity versions in following`, the compiler version of the following contracts have be moved to `pragma solidity >=0.6.0 <0.9.0;`
- src/HoprToken.sol (^0.6.0)
- src/ERC777/ERC777Snapshot.sol (^0.6.0)
- src/openzeppelin-contracts/ERC777.sol (>=0.6.0 <0.8.0)
- lib/openzeppelin-contracts-v3-0-1/contracts/access/AccessControl.sol (^0.6.0)
- lib/openzeppelin-contracts-v3-0-1/contracts/GSN/Context.sol (^0.6.0)
- lib/openzeppelin-contracts-v3-0-1/contracts/introspection/IERC1820Registry.sol (^0.6.0)
- lib/openzeppelin-contracts-v3-0-1/contracts/math/SafeMath.sol (^0.6.0)
- lib/openzeppelin-contracts-v3-0-1/contracts/token/ERC20/IERC20.sol (^0.6.0)
- lib/openzeppelin-contracts-v3-0-1/contracts/token/ERC777/IERC777.sol (^0.6.0)
- lib/openzeppelin-contracts-v3-0-1/contracts/token/ERC777/IERC777Recipient.sol (^0.6.0)
- lib/openzeppelin-contracts-v3-0-1/contracts/token/ERC777/IERC777Sender.sol (^0.6.0)
- lib/openzeppelin-contracts-v3-0-1/contracts/utils/EnumerableSet.sol (^0.6.0)
- lib/openzeppelin-contracts-v3-0-1/contracts/utils/Address.sol (^0.6.2) -->

6. Remove "PermittableToken.sol" from source code as it prevents coverage engine from working. Possibly because its required compiler version is 0.4.x This contract is only used when testing "HoprWrapper" contract. TODO: use a different approach to test "HoprWrapper"
7. Moved `src/mock` to `test/mock` folder, and adapt the relative path used in "HoprWhitehat.sol"
8. To move faster on the rest of toolchain upgrade, only tests for "HoprToken" contract is fully migrated. Tests for "HoprChannels" is halfway through. TODO: complete tests for the following contracts:

```
|____stake
| |____HoprStake.t.sol
| |____HoprStake2.t.sol
| |____HoprStakeSeason3.t.sol
| |____HoprStakeSeason4.t.sol
| |____HoprStakeSeason5.t.sol
| |____HoprStakeBase.t.sol
| |____HoprBoost.t.sol
| |____HoprWhitehat.t.sol
|____proxy
| |____HoprStakingProxyForNetworkRegistry.t.sol
| |____HoprDummyProxyForNetworkRegistry.t.sol
|____ERC777
| |____ERC777Snapshot.t.sol
|____HoprChannels.t.sol (the other half)
|____HoprDistributor.t.sol
|____HoprForwarder.t.sol
|____HoprWrapper.t.sol
|____HoprNetworkRegistry.t.sol
```

5. Temporarily skipped deployment scripts for

- HoprDistributor
- HoprWrapper

6. <del>writeJson is next inline https://github.com/foundry-rs/foundry/pull/3595, to save deployed addressed used in function `writeEnvironment()` in `contracts/script/utils/EnvironmentConfig.s.sol`</del> As `writeJson` got introduced in the foundry nightly release but its smart contract hasn't been introduced in `forge-std`. The current walk-around is to manually add `serialize*` functions [mentioned in the PR](https://github.com/foundry-rs/foundry/pull/3595) into the `Vm.sol` contract.
   However, to fully unleash the power of `writeJson`, especially for nested arrays, compiler version needs to be bumped to `>=0.8.0`. Therefore, a few contracts bumped to `pragma solidity >=0.6.0 <0.9.0;`, such as

- src/HoprToken.sol (^0.6.0)
- src/HoprDistributor.sol (^0.6.0)
- src/HoprWrapper.sol (^0.6.0)
- src/ERC777/ERC777Snapshot.sol (^0.6.0)
- src/openzeppelin-contracts/ERC777.sol (>=0.6.0 <0.8.0)
- lib/openzeppelin-contracts-v3-0-1/contracts/access/AccessControl.sol (^0.6.0)
- lib/openzeppelin-contracts-v3-0-1/contracts/GSN/Context.sol (^0.6.0)
- lib/openzeppelin-contracts-v3-0-1/contracts/introspection/IERC1820Registry.sol (^0.6.0)
- lib/openzeppelin-contracts-v3-0-1/contracts/math/SafeMath.sol (^0.6.0)
- lib/openzeppelin-contracts-v3-0-1/contracts/token/ERC20/IERC20.sol (^0.6.0)
- lib/openzeppelin-contracts-v3-0-1/contracts/token/ERC777/IERC777.sol (^0.6.0)
- lib/openzeppelin-contracts-v3-0-1/contracts/token/ERC777/IERC777Recipient.sol (^0.6.0)
- lib/openzeppelin-contracts-v3-0-1/contracts/token/ERC777/IERC777Sender.sol (^0.6.0)
- lib/openzeppelin-contracts-v3-0-1/contracts/utils/EnumerableSet.sol (^0.6.0)
- lib/openzeppelin-contracts-v3-0-1/contracts/utils/Address.sol (^0.6.2)
  Subsequently, library `openzeppelin-contracts-v3-0-1` is also removed from the project

7. Deployment dependencies graph is like the following:

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

Note that deployment for `HoprDistributor` and `HoprWrapper` are skipped; ERC1820Registry is not deployed in production envirionment.

8. The temporary `contract=address.json` has the following differences compared with `protocol-config.json`

- It does not need "networks" attribute
- In "environment" attribute:
  `- "network_id": "goerli", - "version_range": "*", - "channel_contract_deploy_block": 0, + "stake_season": 5,`

9. Contract verification:

- Production (Gnosis Chain): Sourcify or Gnosisscan
- Staging (Goerli): Etherscan. However it's been reported (in foundry support TG) that goerli etherscan verification doesn't work since roughly a week ago.
