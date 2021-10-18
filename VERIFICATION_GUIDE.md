# Introduction

Right now verifying our smart contracts in Blockscout has been quite difficult with Hardhat, mainly due to the fact that multiple licenses are embedded when flattening the smart contract, as well as the usage of `abiencoder v2`. The first issue can be seen [documented here](https://github.com/nomiclabs/hardhat/issues/1050), and the second one has been describe [already here](https://github.com/blockscout/blockscout/issues/3211). The "official" solution by NomicLabs is being tracked [here](https://github.com/nomiclabs/hardhat/issues/1499). For the time being, to solve this, we implemented this [workaround](https://github.com/boringcrypto/dictator-dao/blob/a3de9f606d05852eb5cfa811a3f38870ab22800a/hardhat.config.js).

# Steps to verify smart contract

1. Create flatten version using workaround script

```
yarn workspace @hoprnet/hopr-ethereum run flatten contracts/HoprChannels.sol --output HoprChannels-flatten.sol
```

2. Go to https://blockscout.com/xdai/mainnet/address/$address_to_verify/contract_verifications/new, select `Via flattened source code` and click "Next".

3. Fill the "New Solidity Smart Contract Verification" form with the following information:

```
Contract Name: HoprChannels.sol
Include nightly builds: No
Compiler: v0.8.3+commit.8d00100c (as of time of writing, might change)
EVM Version: default
Optimization: yes
Optimization runs: 200 (as of time of writing, might change)
Enter the Solidity Contract Code: *copy the contents of HoprChannels-flatten.sol*
Try to fetch constructor arguments automatically: yes
```

4. Click "Verify & Publish"
