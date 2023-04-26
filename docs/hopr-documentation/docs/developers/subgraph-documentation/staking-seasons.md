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

| Field                    | Type         | Description                                                  |
| ------------------------ | ------------ | ------------------------------------------------------------ |
| id                       | ID!          | Unique identifier for staking particpation                   |
| Account                  | Account!     | User account that participates in staking                    |
| stakingSeason            | StakeSeason! | Staking season during which the user participated in staking |
| actualLockedTokenAmount  | BigInt!      | Actual amount of tokens that the user locked                 |
| virtualLockedTokenAmount | BigInt!      | Amount of virtual locked tokens that the user has            |
| airdropLockedTokenAmount | BigInt!      | Amount of airdrop locked tokens that the user has            |
| lastSyncTimestamp        | BigInt!      | Timestamp of the last syncronization                         |
| cumulatedRewards         | BigInt!      | Total amount of rewards the user has earned                  |
| claimedRewards           | BigInt!      | Amount of rewards the user has claimed                       |
| unclaimedRewards         | BigInt!      | Amount of rewards the user has not yet claimed               |
| boostRate                | BigInt!      | Boost rate for the user                                      |
| appliedBoosts            | [Boost!]!    | Applied boosts for the user                                  |
| ignoredBoosts            | [Boost!]!    | Ignored boosts for the user                                  |

## Boost

| Field          | Type     | Description                                 |
| -------------- | -------- | ------------------------------------------- |
| id             | ID!      | Unique identifier for the boost             |
| Owner          | Account! | User account that owns the boost            |
| boostTypeIndex | BigInt!  | Index of the boost type                     |
| uri            | String!  | Universal resource identifier for the boost |
| boostNumerator | BigInt!  | Numerator of the boost rate                 |
| redeemDeadline | BigInt!  | Deadline for redeeming the boost            |

## StakeSeason

| Field                 | Type                     | Description                                                      |
| --------------------- | ------------------------ | ---------------------------------------------------------------- |
| id                    | ID!                      | Unique identifier for the staking season                         |
| seasonNumber          | BigInt!                  | Season number of the staking season                              |
| availableReward       | BigInt!                  | Availabe reward for the staking season                           |
| totalLocked           | BigInt!                  | Total number of tokens locked during staking season              |
| totalVirtual          | BigInt!                  | Total number of virtual tokens during the staking season         |
| totalAirdrop          | BigInt!                  | Total number of airdrop tokens during the staking season         |
| totalCumulatedRewards | BigInt!                  | Total number of cumulated rewards during the staking season      |
| totalClaimedRewards   | BigInt!                  | Total number of claimed rewards during the staking season        |
| totalUnclaimedRewards | BigInt!                  | Total number of unclaimed rewards during the staking season      |
| lastSyncTimestamp     | BigInt!                  | Timestamp of the last syncronization                             |
| blockedTypeIndexes    | [BigInt!]!               | Number of blocked type indexes                                   |
| stakingParticipation  | [StakingParticipation!]! | The staking participation derived from the field - stakingSeason |

## Account

| Field                | Type                     | Description                                                |
| -------------------- | ------------------------ | ---------------------------------------------------------- |
| id                   | ID!                      | Unique identifier for the account                          |
| stakingParticipation | [StakingParticipation!]! | The staking participation derived from the field - account |
| ownedBoosts          | [Boost!]!                | Boosts owned by the user                                   |

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
