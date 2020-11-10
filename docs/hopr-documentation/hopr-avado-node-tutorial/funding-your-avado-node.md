---
description: Funding your AVADO node
---

# Funding Your AVADO Node

Next, you need to send tokens to your HOPR node. To use HOPR, you'll need two types of token:

- a HOPR token
- the native token of the blockchain the HOPR token is linked to

In our current testnet, the native token is MATIC and the HOPR token is hosted on the Matic Network.

{% hint style="danger" %}
Although the testnet replicates many mechanisms from our eventual mainnet, it’s important to stress that **the testnet HOPR token on Matic is not our final HOPR token**. It’s completely separate, and can’t be bought or sold or transferred for HOPR. It’s just for this testnet, and has no value. HOPR prizes will be issued after the HOPR mainnet launch on Ethereum.
{% endhint %}

You currently need 0.02 MATIC in your node address to participate in the testnet. If your node doesn't have enough MATIC, HOPR Chat will not start.

{% hint style="info" %}
It costs MATIC to open payment channels and perform certain other testnet actions, but 0.02 should be more than enough.
{% endhint %}

If your node is unfunded, you can find your MATIC address by simply starting the HOPR dApp. Your AVADO node will report that it is unfunded and won't proceed. It will tell you your address, so you can send MATIC.

![This message will display when you need to fund your AVADO node with MATIC](../.gitbook/assets/avado-matic-no-funds%20%281%29.png)

You can ask for MATIC in our [**Telegram**](https://t.me/hoprnet) or [**Discord**](https://discord.gg/dEAWC4G) channels. A HOPR ambassador will be glad to fund your wallet. If you need more instructions on how to buy and send MATIC, head [**here**](../core-concepts/tokens/native-tokens.md#getting-xdai).

{% hint style="warning" %}
Currently, HOPR Chat doesn't always notify you if an action has failed due to lack of funds. So if things aren't behaving as expected use `balance` to check you aren't low on MATIC. More notifications will be added in future versions.
{% endhint %}

Once you've sent MATIC to your node, restart the HOPR dApp from MyDapps in your AVADO dashboard. The HOPR package is the one labelled `hopr.avado.dnp.dappnode.eth`.

![Restarting your HOPR dApp](../.gitbook/assets/avado-restart-hopr%20%281%29.png)

When you press restart, you'll see the following warning:

![](../.gitbook/assets/avado-restart-warning%20%281%29.png)

This is fine. Just click "Restart" to proceed.

{% hint style="warning" %}
If you run into an error at this stage, please do a hard restart on your HOPR AVADO Node by restarting the device itself.
{% endhint %}

When your node restarts, your MATIC balance will be automatically detected and you can proceed.

## Getting Testnet HOPR

There are two ways to get testnet HOPR tokens on your node: sending them directly to the node address or earning them by relaying data and redeeming tickets.

To learn more about earning testnet HOPR by relaying data, see the sections on [**Multi-hop messages**](sending-a-multi-hop-message.md) and [**Tickets**](../hopr-chat-tutorial/redeeming-tickets.md).

## Checking Your Balance

You can check your balance by typing `balance`. You'll see something like this:

## Withdrawing Funds

To withdraw funds from your node, use the `withdraw` command.
