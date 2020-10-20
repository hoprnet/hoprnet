---
description: 'Learn about why HOPR needs two kinds of tokens to run, and how to get them.'
---

# Native Tokens

In order to participate in the testnet, you'll need to fund your node with at least 0.02 MATIC.

{% hint style="warning" %}
You'll spend a \(very\) small amount of MATIC when you perform actions which interact with the HOPR smart contracts, such as opening and closing payment channels and redeeming tickets after relaying data.

Currently, HOPR Chat doesn't always notify you if an action has failed due to lack of funds. So if things aren't behaving as expected use `balance` to check you aren't low on MATIC. More notifications will be added in future versions.
{% endhint %}

## Getting MATIC

To get MATIC, you'll need to add the network to your Ethereum wallet. [MetaMask](https://metamask.io/) is one of the most widely supported wallets, but there are other options.  
  




Once you've sent MATIC to your node, restart it. When your node restarts, your balance will be automatically detected and you can proceed.

## Getting xHOPR

There are two ways to get xHOPR on your node: sending it directly to the node address or earning it by relaying data and redeeming tickets. For testing purposes, we recommend funding your node directly. Ask in Telegram or Discord and one of our ambassadors will fund your node.

To learn more about earning xHOPR by relaying data, see the sections on [**Multi-hop messages**](../../hopr-avado-node-tutorial/sending-a-multi-hop-message.md) and [**Tickets**](../../hopr-chat-tutorial/redeeming-tickets.md).

