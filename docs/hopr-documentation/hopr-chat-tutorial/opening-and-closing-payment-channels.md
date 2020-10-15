---
description: How to open and close payment channels
---

# Sending a Multi-Hop Message Using a Payment Channel

When you send data via HOPR, there needs to be an open and funded payment channel between every two nodes in the route \(except the penultimate node and the recipient\). For example, if we send a message from Alejandro to Zoe via Betty, Chao and Dmitry, there needs to be three open funded channels: between Alejandro and Betty, Betty and Chao, and Chao and Dmitry.  
  
This is because HOPR is an incentivized mixnet, and users need to be able to prove they have the funds available to send data. \(Also, opening and closing payment channels happens on the blockchain, which takes time. It's not possible to open payment channels on demand and still send data quickly.\)

Sending data privately through the HOPR network involves multiple hops via relay nodes. Nodes are incentivized for providing this service, so hops are only possible between nodes with open and funded payment channels. Payment channels are funded with HOPR tokens, and any unspent HOPR tokens are returned once the payment channel is closed. To fund your node with HOPR tokens see [**Funding Your Node**](funding-your-node.md).

{% hint style="danger" %}
Payment channels need to be funded with HOPR tokens. Before we proceed, make sure you have xHOPR in your node. You can type `balance` to check.   
  
These xHOPR won't be spent \(RandoBot has no way to redeem tickets to claim xHOPR\), but you still need to stake them in the channel. To get xHOPR, ask in our Telegram or Discord channel. Remember you need to send to you wallet \(xDAI\) address, not the HOPR address.
{% endhint %}

## Opening a Channel

### Open A Payment Channel To RandoBot

To open a channel with another node, you need to specify the node address and the amount of HOPR you want to fund the channel with.

To open a payment channel in HOPR Chat, you need to type `open`, followed by the address of the node you want to open the channel to. You will then be asked for the amount of xHOPR you want to fund the channel with.  
  
So to open a channel to RandoBot and fund it with 0.01 xHOPR, type:

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
open 16Uiu2HAmNtoQri1X4ikUzCqjFQptRSLSVKnVzMmtZiCHCHkdWJr7
```

```text
close <peer ID>
```

You'll then be asked how much HOPR you want to stake, along with your balance for reference. For example:

```text
> open 16Uiu2HAmNtoQri1X4ikUzCqjFQptRSLSVKnVzMmtZiCHCHkdWJr7
How many HOPR (0.0080000000000268 HOPR available) shall get staked? : 
```

At the prompt, type an amount to stake. Fees in the testnet are minimal, so any amount should be fine. For example:

Where &lt;peer ID&gt; is the address of the node you have a channel open with that you want to close. You will receive a receipt for this transaction, and the channel's status will change to PENDING.

```text
How many HOPR (0.0080000000000268 HOPR available) shall get staked? : 0.001
<Submitted transaction. Waiting for confirmation>
............

```

{% hint style="warning" %}
The number must be larger than zero. If you don't have any HOPR to stake, you'll need to type `exit` to leave this prompt.
{% endhint %}

{% hint style="danger" %}
Because you can initiate a channel closure unilaterally, there is a cool-off period of two minutes, during which the node at the other end of the channel can conclude any business which requires the channel to be open.
{% endhint %}

This can take a few seconds to work, because your node will need to interact with the HOPR smart contract. Once it does, you'll see a notification that the channel has been opened, along with a receipt:

```text
How many HOPR (0.0080000000000268 HOPR available) shall get staked? : 0.001
<Submitted transaction. Waiting for confirmation>
.................Successfully opened channel 0x361b9b66eb914e786f420c21f5c0c780565678dd459e57d2b3ef59af96ac781a
```

After two minutes, you can use the `close` command again to fully close the channel.

{% hint style="info" %}
You can verify any of the receipts you get on an xDAI block explorer. For example: [https://blockscout.com/poa/xdai/](https://blockscout.com/poa/xdai/)
{% endhint %}

Type `balance`, and you'll see that your xHOPR balance has reduced by the amount you funded the channel with. Your xDAI balance will also have slightly decreased. This is the fee for opening the payment channel.

Once a payment channel is closed, any unspent HOPR tokens in the channel will be returned to your balance. You can check this by typing `balance`.

## Viewing Your Channels

### Check Your Channel Status

To view your currently opened payment channels, type:

You can check on your payment channels by typing `openChannels`.

```text
openChannels
```

```text
> openChannels

Channel        :  0x361b9b66eb914e786f420c21f5c0c780565678dd459e57d2b3ef59af96ac781a
CounterParty   :  16Uiu2HAmNtoQri1X4ikUzCqjFQptRSLSVKnVzMmtZiCHCHkdWJr7
Status         :  OPEN
Total Balance  :  0.001
My Balance     :  0

```

You will then be shown a list of open channels, along with their status. The two status options are:

### Send A Multi-Hop Message

Now let's send your first multi-hop message! We'll send it from your node, via randobot, and back to your node. Find your address using `myAddress`, then set an alias for it \(e.g., "myNode"\) using `alias`.

{% hint style="info" %}
For a refresher on aliases, visit [**this page**](randobot.md#step-3-set-an-alias).
{% endhint %}

Next, `send` a message to your node by typing:

```text
send myNode
```

You'll now me prompted to enter a message. Type whatever you'd like and press `Enter`

```text
> send myNode
Type your message and press ENTER to send:
Sending my first multi-hop message
```

You'll now be asked to choose the route your message will take, selecting each intermediate node in turn.

```text
Sending message to myNode ...
Please select intermediate node 0: (leave empty to exit)
```

You can only choose a node which is connected by an open payment channel. Since you only have a channel open to RandoBot, enter RandoBot's address and press `Enter`.

{% hint style="info" %}
To make things easier, you can press `Tab` to autocomplete the node address. If you only have one open channel, this will automatically fill in the address for you.
{% endhint %}

```text
Sending message to myNode ...
Sending message to 16Uiu2HAmHcHPaB9a64oMRWVESThWQyKAqKCS1QNg5q3sGiia4wce ...
Please select intermediate node 0: (leave empty to exit)
16Uiu2HAmNtoQri1X4ikUzCqjFQptRSLSVKnVzMmtZiCHCHkdWJr7     
```

You will now be asked to enter the node address for the second hop. Since there are no more payment channels to hop through, just press `Enter` and the process will end. The message will now be sent to your node via RandoBot.

```text
===== New message ======
Message: 16Uiu2HAmHcHPaB9a64oMRWVESThWQyKAqKCS1QNg5q3sGiia4wce:Sending my first multi-hop message
Latency: 298251 ms
========================
```

{% hint style="warning" %}
The latency is currently measured from the time you first enter the `send` command, and includes the time it takes to input the intermediate nodes, so don't be concerned if the value seems high.
{% endhint %}

Congratulations, you've just sent your first multi-hop message using HOPR!

### Close the Payment Channel

When you send a multi-hop message, you have to provide a payment for every node along the route. These payments are deducted from the tokens you staked when you opened your payment channel. But it's unlikely you'll have spent all of those tokens. To claim the remainder back, you need to close the payment channel. Type `close <peer ID>` to initiate closure. In this case, the Peer ID is RandoBot's address.

You will get a notification that the channel is being closed, along with a receipt.

```text
> close 16Uiu2HAmNtoQri1X4ikUzCqjFQptRSLSVKnVzMmtZiCHCHkdWJr7
Initiated channel closure, receipt: 0xb62fb7c764118dffef63348c1ecaad0caba84d1ee7d3049a3cc916694ba9fea6
```

If you now check the status of your channel by typing `openChannels`, you'll see the status has changed to `PENDING`.

* **OPEN -** The channel is currently open and funded, so data can be relayed between your node and the target node.
* **PENDING** - A request has been placed to close the channel. After the cool-off period \(currently 2 minutes\), the channel is able to be closed.

```text
> openChannels

Channel        :  0xb15a70555e5d9bd65afe19823ed3a68838ad02c863143d321146fa5bbc6110af
CounterParty   :  16Uiu2HAmNtoQri1X4ikUzCqjFQptRSLSVKnVzMmtZiCHCHkdWJr7
Status         :  PENDING
Total Balance  :  0.01
My Balance     :  0
```

The PENDING status indicates that the channel is in cool-off. This gives the counterparty a chance to claim any unredeemed payment tickets before the channel is closed \(since RandoBot cannot claim tickets, this doesn't apply here\).

{% hint style="warning" %}
The cool-off period in the test net is two minutes. Once HOPR launches, the cool-off period will be much longer and you will be notified when the counterparty initiates channel closure, to ensure everyone has a fair chance to redeem their tickets.
{% endhint %}

Once two minutes have passed, send the `close` command again:

```text
> close 16Uiu2HAmNtoQri1X4ikUzCqjFQptRSLSVKnVzMmtZiCHCHkdWJr7
Closing channel, receipt: 0x4c764bc7d3a162ec28670000ca13f2c052c0023bf6ae8dbb546532795f8f4c70
```

Now when you type `openChannels`, you should see that there are none. The channel to RandoBot has been successfully closed.

```text
> openChannels

No open channels found.
```

Finally, check your balance with `balance`. You'll see that the tokens you staked in the payment channel have been returned to your balance.

{% hint style="info" %}
Because RandoBot cannot redeem tickets, you'll get all of your staked xHOPR back, even though you sent a multi-hop message. Normally, you would expect the node at the other end of the party to redeem their tickets, reducing your balance slightly.
{% endhint %}

Next, lets reverse the process and act as a relaying node for a message sent by someone else, earning payment tickets.  


