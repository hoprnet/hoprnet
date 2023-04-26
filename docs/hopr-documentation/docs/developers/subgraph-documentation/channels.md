---
id: entities-and-queries
sidebar_position: 3
sidebar_label: Channels
title: Entities & Sample Queries
---

## HOPR Channels (`monte_rosa` release) - Entities

- [Account](#account)
- [StatusSnapshot](#statussnapshot)
- [Ticket](#ticket)
- [Channel](#channel)
- [NetworkRegistry](#networkregistry)

## Account

| Field             | Type        | Description                                                   |
| ----------------- | ----------- | ------------------------------------------------------------- |
| id                | Bytes!      | Unique identifier for the account                             |
| publicKey         | Bytes       | Public key associated with the account                        |
| multiaddr         | [Bytes!]!   | A list of multi-addresses associated with the account         |
| fromChannels      | [Channel!]! | Channels where the account is the source                      |
| toChannels        | [Channel!]! | Channels where the account is the destination                 |
| fromChannelsCount | BigInt!     | Total number of outgoing channels associated with the account |
| toChannelsCount   | BigInt!     | Total number of incoming channels associated with the account |
| hasAnnounced      | Boolean!    | Indicates whether the account has set a multi-address for others to use when establishing channels with it |
| balance           | BigDecimal! | Sum of the channel balances where the account is the source   |
| isActive          | Boolean!    | Indicates whether the account has at least one open channel   |
| openChannelsCount | BigInt!     | Number of active channels associated with the account         |

## StatusSnapshot

| Field     | Type           | Description                                              |
| --------- | -------------- | -------------------------------------------------------- |
| id        | String!        | Tx hash - tx index                                       |
| channel   | Channel!       | Channel at snapshot                                      |
| status    | ChannelStatus! | Status of the channel at the time the snapshot was taken |
| timeStamp | BigInt!        | Timestamp of the snapshot                                |

## Ticket

| Field              | Type        | Description                                       |
| ------------------ | ----------- | ------------------------------------------------- |
| id                 | String!     | Channel epoch - ticket epoch - ticket index       |
| channel            | Channel!    | Channel in which the ticket was opened            |
| nextCommitment     | Bytes!      | Next commitment that will be made in the channel  |
| ticketEpoch        | BigInt!     | Epoch in which the ticket was created             |
| ticketIndex        | BigInt!     | Index of the ticket within the epoch              |
| proofOfRelaySecret | Bytes!      | Proof of the relay secret                         |
| amount             | BigDecimal! | Amount of funds that were committed to the ticket |
| winProb            | BigInt!     | Win probability for ticket                        |
| signature          | Bytes!      | Signature that was used to sign the ticket        |

## Channel

| Field               | Type               | Description                                                     |
| ------------------- | ------------------ | --------------------------------------------------------------- |
| id                  | Bytes!             | Tx hash - tx index                                              |
| source              | Account!           | Account that opened the channel                                 |
| destination         | Account!           | Account that the channel is connected to                        |
| balance             | BigDecimal!        | Current balance of the channel                                  |
| commitment          | Bytes!             | Current commitment for the channel                              |
| channelEpoch        | BigInt!            | Epoch in which the channel was created                          |
| ticketEpoch         | BigInt!            | Epoch in which the tickets for this channel can be created      |
| ticketIndex         | BigInt!            | Index of the next ticket to be created for the channel          |
| status              | ChannelStatus      | Current status of the channel, such as open, closed, or settled |
| commitmentHistory   | [Bytes!]!          | History of commitments made for the channel                     |
| statusHistory       | [StatusSnapshot!]! | History of status changes for the channel                       |
| lastOpenedAt        | BigInt!            | History of status changes for the channel                       |
| lastClosedAt        | BigInt!            | Timestamp when the channel was last closed                      |
| tickets             | [Ticket!]!         | Tickets associated with this channel                            |
| redeemedTicketCount | BigInt!            | Number of tickets that have been redeemed for this channel      |

## NetworkRegistry

| Field           | Type      | Description                                             |
| --------------- | --------- | ------------------------------------------------------- |
| id              | Bytes!    | Account that registered nodes                           |
| registeredPeers | String!   | List of HOPR nodes (or peers) registered by the account |
| eligibility     | Boolean!  | Indicates if the account is eligible                    |

## Sample Queries

Below are some sample queries you can use to gather information from the HOPR Channel subgraph.

You can build your own queries using a [GraphQL Explorer](https://graphiql-online.com/graphiql) and enter your endpoint to limit the data to exactly what you need.

Each entity has a plural version and a singular version. When querying for a single record response (e.g. account), you will need to supply the id for the entity. When querying for a list of responses (e.g. accounts), you may add filters using the 'where' clause.

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
