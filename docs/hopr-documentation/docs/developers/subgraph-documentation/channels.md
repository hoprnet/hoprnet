---
title: Entities & Sample Queries
sidebar_position: 3
id: entities-and-queries
sidebar_label: Channels
---

## HOPR Channels (`monte_rosa` release) - Entities

- [Account](#account)
- [StatusSnapshot](#statussnapshot)
- [Ticket](#ticket)
- [Channel](#channel)
- [NetworkRegistry](#networkregistry)

## Account

| Field             | Type        | Description                                                 |
| ----------------- | ----------- | ----------------------------------------------------------- |
| id                | Bytes!      | Account's address                                           |
| publicKey         | Bytes       | Account's public key                                        |
| multiaddr         | [Bytes!]!   | Multi address                                               |
| fromChannels      | [Channel!]! | Channels where the account is the source                    |
| toChannels        | [Channel!]! | Channels where the account is the destination               |
| fromChannelsCount | BigInt!     | Number of outgoing channels                                 |
| toChannelsCount   | BigInt!     | Number of incoming channels                                 |
| hasAnnounced      | Boolean!    | has the Account set a multiaddr?                            |
| balance           | BigDecimal! | Sum of the channel balances where the account is the source |
| isActive          | Boolean!    | Has at least 1 open channel                                 |
| openChannelsCount | BigInt!     | Number of active channels                                   |

## StatusSnapshot

| Field     | Type           | Description                |
| --------- | -------------- | -------------------------- |
| id        | String!        | Tx hash - tx index         |
| channel   | Channel!       | Channel at snapshot        |
| status    | ChannelStatus! | Status of the channel      |
| timeStamp | BigInt!        | Timestamp of the snapshot  |

## Ticket

| Field              | Type        | Description                                  |
| ------------------ | ----------- | -------------------------------------------- |
| id                 | String!     | Channel epoch - ticket epoch - ticket index  |
| channel            | Channel!    | Channel opened                               |
| nextCommitment     | Bytes!      | Next commitment                              |
| ticketEpoch        | BigInt!     | Ticket epoch                                 |
| ticketIndex        | BigInt!     | Ticket index                                 |
| proofOfRelaySecret | Bytes!      | Proof of relay secret                        |
| amount             | BigDecimal! | Ticket amount                                |
| winProb            | BigInt!     | Win probability for ticket                   |
| signature          | Bytes!      | Ticket signature                             |

## Channel

| Field               | Type               | Description                            |
| ------------------- | ------------------ | -------------------------------------- |
| id                  | Bytes!             | Tx hash - tx index                     |
| source              | Account!           | Source account                         |
| destination         | Account!           | Destinantion account                   |
| balance             | BigDecimal!        | Channel balance                        |
| commitment          | Bytes!             | Commitment                             |
| channelEpoch        | BigInt!            | Channel epoch                          |
| ticketEpoch         | BigInt!            | Ticket epoch                           |
| ticketIndex         | BigInt!            | Ticket index                           |
| status              | ChannelStatus      | Status of the channel                  |
| commitmentHistory   | [Bytes!]!          | Commitment history                     |
| statusHistory       | [StatusSnapshot!]! | Status history                         |
| lastOpenedAt        | BigInt!            | Timestamp when it was opened last time |
| lastClosedAt        | BigInt!            | Timestamp when it was closed last time |
| tickets             | [Ticket!]!         | Total tickets received in channel      |
| redeemedTicketCount | BigInt!            | Number of redeemed tickes              |

## NetworkRegistry

| Field           | Type      | Description                                             |
| --------------- | --------- | ------------------------------------------------------- |
| id              | Bytes!    | Account that registered nodes                           |
| registeredPeers | String!   | List of HOPR nodes (or peers) registered by the account |
| eligibility     | Boolean!  | If the account is eligible                              |

## Sample Queries

Below are some sample queries you can use to gather information from the HOPR Channel subgraph.

You can build your own queries using a [GraphQL Explorer](https://graphiql-online.com/graphiql) and enter your endpoint to limit the data to exactly what you need.

### Channels

Description: This query returns the channel status and epoch where redeemed ticket count is greater than 1.

```graphql
{
  channels(where: {redeemedTicketCount_gte: "1"}) {
    id
    channelEpoch
    redeemedTicketCount
    status
  }
}
```

### Snapshot

Description: This query returns the snapshot of channels that have a 'closed' status.

```graphql
{
  statusSnapshots(where: {status: CLOSED}) {
    id
    status
    timestamp
    channel {
      balance
      tickets {
        amount
      }
    }
  }
}
```
