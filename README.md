# Introduction

Hopr-ethereum contains the on-chain logic that is used to process payments for [HOPR.network](https://hopr.network) on the Ethereum blockchain.

Table of Contents:

- [Introduction](#introduction)
- [Requirements](#requirements)
- [Install](#install)
- [Testing](#testing)
- [Coverage](#coverage)
- [Migrating](#migrating)
- [Contracts](#contracts)
  - [HoprChannel](#hoprchannel)
  - [HoprToken](#hoprtoken)
- [Future Improvements](#future-improvements)

# Requirements

- [Node.js](https://nodejs.org)
- [Yarn](https://yarnpkg.com)

# Install

```bash
# 1. Installs dependancies
# 2. Runs linter
# 3. Compiles smart contracts
# 4. Generates smart contracts' typescript types
# 5. Compiles migrations to `.js`
yarn
```

# Testing

```bash
# 1. Runs linter
# 2. Compiles smart contracts
# 3. Generates smart contracts' typescript types
# 4. Compiles migrations to `.js`
# 5. If ganache port is free, launches ganache or use an existing instance
# 6. Runs `truffle test`
yarn test
```

> tip: we can use truffle's [debug](https://www.trufflesuite.com/docs/truffle/getting-started/debugging-your-contracts#debugging-your-contracts) feature to seemingly debug our tests, take look at this [example](./examples/test/DebugExample.test.ts)

Everytime `yarn test` is run, it makes sure to always regenerate typescript types before running the tests.
If you want to generate types only, you can do it like this:

```bash
# 1. Runs linter
# 2. Compiles smart contracts
# 3. Generates smart contracts' typescript types
yarn build:sol
```

# Coverage

```bash
# 1. Runs solidity-coverage
# 2. Stores result in `coverage` folder
yarn coverage
```

> tip: see coverage results by launching `./coverage/index.html`

# Migrating

While migrations are implemented, setups for testnet/mainnet are not yet complete.
You can run migrations by running your own in memory blockchain, for example:

```bash
npx truffle develop
yarn migrate
```

# Contracts

## HoprChannel

## HoprToken

A standard ERC20 token with snapshot functionality.

```
Name: HOPR Token
Symbol: HOPR
Decimals: 18
Total Supply: 100,000,000
```

# Future Improvements

* **ganache-cli-coverage**: eventually we would like to switch to [ganache-core-coverage](https://github.com/OpenZeppelin/ganache-core-coverage) once it matures enough. [#issue](https://forum.openzeppelin.com/t/how-is-solidity-coverage-integrated-into-openzeppelin/1323/3)

* **redundant compiles**: when running `yarn test` or `yarn coverage`, we always make sure to generate the latest typescript types, this requires us to compile the contracts. Internally, both scripts use `truffle test` which recompiles the contracts even though they haven't changed. [#issue](https://github.com/trufflesuite/truffle/issues/469) [#solution](https://github.com/trufflesuite/truffle/issues/2661)

* **fix-typechain**: [typechain](https://github.com/ethereum-ts/TypeChain) does not fully support Truffle v5 yet. [#issue](https://github.com/ethereum-ts/TypeChain/issues/193)

* **fix-truffle-typings**: [truffle-typings](https://github.com/ethereum-ts/truffle-typings) does not fully support Truffle v5 yet. [#issue](https://github.com/ethereum-ts/truffle-typings/pull/13#issuecomment-550325019)