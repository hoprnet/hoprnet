# Overview

The HOPR ecosystem is a two-platform network with dynamic components powering its communication and incentivization mechanisms.

In one side, we have the **HOPR Core,** the privacy-networking module able to communicate and transfer messages securely. In the other side, we have a **Payment Gateway**, which is a Distributed Ledger Technology \(DLT\) or Blockchain infrastructure able to open payment channels on behalf of nodes running in the HOPR Network.

In its first implementation, HOPR relies on the **Ethereum Blockchain** as its first payment gateway using **Ethereum Smart Contracts.** Using Ethereum Smart Contracts**,** we can open **State Channels** on behalf of the relayers while forwarding messages. Senders of the messages then attach **\$HOPR** tokens in their messages, which upon successful delivery, are deducted and paid to the relayers involved.

To implement this process, a HOPR node implements a **Connector Interface** that communicates to the Ethereum network using its popular web library, **Web3.js.** These interfaces allow HOPR nodes to monitor, approve, sign and verify when a message is transfered, and thus close a State Channel and get their \$HOPR earned. Each node verify each other, avoiding foul play and rewarding only **Honest Relayers**.

![](.gitbook/assets/paper.bloc.8-2.png)

Although the first interaction of the HOPR network is on the Ethereum network, HOPR is by design **Chain Agnostic,** which means that HOPR nodes can eventually implement different payment channels in different Blockchains. At the time of writing, HOPR is also able to implement a [Polkadot-enabled payment gateway.](https://github.com/hoprnet/hopr-polkadot)

# Contents

# Table of contents

- [HOPR](README.md)

## Getting started

- [First Time Users: Start Here](getting-started/hopr-chat.md)

## HOPR Chat Tutorial

- [First Time Setup](hopr-chat-tutorial/quickstart.md)
- [Previous Users](hopr-chat-tutorial/getting-started.md)
- [Funding Your Node](hopr-chat-tutorial/funding-your-node.md)
- [Exploring The Network](hopr-chat-tutorial/exploring-the-network.md)
- [Messaging RandoBot](hopr-chat-tutorial/randobot.md)
- [Registering with CoverBot](hopr-chat-tutorial/coverbot.md)
- [Earning and Redeeming Tickets](hopr-chat-tutorial/redeeming-tickets.md)
- [Changing Your Routing Settings](hopr-chat-tutorial/changing-your-routing-settings.md)
- [Sending a Multi-Hop Message Using a Payment Channel](hopr-chat-tutorial/opening-and-closing-payment-channels.md)
- [Withdrawing Funds](hopr-chat-tutorial/withdrawing-funds.md)
- [Advanced Setup](hopr-chat-tutorial/setup.md)
- [Troubleshooting](hopr-chat-tutorial/troubleshooting.md)

## HOPR AVADO Node Tutorial

- [Setting Up Your AVADO Node](hopr-avado-node-tutorial/setting-up-your-avado-node.md)
- [Funding Your AVADO Node](hopr-avado-node-tutorial/funding-your-avado-node.md)
- [Exploring The Network](hopr-avado-node-tutorial/finding-your-address.md)
- [Messaging Randobot](hopr-avado-node-tutorial/talking-with-randobot.md)
- [Registering With CoverBot](hopr-avado-node-tutorial/registering-with-coverbot.md)
- [Earning and Redeeming Tickets](hopr-avado-node-tutorial/redeeming-tickets.md)
- [Changing Your Routing Settings](hopr-avado-node-tutorial/changing-your-routing-settings.md)
- [Sending A Multi-hop Message Using A Payment Channel](hopr-avado-node-tutorial/sending-a-multi-hop-message.md)
- [Withdrawing Funds](hopr-avado-node-tutorial/withdrawing-funds.md)

## Core Concepts

- [Overview](core-concepts/overview.md)
- [Protocol, Network and Token](core-concepts/protocol-network-token.md)
- [HOPR Chat](core-concepts/hopr-chat/README.md)
  - [Troubleshooting](core-concepts/hopr-chat/troubleshooting.md)
- [Proof of Relay](core-concepts/proof-of-relay/README.md)
  - [Routing Settings](core-concepts/proof-of-relay/routing-settings.md)
- [Bootstrap Nodes](core-concepts/bootstrap-nodes.md)
- [Cover Traffic](core-concepts/cover-traffic.md)
- [Tokens](core-concepts/tokens/README.md)
  - [Native Tokens](core-concepts/tokens/native-tokens.md)
  - [Testnet HOPR Tokens](core-concepts/tokens/hopr-tokens.md)
- [Payment Channels](core-concepts/payment-channels.md)
- [Tickets](core-concepts/tickets.md)

## Resources

- [Glossary](resources/glossary.md)
- [Releases](resources/releases.md)

## Community

- [HOPR Games](community/hopr-games/README.md)
  - [Bounties](community/hopr-games/bounties/README.md)
    - [Bouncer Bot](community/hopr-games/bounties/bouncer-bot.md)
- [Past Testnets](community/past-testnets/README.md)
  - [Säntis Testnet \(Ended 06 Oct 2020\)](community/past-testnets/saentis-testnet.md)
  - [Basòdino Testnet \(Ended 4th Nov 2020\)](community/past-testnets/basodino-testnet-runs-20th-oct-4th-nov/README.md)
    - [Prize Fund and Scoreboard](community/past-testnets/basodino-testnet-runs-20th-oct-4th-nov/prize-fund-and-scoreboard.md)
  - [Basòdino Testnet v2 \(Ended 23rd Nov 2020\)](community/past-testnets/basodino-testnet-v2-runs-9th-nov-23rd-nov.md)

## QA

- [Testing HOPR](qa/testing-hopr/README.md)
  - [Filling a QA checklist](qa/testing-hopr/filling-a-qa-checklist.md)
