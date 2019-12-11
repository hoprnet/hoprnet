# Introduction

Hopr-ethereum contains the on-chain logic that is used to process payments for [HOPR.network](https://hopr.network) on the Ethereum blockchain.

Table of Contents:

- [Introduction](#introduction)
- [Install](#install)
- [Contracts](#contracts)
  - [HoprChannel](#hoprchannel)
  - [HoprToken](#hoprtoken)

# Install

Requirements:

- [Node.js](https://nodejs.org)
- [Yarn](https://yarnpkg.com)

```bash
# Installs dependencies & compiles the typescript files
yarn # `yarn build` will be executed after install

yarn develop # spawns a development blockchain
yarn migrate
yarn test
```

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
