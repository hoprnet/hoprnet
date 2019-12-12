# Introduction

Hopr-ethereum contains the on-chain logic that is used to process payments for [HOPR.network](https://hopr.network) on the Ethereum blockchain.

Table of Contents:

- [Introduction](#introduction)
- [Requirements](#requirements)
- [Install](#install)
- [Testing](#testing)
- [Coverage](#coverage)
- [Contracts](#contracts)
  - [HoprChannel](#hoprchannel)
  - [HoprToken](#hoprtoken)

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

Everytime `yarn test` is run, it will make sure to always regenerate typescript types, however, while working on the smart contracts you may want to keep types up to date before running a test, you can achieve that by using:

```bash
# 1. Runs linter
# 2. Compiles smart contracts
# 3. Generates smart contracts' typescript types
yarn build:sol
```

> tip: we can use truffle's [debug](https://www.trufflesuite.com/docs/truffle/getting-started/debugging-your-contracts#debugging-your-contracts) feature to seemingly debug our tests, take look at this [example](./test/examples/DebugExample.test.ts)

# Coverage

```bash
# 1. Runs solidity-coverage
# 2. Stores result in `coverage` folder
yarn coverage
```

> tip: see coverage results by launching `./coverage/index.html`

# Contracts

## HoprChannel

## HoprToken

A standard ERC20 token with snapshot functionality.

```
Name: HOPR
Symbol: HOPR
Decimals: 18
Total Supply: 100,000,000
```
