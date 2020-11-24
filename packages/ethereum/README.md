# HOPR net

HOPR is a privacy-preserving messaging protocol that incentivizes users to participate in the network. It provides privacy by relaying messages via several relay nodes to the recipient. Relay nodes are getting paid via payment channels for their services.

## hopr-ethereum

Hopr-ethereum contains the on-chain logic that is used to process payments for [hoprnet.org](https://hoprnet.org) on the Ethereum blockchain.

## Table of Contents

- [HOPR net](#hopr-net)
  - [hopr-ethereum](#hopr-ethereum)
  - [Table of Contents](#table-of-contents)
- [Requirements](#requirements)
- [Install](#install)
- [Build](#build)
- [Testing](#testing)
- [Coverage](#coverage)
- [Migrating](#migrating)
- [Contracts](#contracts)
  - [HoprChannel](#hoprchannel)
  - [HoprToken](#hoprtoken)
  - [Linting](#linting)
- [Future Improvements](#future-improvements)

# Requirements

- [Node.js](https://nodejs.org)
- [Yarn](https://yarnpkg.com)

# Install

```bash
# 1. Installs dependancies
yarn
```

# Build

```bash
# 1. Runs linter
# 2. Compiles smart contracts
# 3. Generates smart contracts' typescript types
# 4. Compiles migrations to `.js`
yarn build
```

# Testing

```bash
# Runs `truffle test`
yarn test
```

> tip: we can use truffle's [debug](https://www.trufflesuite.com/docs/truffle/getting-started/debugging-your-contracts#debugging-your-contracts) feature to seemingly debug our tests, take look at this [example](./examples/test/DebugExample.test.ts)

# Coverage

```bash
# 1. Runs solidity-coverage
# 2. Stores result in `coverage` folder
yarn coverage
```

> tip: see coverage results by launching `./coverage/index.html`

# Migrating

For public network migrations (rinkeby, kovan, [etc](./utils/networks.ts)), you will have to create a [.env](./.env.example) file within the root directory of this project.

```bash
yarn network # starts a locally hosted network
yarn migrate

# deploying smart contract on a public network
yarn migrate --network matic
```

# Contracts

## HoprChannel

## HoprToken

A standard ERC777 token with snapshot functionality.

```
Name: HOPR Token
Symbol: HOPR
Decimals: 18
Total Supply: 100,000,000
```

## Linting

We use solhint's default preset to perform linting onto our smart contracts.

# Future Improvements

- **ganache-cli-coverage**: eventually we would like to switch to [ganache-core-coverage](https://github.com/OpenZeppelin/ganache-core-coverage) once it matures enough. [#issue](https://forum.openzeppelin.com/t/how-is-solidity-coverage-integrated-into-openzeppelin/1323/3)

- **redundant compiles**: when running `yarn test` or `yarn coverage`, we always make sure to generate the latest typescript types, this requires us to compile the contracts. Internally, both scripts use `truffle test` which recompiles the contracts even though they haven't changed. [#issue](https://github.com/trufflesuite/truffle/issues/469) [#solution](https://github.com/trufflesuite/truffle/issues/2661)
