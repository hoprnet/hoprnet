---
id: entities-and-queries
sidebar_position: 5
sidebar_label: Staking Seasons
title: Entities & Sample Queries
---

## Stake All Seasons Entities

- [`StakingParticipation`](#stakingparticipation)
- [`Boost`](#boost)
- [`StakeSeason`](#stakeseason)
- [`Account`](#account)

## StakingParticipation

| Field                    | Type         | Description                                |
| ------------------------ | ------------ | ------------------------------------------ |
| id                       | ID!          | Unique identifier for staking particpation |
| Account                  | Account!     | User account                               |
| stakingSeason            | StakeSeason! | Staking season                             |
| actualLockedTokenAmount  | BigInt!      | Actual amount of tokens locked             |
| virtualLockedTokenAmount | BigInt!      | Amount of virtual locked tokens            |
| airdropLockedTokenAmount | BigInt!      | Amount of airdrop locked tokens            |
| lastSyncTimestamp        | BigInt!      | Timestamp of last sync                     |
| cumulatedRewards         | BigInt!      | Cumulated rewards                          |
| claimedRewards           | BigInt!      | Rewards claimed                            |
| unclaimedRewards         | BigInt!      | Unclaimed rewards                          |
| boostRate                | BigInt!      | Boost rate                                 |
| appliedBoosts            | [Boost!]!    | Applied boosts                             |
| ignoredBoosts            | [Boost!]!    | Ignored boosts                             |

## Boost

| Field          | Type     | Description                     |
| -------------- | -------- | ------------------------------- |
| id             | ID!      | Unique identifier for the boost |
| Owner          | Account! | User account                    |
| boostTypeIndex | BigInt!  | Boost type index                |
| uri            | String!  | Universal resource identifier   |
| boostNumerator | BigInt!  | Boost numerator                 |
| redeemDeadline | BigInt!  | Redeem deadline                 |

## StakeSeason

| Field                 | Type                     | Description                        |
| --------------------- | ------------------------ | ---------------------------------- |
| id                    | ID!                      | Unique identifier for stake season |
| seasonNumber          | BigInt!                  | Stake season number                |
| availableReward       | BigInt!                  | Availabe reward for season         |
| totalLocked           | BigInt!                  | Total tokens locked                |
| totalVirtual          | BigInt!                  | Total virtual tokens               |
| totalAirdrop          | BigInt!                  | Total airdrop tokens               |
| totalCumulatedRewards | BigInt!                  | Total cumulated rewards for season |
| totalClaimedRewards   | BigInt!                  | Total claimed rewards for season   |
| totalUnclaimedRewards | BigInt!                  | Total unclaimed rewards            |
| lastSyncTimestamp     | BigInt!                  | Timestamp of last sync             |
| blockedTypeIndexes    | [BigInt!]!               | Number of blocked type indexes     |
| stakingParticipation  | [StakingParticipation!]! | Derived from field - stakingSeason |

## Account

| Field                | Type                     | Description                   |
| -------------------- | ------------------------ | ----------------------------- |
| id                   | ID!                      | Unique identifier for account |
| stakingParticipation | [StakingParticipation!]! | Derived from field - account  |
| ownedBoosts          | [Boost!]!                | Derived from field - owner    |

## Sample Queries

You can build your own queries using a [GraphQL Explorer](https://graphiql-online.com/graphiql) and enter your endpoint to limit the data to exactly what you need.

Each entity has a plural version and a singular version. When querying for a single record response (e.g. account), you will need to supply the id for the entity. When querying for a list of responses (e.g. accounts), you may add filters using the 'where' clause.

Below are some sample queries you can use to gather information from the HOPR contracts.

### Get Staking and Boost Info

Retrieve information about staking participations and boosts, including their IDs, associated accounts or owners, staking seasons, actual locked token amounts, boost type indexes, and URIs.

```graphql
{
  stakingParticipations(first: 5) {
    id
    account {
      id
    }
    stakingSeason {
      id
    }
    actualLockedTokenAmount
  }
  boosts(first: 5) {
    id
    owner {
      id
    }
    boostTypeIndex
    uri
  }
}
```

### Staking Participants

Where the cumulatedRewards field is greater than 1 ETH (which is represented in wei). It returns the id and cumulatedRewards fields for each staking participation, as well as the associated staking season information, including the id, seasonNumber, and various totals related to the staking season.

```graphql
{
  stakingParticipations(
    first: 10
    where: { cumulatedRewards_gt: 1000000000000000000 }
  ) {
    id
    cumulatedRewards
    stakingSeason {
      id
      seasonNumber
      totalLocked
      totalVirtual
      totalAirdrop
      totalCumulatedRewards
      totalClaimedRewards
      totalUnclaimedRewards
    }
  }
}
```

### Top Stake Season by Total Locked

Retrieves information on the top stake seasons based on the amount of total tokens locked in each season.

```graphql
{
  stakeSeasons(first: 10, orderBy: totalLocked, orderDirection: desc) {
    id
    seasonNumber
    totalLocked
  }
}
```
