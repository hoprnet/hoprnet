---
description: Funding your AVADO node
---

# Funding Your AVADO Node

Next, you need to send tokens to your HOPR node. To use HOPR, you'll need two types of token:

* a HOPR token
* the [**native token**](../core-concepts/tokens/native-tokens.md) of the blockchain the HOPR token is linked to

In our current testnet, the native token is xDAI \(xDAI is a sidechain of Ethereum\) and the HOPR token is xHOPR\).

You currently need 0.02 xDAI in your node address to participate in the testnet. If your node doesn't have enough xDAI, it won't start.

{% hint style="info" %}
xDAI is a USD stablecoin, so 0.02 xDAI is worth around 2 cents. It costs xDAI to open payment channels and perform certain other testnet actions, but 0.02 is more than enough.
{% endhint %}

If your node is unfunded, you can find your xDAI address by simply starting the HOPR dApp. Your AVADO node will report that it is unfunded and won't proceed. It will tell you your address, so you can send xDAI.

![](../.gitbook/assets/avado-no-funds%20%282%29%20%282%29.png)

You can ask for xDAI in our [**Telegram**](https://t.me/hoprnet) or [**Discord**](https://discord.gg/dEAWC4G) channels. A HOPR ambassador will be glad to fund your wallet. If you need more instructions on how to buy and send xDAI, head [**here**](../core-concepts/tokens/native-tokens.md#getting-xdai).

{% hint style="warning" %}
You'll spend a \(very\) small amount of xDAI when you perform actions which interact with the HOPR smart contracts, such as opening and closing payment channels and redeeming tickets after relaying data.

Currently, HOPR Chat doesn't always notify you if an action has failed due to lack of funds. So if things aren't behaving as expected use `balance` to check you aren't low on xDAI. More notifications will be added in future versions.
{% endhint %}

Once you've sent xDAI to your node, restart the HOPR dApp from MyDapps in your AVADO dashboard. When your node restarts, your balance will be automatically detected and you can proceed.

## Getting xHOPR

There are two ways to get xHOPR on your node: sending it directly to the node address or earning it by relaying data and redeeming tickets. For testing purposes, we recommend funding your node directly. Ask in Telegram or Discord and one of our ambassadors will fund your node.

To learn more about earning xHOPR by relaying data, see the sections on [**Multi-hop messages**](sending-a-multi-hop-message.md) and [**Tickets**](../hopr-chat-tutorial/redeeming-tickets.md).

## Checking Your Balance

You can check your balance by typing `balance`\(because the HOPR mainnet will run on the Ethereum mainchain, you'll currently see your balance described as ETH rather than xDAI\).

## Withdrawing Funds

To withdraw funds from your node, use the `withdraw` command.

