---
id: smart-contract
title: Smart Contract Overview
---

**HoprChannels** is the main smart contract of HOPR protocol. It manages the payment channel between HOPR nodes and the announcement of public nodes.

### Lifecycle of payment channel

A payment channel runs throug multiple states during its lifecycle, as shown in the figure below.

1. Initially, each payment channel is **Closed**.
2. Node can lock some assets to payment channels, which leads to the status of **Wait for commitement**.
3. When the destination channel sets a commitment, the payment channel is considered as **Open**. If the counterparty has already set commitment, then funding immediately changes the state to **Open**.
4. When closing a payment channel, the payment channel first goes into **Pending to close**. Channels need to wait for a timeout for nodes to redeem tickets and thus retrieve the assets. In rare cases it can happen that the counterparty does not submit an on-chain commit after a node intends to open a payment channel. Under this circumstance, the channel can be turned immediately into **Pending to close**.
5. Once the timeout is done, any of the involving nodes can finalize the closure and turn the payment channel state into **Closed**.

![Payment channel states and possible state transitions.](/img/developer/channel_lifecycle.png)

### Struct

#### Channel

```solidity
struct Channel {
    uint256 balance;
    bytes32 commitment;
    uint256 ticketEpoch;
    uint256 ticketIndex;
    ChannelStatus status;
    uint256 channelEpoch;
    uint32 closureTime;
}
```

#### Parameters:

| Name           | Type                | Description                                                                           |
| -------------- | ------------------- | ------------------------------------------------------------------------------------- |
| `balance`      | uint256             | Stake of HOPR token in the channel.                                                   |
| `commitment`   | bytes32             | Commitment of the channel, set by the destination node.                               |
| `ticketEpoch`  | uint256             | Ticket epoch of the channel.                                                          |
| `ticketIndex`  | uint256             | Ticket index of the channel.                                                          |
| `status`       | enum(ChannelStatus) | The Proof-of-Relay secret.                                                            |
| `channelEpoch` | uint256             | Current channel epoch.                                                                |
| `closureTime`  | uint32              | Block timestamp when the channel can be closed. Note that it overloads at year >2105. |

### Events

#### ChannelUpdated

```solidity
event ChannelUpdated(
    address indexed source,
    address indexed destination,
    Channel newState
);
```

Emitted on every channel state change.

#### Parameters:

| Name          | Type           | Indexed | Description                                                    |
| ------------- | -------------- | ------- | -------------------------------------------------------------- |
| `source`      | address        | true    | Ethereum address of the source node of a payment channel.      |
| `destination` | address        | true    | Ethereum address of the destination node of a payment channel. |
| `newState`    | tuple(Channel) | false   | Latest state of the payment channel.                           |

#### Announcement

```solidity
event Announcement(
    address indexed account,
    bytes publicKey,
    bytes multiaddr
);
```

Emitted once an account announces.

#### Parameters:

| Name        | Type    | Indexed | Description                               |
| ----------- | ------- | ------- | ----------------------------------------- |
| `account`   | address | true    | Ethereum address of the public HOPR node. |
| `publicKey` | bytes   | false   | Public key of the announced HOPR node.    |
| `multiaddr` | bytes   | false   | Multiaddress of the announced HOPR node.  |

#### ChannelFunded

```solidity
event ChannelFunded(
    address indexed funder,
    address indexed source,
    address indexed destination,
    uint256 amount
);
```

Emitted once a channel if funded.

#### Parameters:

| Name          | Type    | Indexed | Description                                           |
| ------------- | ------- | ------- | ----------------------------------------------------- |
| `funder`      | address | true    | Address of the account that funds the channel.        |
| `source`      | address | true    | Address of the source node of a payment channel.      |
| `destination` | address | true    | Address of the destination node of a payment channel. |
| `amount`      | uint256 | false   | Amount of HOPR being staked into the channel          |

#### ChannelOpened

