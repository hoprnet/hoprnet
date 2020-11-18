---
description: How to withdraw funds from your node
---

# Withdrawing Funds

You can withdraw xDAI or xHOPR from your node to an Ethereum address of your choice.

To withdraw funds from your node, use the `withdraw` command.

{% tabs %}
{% tab title="Withdrawing xDAI" %}
To withdraw xDAI, simply specify the amount and the destination address. Because HOPR is designed to run on Ethereum, you need to type ETH as the currency parameter.

```text
withdraw [amount] native [ETH address]
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

