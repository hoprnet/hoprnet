# Commands

This page will give a short overview of the different commands you can run from your node, and the syntax for using them.

```eval_rst
.. ATTENTION::
   If you've used previous versions of HOPR based around the HOPR Chat app, please be aware that the syntax for many commands has changed. In particular, all commands which required multiple inputs in HOPR Chat must be entered on a single line in hoprd.
```

To access the list of available commands, type `help`. If you incorrectly enter a command, your node will try to suggest the correct syntax. The list below is grouped by function.

## info
Type `info` to display information about your HOPR Node, including the bootstrap node it's currently connected to.

## version
Type `version` to see the version of hoprd that you're running.

## address
Type `address` to see the two addresses associated with your node. The top address is your HOPR address, which is used for receiving messages. By default, this only shows the last five characters. Click them to expand and see the full address.

The bottom address is your native address, used for funding with native and HOPR tokens.

## settings
Type `settings` to see your current settings. This will show whether you're currently including your address with sent messages (includeRecipient true / false), and your current channel opening strategy (promiscuous / passive). To change your `includeRecipient` setting, type `settings includeRecipient true` or `settings includeRecipient false`. To change your funding strategy, type `settings strategy promiscuous` or `settings strategy passive`.

## ping
Type `ping [HOPR address]` to attempt to pings another node. You should receive a pong and a latency report. This can be used to assess the health of the target node and your own node.

## peers
Type `peers` to see a list of nodes your node has discovered and made a connection to. Your node will use this list of peers when it attempts to send and route messages and automatically open payment channels.

## send
Type `send [HOPR address] [message]` to send a message to the specified HOPR address. If you want them to know who sent it, ensure that you've set your `includeRecipient` setting to `true`. Your node will attempt to find the best route to send this message based on its knowledge of the network.

## alias
You can use the alias command to give an address a more memorable name. Type `alias [address] [alias]`. And your node will assign the alias to that address. You can now use that alias in commands like `send`, instead of typing the full address. Note that these aliases are not available publicly, and will reset when you restart your node.

## balance
Type `balance` to display your current HOPR and native balances.

## withdraw
Type `withdraw [amount] [native / hopr] [address]` to withdraw the specified amount of native or HOPR tokens to the target address. Ensure you have sufficient native tokens in your balance to pay for the gas fees.

## open
Type `open [HOPR addresss] [HOPR amount]` to manually opens a payment channel to the specified node and fund it with the specified amount of HOPR tokens. Make sure you have sufficient native tokens to pay the gas fees.

## channels
Type `channels` to see your currently open channels. You'll see the node that each channel is open to and the amount with which they're funded.

## close
Type `close [HOPR address]` to close an open channel. Once you've initiated channel closure, you have to wait at least two minutes and then send the command again. This is a cool down period to give the other party in the channel sufficient time to redeem their tickets.

## tickets
Type `tickets` to displays information about your redeemed and unredeemed tickets. Tickets are earned by relaying data and can be redeemed for HOPR tokens.

## redeemTickets
Type `redeemTickets` to attempt to redeem your earned tickets for HOPR. Make sure you have sufficient native currency in your balance to cover the gas fees.

## covertraffic
Type `covertraffic start` to being generating cover traffic messages. Type `covertraffic stop` to stop the cover traffic. You can type `covertraffic stats` to see the current status and reliability of your cover traffic.

## quit           
Type `quit` to terminate your node.










