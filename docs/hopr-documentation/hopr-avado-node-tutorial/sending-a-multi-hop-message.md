---
description: Send your first multi-hop message
---

# Sending A Multi-hop Message Using A Payment Channel

When you send data via HOPR, there needs to be an open and funded payment channel between every two nodes in the route \(except the penultimate node and the recipient\). For example, if we send a message from Alejandro to Zoe via Betty, Chao and Dmitry, there needs to be three open funded channels: between Alejandro and Betty, Betty and Chao, and Chao and Dmitry.

This is because HOPR is an incentivized mixnet, and users need to be able to prove they have the funds available to send data. \(Also, opening and closing payment channels happens on the blockchain, which takes time. It's not possible to open payment channels on demand and still send data quickly.\)

{% hint style="danger" %}
Payment channels need to be funded with HOPR tokens. Before we proceed, make sure you have xHOPR in your node. You can type `balance` to check.

These xHOPR won't be spent \(RandoBot has no way to redeem tickets to claim xHOPR\), but you still need to stake them in the channel. To get xHOPR, ask in our Telegram or Discord channel. Remember you need to send to you wallet \(xDAI\) address, not the HOPR address.
{% endhint %}

## Open A Payment Channel To RandoBot

To open a payment channel on your AVADO node, you need to type `open`, followed by the address of the node you want to open the channel to, followed by the amount of xHOPR you want to fund the channel with.

So to open a channel to RandoBot and fund it with 0.01 xHOPR, type:

```text
open 16Uiu2HAmNtoQri1X4ikUzCqjFQptRSLSVKnVzMmtZiCHCHkdWJr7 0.01
```

This can take a few seconds to work, because your node will need to interact with the HOPR smart contract. Once it does, you'll see a notification that the channel has been opened, along with a receipt.

![Opening a channel to RandoBot](../.gitbook/assets/avado-channel-to-randobot%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29.png)

{% hint style="info" %}
You can verify any of the receipts you get on an xDAI block explorer. For example: [https://blockscout.com/poa/xdai/](https://blockscout.com/poa/xdai/)
{% endhint %}

Type `balance`, and you'll see that your xHOPR balance has reduced by the amount you funded the channel with. Your xDAI balance will also have slightly decreased. This is the fee for opening the payment channel.

## Check Your Channel Status

You can check on your payment channels by typing `openChannels`:

![Currently open channels](../.gitbook/assets/avado-open-channels%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29.png)

## Send A Multi-Hop Message

Now let's send your first multi-hop message! We'll send it from your node, via randobot, and back to your node. Find your address using `myAddress`. Then type:`send <your address> <intermediate node> <your message>`.

{% hint style="info" %}
To make this simpler, you can also set an alias for your own address. For a refresher on aliases, visit [**this page**](talking-with-randobot.md#step-3-set-an-alias).
{% endhint %}

Congratulations! You've sent your first multi-hop message on the HOPR network.

## Close the Payment Channel

When you send a multi-hop message, you have to provide a payment for every node along the route. These payments are deducted from the tokens you staked when you opened your payment channel. But it's unlikely you'll have spent all of those tokens. To claim the remainder back, you need to close the payment channel. Type `close <peer ID>` to initiate closure. In this case, the Peer ID is RandoBot's address.

```text
> close 16Uiu2HAmNtoQri1X4ikUzCqjFQptRSLSVKnVzMmtZiCHCHkdWJr7
Initiated channel closure, receipt: 0xb62fb7c764118dffef63348c1ecaad0caba84d1ee7d3049a3cc916694ba9fea6
```

You will get a notification that the channel is being closed, along with a receipt.

If you now check the status of your channel by typing `openChannels`, you'll see the status has changed to `PENDING`.

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

