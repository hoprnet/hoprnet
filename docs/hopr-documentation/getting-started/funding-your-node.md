---
description: Funding your node with xDAI
---

# Funding Your Node

HOPR testnets run on xDAI Chain, a sidechain of Ethereum. In order to participate in the testnet, you'll need to fund your node with 0.02 xDAI.

First, you'll need to load some ETH into your Ethereum wallet. [MetaMask](https://metamask.io/) is one of the most widely supported wallets, but there are other options.

Next, you'll need to convert some ETH into xDAI. There are several tools you can use to do this. The simplest is to connect your wallet to the tool at [xdai.io](https://xdai.io), where you can swap ETH to DAI and then DAI to xDAI.

![](../.gitbook/assets/xdai-burner%20%282%29%20%281%29.png)

Finally, you'll need to send some xDAI to your node. Because xDAI is a separate chain, you'll need to change the network settings in MetaMask. [The xDAI docs have a step-by-step guide for this](https://www.xdaichain.com/for-users/wallets/metamask/metamask-setup).

Once you've sent xDAI to your node, restart **HOPR Chat** or your **HOPR PC Node**. When your node restarts, your balance will be automatically detected and you can proceed to the [registration stage](coverbot.md).

Later, you can check your balance by typing `balance`\(because the HOPR mainnet will run on the Ethereum mainchain, you'll currently see your balance described as ETH rather than xDAI\).

To withdraw funds from your node, use the `withdraw` command.

{% tabs %}
{% tab title="Withdrawing xDAI" %}
To withdraw xDAI, simply specify the amount and the destination address. Because HOPR is designed to run on Ethereum, you need to type ETH as the currency parameter.

```text
withdraw [amount] ETH [ETH address]
```

{% endtab %}

{% tab title="Withdrawing HOPR" %}
To withdraw HOPR, you'll need to add HOPR to your wallet so it can recognise your balance. The smart contract address is: 0x12481c3Ed97b32D94E71C2039DBC44432ADD39a0

To withdraw, type:

```text
withdraw [amount] HOPR [ETH address]
```

{% endtab %}
{% endtabs %}
