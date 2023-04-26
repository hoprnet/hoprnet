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

| Field                    | Type      | Description                                       |
| ------------------------ | --------- | ------------------------------------------------- |
| id                       | ID!       | Account address                                   |
| actualLockedTokenAmount  | BigInt!   | actual locked token amount for the account        |
| airdropLockedTokenAmount | BigInt!   | Amount of airdrop locked tokens that the user has |
| lastSyncTimestamp        | BigInt!   | Timestamp of the last syncronization              |
| cumulatedRewards         | BigInt!   | Total amount of rewards the user has earned       |
| claimedRewards           | BigInt!   | Amount of rewards the user has claimed            |
| unclaimedRewards         | BigInt!   | Amount of rewards the user has claimed            |
| boostRate                | BigInt!   | Boost rate for the user                           |
| appliedBoosts            | [Boost!]! | Applied boosts for the user                       |
| ignoredBoosts            | [Boost!]! | Ignored boosts for the user                       |

## Boost

| Field          | Type    | Description                                  |
| -------------- | ------- | -------------------------------------------- |
| id             | ID!     | Account address                              |
| boostTypeIndex | BigInt! | Boost type index for the boost               |
| uri            | String! | Url to the metadata of the NFT               |
| boostNumerator | BigInt! | Boost numerator for the boost                |
| redeemDeadline | BigInt! | Deadline by which the boost must be redeemed |

## Program

| Field                 | Type       | Description                                           |
| --------------------- | ---------- | ----------------------------------------------------- |
| id                    | ID!        | Account address                                       |
| availableReward       | BigInt!    | Available reward for the program                      |  
| totalLocked           | BigInt!    | Total amount locked for the program                   |
| totalAirdrop          | BigInt!    | Total airdrop amount for the program                  |
| totalCumulatedRewards | BigInt!    | Total cumulated rewards for the program               |
| totalClaimedRewards   | BigInt!    | Total claimed rewards for the program                 |
| totalUnclaimedRewards | BigInt!    | Total unclaimed rewards for the program               |
| lastSyncTimestamp     | BigInt!    | Timestamp of the last synchronization for the program |
| blockedTypeIndexes    | [BigInt!]! | List of blocked type indexes for the program          |

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
