---
title: Introduction
sidebar_position: 1
id: subgraph-intro
sidebar_label: Subgraph Introduction
---

## HOPR Subgraph Introduction

The following pages contain everything you need to know about the HOPR suite of subgraphs.

HOPR has multiple GraphQL API Endpoints hosted by [The Graph](https://thegraph.com/docs/about/introduction#what-the-graph-is) called a subgraph for indexing and organizing data from the HOPR smart contracts.

These subgraphs can be used to query on-chain HOPR data. The subgraphs data are serviced by a decentralized group of server operators called [Indexers](https://thegraph.com/docs/en/network/indexing/).

The HOPR subgraphs work by listening for events emitted by one or more data sources (Smart Contracts) on the various chains. They handle the indexing and caching of data which can later be queried using the GraphQL API Endpoint, providing an excellent developer experience.

## Get Started

Learn more about how subgraphs work by checking out [The Graph's official documentation](https://thegraph.com/docs/en/). If you are unfamiliar with GraphQL, we recommend taking a quick look through their documentation first [here](https://graphql.org/learn/)

## Current Subgraph locations

## Mainnet

| Subgraph       | Deployment                                                                            |
| :------------- | :------------------------------------------------------------------------------------ |
| HOPR Token     | `https://thegraph.com/explorer/subgraphs/5GJcMEW1uKvE9CRddN6yg8qbPnsGuB5dg4wA1UTEQv5W`|

## Gnosis

| Subgraph                             | Deployment                                                                             |
| :----------------------------------- | :--------------------------------------------------------------------------------------|
| HOPR Token                           | `https://thegraph.com/explorer/subgraphs/njToE7kpetd3P9sJdYQPSq6yQjBs7w9DahQpBj6WAoD`  |
| HOPR Channels (`monte_rosa` release) | `https://thegraph.com/explorer/subgraphs/Feg6Jero3aQzesVYuqk253NNLyNAZZppbDPKFYEGJ1Hj` |
| Stake Season6                        | `https://thegraph.com/explorer/subgraphs/C7cu6NvvMxgjdaK9pKSGezZ3EgRCmM4tWrmTtGUDG15t` |

## GraphQL Endpoint Links

- HOPR Token (Mainnet): `https://gateway.thegraph.com/api/[api-key]/subgraphs/id/5GJcMEW1uKvE9CRddN6yg8qbPnsGuB5dg4wA1UTEQv5W`
- HOPR Token (Gnosis): `https://gateway.thegraph.com/api/[api-key]/subgraphs/id/njToE7kpetd3P9sJdYQPSq6yQjBs7w9DahQpBj6WAoD`
- HOPR Channels: `https://gateway.thegraph.com/api/[api-key]/subgraphs/id/Feg6Jero3aQzesVYuqk253NNLyNAZZppbDPKFYEGJ1Hj`
- Stake Season6: `https://gateway.thegraph.com/api/[api-key]/subgraphs/id/C7cu6NvvMxgjdaK9pKSGezZ3EgRCmM4tWrmTtGUDG15t`

## Helpful Resources

- [Video Tutorial on creating an API Key](https://www.youtube.com/watch?v=UrfIpm-Vlgs)
- [Managing your API Key & setting your indexer preferences](https://thegraph.com/docs/en/studio/managing-api-keys/)
- [Querying from an application](https://thegraph.com/docs/en/developer/querying-from-your-app/)
- [How to use the explorer and playground to query on-chain data](https://medium.com/@chidubem_/how-to-query-on-chain-data-with-the-graph-f8507488215)
- [Explorer Page](https://thegraph.com/explorer/subgraph?id=FDrqtqbp8LhG1hSnwtWB2hE6C97FWA54irrozjb2TtMH&view=Overview)
- [Deploy your own Subgraph](https://thegraph.com/docs/en/developing/creating-a-subgraph/)
