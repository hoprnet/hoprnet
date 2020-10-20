---
description: Funding your node with native and HOPR tokens
---

# Funding Your Node

Next, you need to send tokens to your HOPR node. To use HOPR, you'll need two types of token:

* a HOPR token
* the native token of the blockchain the HOPR token is linked to

In our current testnet, the native token is xDAI \(xDAI is a sidechain of Ethereum\) and the HOPR token is xHOPR\).

You currently need 0.02 xDAI in your node address to participate in the testnet. If your node doesn't have enough xDAI, HOPR Chat will not start.

{% hint style="info" %}
xDAI is a USD stablecoin, so 0.02 xDAI is worth around 2 cents. It costs xDAI to open payment channels and perform certain other testnet actions, but 0.02 is more than enough.
{% endhint %}

If your node is unfunded, you can find your xDAI address by simply starting the HOPR Chat client. HOPR Chat will recognize that your node is unfunded, and won't proceed. It will tell you your address, so you can send xDAI. Once your node is funded, you can find your address by typing `myAddress`.

![](../.gitbook/assets/no-funds%20%283%29%20%282%29.png)

You can ask for xDAI in our [**Telegram**](https://t.me/hoprnet) or [**Discord**](https://discord.gg/dEAWC4G) channels. A HOPR ambassador will be glad to fund your wallet. If you need more instructions on how to buy and send xDAI, head [**here**](../core-concepts/tokens/native-tokens.md#getting-xdai).

{% hint style="warning" %}
You'll spend a \(very\) small amount of xDAI when you perform actions which interact with the HOPR smart contracts, such as opening and closing payment channels and redeeming tickets after relaying data.

Currently, HOPR Chat doesn't always notify you if an action has failed due to lack of funds. So if things aren't behaving as expected use `balance` to check you aren't low on xDAI. More notifications will be added in future versions.
{% endhint %}

Once you've sent xDAI to your node, restart **HOPR Chat** or your **HOPR PC Node**. When your node restarts, your balance will be automatically detected and you can proceed.

Later, you can check your balance by typing `balance`\(because the HOPR mainnet will run on the Ethereum mainchain, you'll currently see your balance described as ETH rather than xDAI\).

## Getting xHOPR

There are two ways to get xHOPR on your node: sending it directly to the node address or earning it by relaying data and redeeming tickets. For testing purposes, we recommend funding your node directly. Ask in [**Telegram**](https://t.me/hoprnet) or [**Discord**](https://discord.gg/dEAWC4G) and one of our ambassadors will fund your node.

To learn more about earning xHOPR by relaying data, see the sections on [**Multi-hop messages**](opening-and-closing-payment-channels.md) and [**Tickets**](redeeming-tickets.md).

## Checking Your Balance

You can check your balance by typing `balance`\(because the HOPR mainnet will run on the Ethereum mainchain, you'll currently see your balance described as ETH rather than xDAI\).

## Withdrawing Funds

To withdraw funds from your node, use the `withdraw` command.

{% tabs %}
{% tab title="Withdrawing xDAI" %}
To withdraw xDAI, simply specify the amount and the destination address. Because HOPR is designed to run on Ethereum, you need to type ETH as the currency parameter.

```text
withdraw [amount] ETH [ETH address]
```
{% endtab %}

{% tab title="Withdrawing xHOPR" %}
To withdraw xHOPR, you'll need to add xHOPR to your wallet so it can recognise your balance. The smart contract address is: 0x12481c3Ed97b32D94E71C2039DBC44432ADD39a0

To withdraw, type:

```text
withdraw [amount] HOPR [ETH address]
```
{% endtab %}
{% endtabs %}

