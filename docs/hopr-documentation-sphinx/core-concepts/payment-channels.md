# Payment Channels

Sending data privately through the HOPR network involves multiple hops via relay nodes. Nodes are incentivized for providing this service, so hops are only possible between nodes with open and funded payment channels. Payment channels are funded with HOPR tokens, and any unspent HOPR tokens are returned once the payment channel is closed.

## Auto Channel Opening

By default, your HOPR node has a 'promiscuous' funding strategy, where it will automatically open payment channels to healthy nodes it is aware of, until it reaches the a cap of 5. You can turn this feature off by typing `settings strategy passive`, and turn it back on by typing `settings strategy promiscuous`.

## Manually Opening a Channel

You can also manually open payment channels to specific nodes. To manually open a channel with another node, you need to specify the node address and the amount of HOPR you want to fund the channel with.

```text
open <peer ID> <amount>
```

Where `<peer ID>` is the HOPR address of the node you want to open a payment channel with and `<amount>` is the amount of HOPR tokens to fund the payment channel with, which must be greater than 0.

Opening a payment channel involves an interaction with the HOPR smart contract, so this can take some time. You will be notified when the channel has been opened and will receive a receipt. You can use this to view the transaction on the blockchain.

## Viewing Your Channels

To view your currently opened payment channels, type:

```text
channels
```
You will then be shown a list of open channels, along with their status. The two status options are:

- **OPEN -** The channel is currently open and funded, so data can be relayed between your node and the target node.
- **PENDING** - A request has been placed to close the channel. After the cool-off period \(currently 2 minutes\), the channel is able to be closed.

## Closing a Channel

To close a channel, type:

```text
close <peer ID>
```

Where &lt;peer ID&gt; is the address of the node you have a channel open with that you want to close. You will receive a receipt for this transaction, and the channel's status will change to PENDING.

After two minutes, you can use the `close` command again to fully close the channel.

Once a payment channel is closed, any unspent HOPR tokens in the channel will be returned to your balance. You can check this by typing `balance`.

This can take a few seconds to work, because your node will need to interact with the HOPR smart contract. Once it does, you'll see a notification that the channel has been opened, along with a receipt.