# HOPR Ethereum Package

Draft readme, for rust migration.
Appending node-staking management notes to the end.

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

When developing staking contract, make sure to test against the forked gnosis chain, e.g.

```
forge test --fork-url "https://provider-proxy.hoprnet.workers.dev/xdai_mainnet" --match-path test/stake/HoprStakeSeason6.t.sol
```

### Run Coverage

```
make sc-coverage
```

### Deployment and verfy deployed contracts

#### Local

```
# run anvil as a daemon.
anvil & make anvil-deploy-all
```

```
# The anvil daemon can be killed with
lsof -i :8545 -s TCP:LISTEN -t | xargs -I {} -n 1 kill {}
```

#### Staging

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
   --verify --verifier etherscan --verifier-url "https://api.gnosisscan.io/api" \
   --delay 30 --chain 100 --etherscan-api-key "${ETHERSCAN_API_KEY}" \
   --use <specify_if_other_than_that_in_config> \
   script/DeployAll.s.sol:DeployAllContractsScript
```

#### Production

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

#### Deploy new staking season contract

1. Create a new season contract from `HoprStakeBase.sol` and update the parameters (start, end timestamp) accordingly
2. Change the `"stake_season":` to the number of the new season, for `rotsee`, `debug-staging` and the running production environment.
3. Deploy for each network with
   - `debug-staging`: `FOUNDRY_PROFILE=staging NETWORK=debug-staging forge script --broadcast --verify --verifier etherscan --verifier-url "https://api.gnosisscan.io/api" --chain 100 script/DeployAll.s.sol:DeployAllContractsScript`
   - `rotsee`: FOUNDRY_PROFILE=staging NETWORK=rotsee forge script --broadcast --verify --verifier etherscan --verifier-url "https://api.gnosisscan.io/api" --chain 100 script/DeployAll.s.sol:DeployAllContractsScript`
   - `monte_rosa`: _Temporarily update the `token_contract_address` to wxHOPR. Then run_ `FOUNDRY_PROFILE=production NETWORK=monte_rosa forge script --broadcast --verify --verifier etherscan --verifier-url "https://api.gnosisscan.io/api" --chain 100 script/DeployAll.s.sol:DeployAllContractsScript`
4. Switch back `token_contract_address` for `monte_rosa`
5. Commit contract changes and make a PR
6. Transfer contract ownership to COMM multisig with `cast send <new_stake_season_contract> "transferOwnership(address)" 0xD9a00176Cf49dFB9cA3Ef61805a2850F45Cb1D05 --rpc-url https://provider-proxy.hoprnet.workers.dev/xdai_mainnet --private-key $PRIVATE_KEY`
7. Run `scripts/update-protocol-config.sh -e master`, `scripts/update-protocol-config.sh -e debug-staging` and `scripts/update-protocol-config.sh -e monte_rosa`
8. Create a branch (e.g. `feature/staking-s7-contract-update`) from `master` which contains changes in `contracts-addresses.json` and `protocol-config.json` and make a PR
9. Extend "hopr-stake-all-season" subgraph with the production stake season contract and deploy it
10. Create an issue in `hoprnet/hopr-devrel` repo with actions to check:
    > Only after Sx starts (time)
    >
    > - [ ] Merge Update staking Sx contract addresses <contract address update PR>, so that procotol-config.json is updated with latest addresses.
    > - [ ] Important staking accounts (e.g. DevBank, CI, Operator) unstake from S6 and stake their NR NFTs to Sx
    > - [ ] Owner of `NetworkRegistryProxy` update the stake contract (`updateStakeContract()`) with the new season contract address

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
7. Moved `src/mock` to `test/mock` folder, and adapt the relative path used in "HoprWhitehat.sol". Remove `ERC20Mock.sol`, `ERC721Mock.sol`, `ERC777SenderRecipientMock.sol` contracts
8. To move faster on the rest of toolchain upgrade, only tests for "HoprToken" contract is fully migrated. Tests for "HoprChannels" is halfway through. TODO: complete tests for the following contracts:

