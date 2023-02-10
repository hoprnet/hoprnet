---
id: entities-and-queries
sidebar_position: 3
sidebar_label: Token (Gnosis)
title: Entities & Sample Queries
---

## HOPR Token (Gnosis) - Entities

- [Account](#account)

## Account

| Field         | Type     | Description               |
| -------------- | --------| ------------------------- |
| id             | Bytes!  | Account address           |
| xHoprBalance   | BigInt! | # uint256                 |
| wxHoprBalance  | BigInt! | # uint256                 |
| totalBalance   | BigInt! | Total hopr token balance  |
| blockTimestamp | BigInt! | Block timestamp           |
| blockNumber    | BigInt! | Block number              |

## Sample Queries

Below are some sample queries you can use to gather information from the HOPR Token (Gnosis) subgraph.

You can build your own queries using a [GraphQL Explorer](https://graphiql-online.com/graphiql) and enter your endpoint to limit the data to exactly what you need.

Each entity has a plural version and singular version. When querying for a single record response (eg account) you will need to supply the id for the entity. When querying for list of responses (eg accounts) you may add filters using the 'where' clause.

### Token Balance

Description: This query returns accounts that have both xhopr and wxhopr tokens and their corresponding amounts.

```graphql
{
  accounts(where: {wxHoprBalance_gte: "1", xHoprBalance_gte: "1"}) {
    id
    wxHoprBalance
    xHoprBalance
    totalBalance
  }
}
```

### XHOPR Token

Description: This query returns the first 10 accounts that have XHOPR tokens after a certain block number.

```graphql
{
  accounts(
    where: {xHoprBalance_gte: "1", blockNumber_gte: "24743804"}
    first: 10
    orderBy: id
    orderDirection: asc
  ) {
    id
    xHoprBalance
    totalBalance
    blockNumber
  }
}
```