```solidity
event ChannelOpened(
    address indexed source,
    address indexed destination
);
```

Emitted once a channel is opened.

#### Parameters:

| Name          | Type    | Indexed | Description                                           |
| ------------- | ------- | ------- | ----------------------------------------------------- |
| `source`      | address | true    | Address of the source node of a payment channel.      |
| `destination` | address | true    | Address of the destination node of a payment channel. |

#### ChannelBumped

```solidity
event ChannelBumped(
    address indexed source,
    address indexed destination,
    bytes32 newCommitment,
    uint256 ticketEpoch,
    uint256 channelBalance
);
```

Emitted once bumpChannel is called and the commitment is changed.

#### Parameters:

| Name             | Type    | Indexed | Description                                           |
| ---------------- | ------- | ------- | ----------------------------------------------------- |
| `source`         | address | true    | Address of the source node of a payment channel.      |
| `destination`    | address | true    | Address of the destination node of a payment channel. |
| `newCommitment`  | bytes32 | false   | New channel commitment                                |
| `ticketEpoch`    | uint256 | false   | Current ticket epoch of the channel.                  |
| `channelBalance` | uint256 | false   | Amount of HOPR being staked into the channel          |

#### ChannelClosureInitiated

```solidity
event ChannelClosureInitiated(
    address indexed source,
    address indexed destination,
    uint32 closureInitiationTime
);
```

Emitted once a channel closure is initialized.

#### Parameters:

| Name                    | Type    | Indexed | Description                                                  |
| ----------------------- | ------- | ------- | ------------------------------------------------------------ |
| `source`                | address | true    | Address of the source node of a payment channel.             |
| `destination`           | address | true    | Address of the destination node of a payment channel.        |
| `closureInitiationTime` | uint32  | false   | Block timestamp at which the channel closure is initialized. |

#### ChannelClosureInitiated

```solidity
event ChannelClosureFinalized(
    address indexed source,
    address indexed destination,
    uint32 closureFinalizationTime,
    uint256 channelBalance
);
```

Emitted once a channel closure is finalized.

#### Parameters:

| Name                      | Type    | Indexed | Description                                                |
| ------------------------- | ------- | ------- | ---------------------------------------------------------- |
| `source`                  | address | true    | Address of the source node of a payment channel.           |
| `destination`             | address | true    | Address of the destination node of a payment channel.      |
| `closureFinalizationTime` | uint32  | false   | Block timestamp at which the channel closure is finalized. |
| `channelBalance`          | uint256 | false   | Current stake in the channel.                              |

#### TicketRedeemed

```solidity
event TicketRedeemed(
    address indexed source,
    address indexed destination,
    bytes32 nextCommitment,
    uint256 ticketEpoch,
    uint256 ticketIndex,
    bytes32 proofOfRelaySecret,
    uint256 amount,
    uint256 winProb,
    bytes signature
);
```

Emitted once a ticket is redeemed.

#### Parameters:

| Name                 | Type    | Indexed | Description                                                                |
| -------------------- | ------- | ------- | -------------------------------------------------------------------------- |
| `source`             | address | true    | Address of the source node of a payment channel.                           |
| `destination`        | address | true    | Address of the destination node of a payment channel.                      |
| `nextCommitment`     | bytes32 | false   | Commitment that hashes to the redeemers previous commitment.               |
| `ticketEpoch`        | uint256 | false   | Current ticket epoch of the channel.                                       |
| `ticketIndex`        | uint256 | false   | Current ticket index of the channel.                                       |
| `proofOfRelaySecret` | bytes32 | false   | The Proof-of-Relay secret.                                                 |
| `amount`             | uint256 | false   | Amount of HOPR token embedded in the ticket.                               |
| `winProb`            | uint256 | false   | The probability of which the ticket wins. This value is set by the sender. |
| `signature`          | bytes   | false   | Signature associated with the ticket, which is signed by the source node.  |
