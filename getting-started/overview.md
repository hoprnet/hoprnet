---
description: A general introduction to the HOPR ecosystem.
---

# Overview

HOPR is a two-layer network with communication and incentivization mechanisms.

**HOPR** is a privacy-networking protocol able to communicate and transfer messages securely. This is defined by a **payment layer**, which is a distributed ledger technology \(DLT\) or blockchain infrastructure able to open payment channels on behalf of **HOPR Nodes** running the **HOPR Network**.

### Ethereum Blockchain

In its first implementation, HOPR relies on the **Ethereum Blockchain** as its payment layer. Using **Ethereum Smart Contracts**, we open **payment channels** on behalf of **HOPR Nodes** that forward messages. Message senders attach **HOPR** **Tokens** in their messages, which upon successful delivery are paid to the **HOPR Nodes** that relayed the message.

To achieve this, a **HOPR Node** implements a connector interface that communicates to the Ethereum Blockchain using its popular web library, _web3js_**.** These interfaces allow **HOPR Nodes** to monitor, approve, sign and verify when a message is transferred, and thus close a state channel and receive their earned HOPR Tokens. Each **HOPR Node** verifies each other, avoiding foul play and rewarding only honest relayers.

![HOPR Protocol Ethereum Blockchain connector architecture](../.gitbook/assets/image%20%2821%29.png)

Although the first instantiation of the **HOPR Network** is on the Ethereum Blockchain, HOPR is _chain agnostic_**,** which means that **HOPR Nodes** can eventually implement different payment channels in different blockchains. 

At the time of writing, HOPR is also able to implement a [Polkadot-enabled payment gateway.](https://github.com/hoprnet/hopr-polkadot)

