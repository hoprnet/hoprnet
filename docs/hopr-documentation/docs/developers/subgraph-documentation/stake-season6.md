---
id: entities-and-queries
sidebar_position: 4
sidebar_label: Stake Season6
title: Entities & Sample Queries
---

## HOPR Stake Season6 - Entities

- [Account](#account)
- [Boost](#boost)
- [Program](#program)

## Account

| Field                    | Type      | Description                  |
| ------------------------ | --------- | ---------------------------- |
| id                       | ID!       | Account address              |
| actualLockedTokenAmount  | BigInt!   | Actual locked token amount   |
| airdropLockedTokenAmount | BigInt!   | Airdrop locked token amount  |
| lastSyncTimestamp        | BigInt!   | Last sync timestamp          |
| cumulatedRewards         | BigInt!   | Cumulated rewards            |
| claimedRewards           | BigInt!   | Claimed rewards              |
| unclaimedRewards         | BigInt!   | Unclaimed rewards            |
| boostRate                | BigInt!   | Boost rate                   |
| appliedBoosts            | [Boost!]! | Applied Boosts               |
| ignoredBoosts            | [Boost!]! | Ignored boosts               |

## Boost

| Field          | Type    | Description                    |
| -------------- | ------- | ------------------------------ |
| id             | ID!     | Account address                |
| boostTypeIndex | BigInt! | Boost type index               |
| uri            | String! | Url to the metadata of the NFT |
| boostNumerator | BigInt! | Boost numerator                |
| redeemDeadline | BigInt! | Redeem deadline                |

## Program

| Field                 | Type       | Description               |
| --------------------- | ---------- | ------------------------- |
| id                    | ID!        | Account address           |
| availableReward       | BigInt!    | Available reward          |  
| totalLocked           | BigInt!    | Total amount locked       |
| totalAirdrop          | BigInt!    | Total airdrop amount      |
| totalCumulatedRewards | BigInt!    | Total cumulated rewards   |
| totalClaimedRewards   | BigInt!    | Total claimed rewards     |
| totalUnclaimedRewards | BigInt!    | Total unclaimed rewards   |
| lastSyncTimestamp     | BigInt!    | Last sync timestamp       |
| blockedTypeIndexes    | [BigInt!]! | Blocked type index        |

## Sample Queries

Below are some sample queries you can use to gather information from the Stake Season6 subgraph.

You can build your own queries using a [GraphQL Explorer](https://graphiql-online.com/graphiql) and enter your endpoint to limit the data to exactly what you need.

Each entity has a plural version and a singular version. When querying for a single record response (e.g. account), you will need to supply the id for the entity. When querying for a list of responses (e.g. accounts), you may add filters using the 'where' clause.

### Staked balance

Description: This query returns balances for available rewards, total claimed and unclaimed for staked tokens.

```graphql
{
  programs {
    availableReward
    totalClaimedRewards
    totalUnclaimedRewards
  }
}
```

### Account BoostRate

Description: This query, in descending order returns the first 10 accounts with a boost rate greater than 3000.

```graphql
{
  accounts(
    where: {boostRate_gte: "3000"}
    first: 10
    orderBy: id
    orderDirection: desc
  ) {
    id
    claimedRewards
    boostRate
  }
}
```