```
|____stake
| |____HoprStake.t.sol // <- skip this as this contract is archived
| |____HoprStake2.t.sol // <- skip this as this contract is archived
| |____HoprStakeSeason3.t.sol // <- skip this as this contract is archived
| |____HoprStakeSeason4.t.sol // <- skip this as this contract is archived
| |____HoprStakeSeason5.t.sol // <- skip this as this contract is archived
| |____HoprWhitehat.t.sol // <- skip this as this contract is archived
|____HoprForwarder.t.sol // <- skip this as this contract is deprecated. Multisig can register implementation to receive ERC777 tokens
```

Notes on Test cases:

- `testFail_ExceedMaxMint` in `packages/ethereum/contracts/test/HoprDistributor.t.sol` should have been `testRevert_ExceedMaxMint`. However, foundry has trouble catching uint128 overflow.
- After the update of Permittable token, it's possible to wrap tokens with "transfer" (Regarding `test_CanAlsoWrapWithTransfer` case in `HoprWrapper.t.sol`)

5. Temporarily skipped deployment scripts for

- HoprDistributor
- HoprWrapper

6. <del>writeJson is next inline https://github.com/foundry-rs/foundry/pull/3595, to save deployed addressed used in function `writeNetwork()` in `contracts/script/utils/NetworkConfig.s.sol`</del> As `writeJson` got introduced in the foundry nightly release but its smart contract hasn't been introduced in `forge-std`. The current walk-around is to manually add `serialize*` functions [mentioned in the PR](https://github.com/foundry-rs/foundry/pull/3595) into the `Vm.sol` contract.
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

8. The temporary `contract-address.json` has the following differences compared with `protocol-config.json`

- It does not need "chains" attribute
- In "network" attribute:
  ```
  - "chain": "goerli",
  - "version_range": "*",
  - "channel_contract_deploy_block": 0,
  + "stake_season": 5,
  + "indexer_start_block_number": <the minimum block number of transactions that deploys HoprChannels and HoprNetworkRegistry>,
  ```

9. Contract verification:

- Production (Gnosis Chain): Sourcify or Gnosisscan
- Staging (Goerli): Etherscan. However it's been reported (in foundry support TG) that goerli etherscan verification doesn't work since roughly a week ago.

10. Script migration:

- `hardhat accounts` turns into `make get-account-balances network=<name of the network, e.g. "monte_rosa"> environment-type=<type of environment, from development, staging, to production> account=<address to check>`

11. `ETHERSCAN_API_KEY` contains the value of "API key for Gnosisscan", as our production and staging environment is on Gnosis chain. The reason why it remains "ETHERSCAN" instead of "GNOSISSCAN" is that foundry reads `ETHERSCAN_API_KEY` as an environment vairable for both `forge verify-contract` and `forge script`, which can not be configured in the foundry.toml file

## Node Management Smart Contracts

The latest node-staking design uses Safe as a center-piece.
Leveraging its Account-Abstraction design, HOPR node runners can secure node operation with an m-of-n Smart Account.

### Developer Notes:

1. Imported libraries:

Dependencies are vendored directly into the repo. :

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

2. `SafeSuiteSetupScript` deploys basic Safe suites. We deploy all the contracts with `main-suite` tag, in a deterministic way

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

3. deployment starts with a singleton contract. Singleton's deployment details are saved under https://github.com/safe-global/safe-singleton-factory/tree/main/artifacts
   Specifically, for "anvil-deploy-safe-singleton" target, it follows instruction from [safe-global/safe-singleton-factory/artifacts/31337/deployment.json](https://github.com/safe-global/safe-singleton-factory/blob/6700a7c90ececc8cb9e1a4d97fd70fea1ee4670d/artifacts/31337/deployment.json)

4. when running `make run-anvil`, it also deploys the SafeSingleton which is used as a deployer factory in deterministic deployment.
