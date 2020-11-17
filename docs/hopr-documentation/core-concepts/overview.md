---
description: A general introduction to the HOPR ecosystem.
---

# Overview

HOPR is a two-layer network with communication and incentivization mechanisms.

**HOPR** is a privacy-networking protocol able to communicate and transfer messages securely. This is defined by a **payment layer**, which is a distributed ledger technology \(DLT\) or blockchain infrastructure able to open payment channels on behalf of **HOPR nodes** running the **HOPR network**.

## Ethereum Blockchain

In its first implementation, HOPR relies on the **Ethereum blockchain** as its payment layer. Using **Ethereum smart contracts**, we open **payment channels** on behalf of **HOPR nodes** that forward messages. Message senders attach **HOPR** **tokens** in their messages, which upon successful delivery are paid to the **HOPR nodes** that relayed the message.

To achieve this, a **HOPR node** implements a connector interface that communicates to the Ethereum Blockchain using its popular web library, _web3js_**.** These interfaces allow **HOPR nodes** to monitor, approve, sign and verify when a message is transferred, and thus close a state channel and receive their earned HOPR Tokens. Each **HOPR node** verifies each other, avoiding foul play and rewarding only honest relayers.

![HOPR Protocol Ethereum Blockchain connector architecture](../.gitbook/assets/image%20%2821%29%20%282%29%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29.png)

Although the first instantiation of the **HOPR network** is on the Ethereum blockchain, HOPR is _chain agnostic_**,** which means that **HOPR nodes** can eventually implement different payment channels in different blockchains.

At the time of writing, HOPR is also able to implement a [Polkadot-enabled payment gateway.](https://github.com/hoprnet/hopr-polkadot)
