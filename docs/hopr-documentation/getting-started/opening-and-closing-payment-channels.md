---
description: How to open and close payment channels
---

# Opening and Closing Payment Channels

Sending data privately through the HOPR network involves multiple hops via relay nodes. Nodes are incentivized for providing this service, so hops are only possible between nodes with open and funded payment channels. Payment channels are funded with HOPR tokens, and any unspent HOPR tokens are returned once the payment channel is closed. To fund your node with HOPR tokens see [**Funding Your Node**](funding-your-node.md).

## Opening a Channel

To open a channel with another node, you need to specify the node address and the amount of HOPR you want to fund the channel with.

The syntax for this is currently different between HOPR Chat and a HOPR Avado node.

{% tabs %}
{% tab title="HOPR Chat" %}
On HOPR Chat, start by typing:

```text
open <peer ID>
```

Where `<peer ID>` is the HOPR address of the node you want to open a payment channel with.

You will then be asked what amount of HOPR you want to fund the payment channel with. You cannot choose 0. If your HOPR node has a zero balance, type `exit` to cancel the process, then [**fund your node**](funding-your-node.md) and try again.
{% endtab %}

{% tab title="AVADO Node" %}
On an AVADO Node, type:

```text
open <peer ID> <amount>
```

Where `<peer ID>` is the HOPR address of the node you want to open a payment channel with and `<amount>` is the amount of HOPR tokens to fund the payment channel with, which must be greater than 0.
{% endtab %}
{% endtabs %}

Opening a payment channel involves an interaction with the HOPR smart contract, so this can take some time. You will be notified when the channel has been opened and will receive a receipt. You can use this to view the transaction on the xDAI chain, by visiting an xDAI block explorer such as [**this one**](https://blockscout.com/poa/xdai/).

## Closing a Channel

To close a channel, type:

```text
close <peer ID>
```

Where &lt;peer ID&gt; is the address of the node you have a channel open with that you want to close. You will receive a receipt for this transaction, and the channel's status will change to PENDING.

{% hint style="danger" %}
Because you can initiate a channel closure unilaterally, there is a cool-off period of two minutes, during which the node at the other end of the channel can conclude any business which requires the channel to be open.
{% endhint %}

After two minutes, you can use the `close` command again to fully close the channel.

Once a payment channel is closed, any unspent HOPR tokens in the channel will be returned to your balance. You can check this by typing `balance`.

## Viewing Your Channels

To view your currently opened payment channels, type:

```text
openChannels
```

You will then be shown a list of open channels, along with their status. The two status options are:

- **OPEN -** The channel is currently open and funded, so data can be relayed between your node and the target node.
- **PENDING** - A request has been placed to close the channel. After the cool-off period \(currently 2 minutes\), the channel is able to be closed.
