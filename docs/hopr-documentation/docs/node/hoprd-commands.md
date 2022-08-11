---
id: hoprd-commands
title: hoprd Commands
---

This page gives a short overview of the different commands you can run from your node, and the syntax for using them.

:::info

To access the list of available commands, type `help`. If you incorrectly enter a command, your node will try to suggest the correct syntax. The list below is grouped by function. Type commands only with lowercase letters.

:::

:::caution Warning

The HOPR client is still under development. Please do NOT add funds to the HOPR node that you can’t lose.

:::

## address

Type address to see the two addresses associated with your node.

```
address
```

Output will look similar to this:

```
HOPR Address:  M6psb
ETH Address:   0x81057b10E5ed35949C1a75b114818dc553755016
```

The **HOPR address** is used for interacting with other nodes in the HOPR network which includes sending and receiving messages. By default, this only shows the last five characters. Click them to expand and see the full address.

The **ETH address** is your native address, used for funding with native and HOPR tokens.

## alias

You can use the alias command to give an address a more memorable name.

```
alias [HOPR address] [alias]
```

**HOPR address** - starts with **16Uiu2HA...**.

**alias** - alias or name, you want to assign to a HOPR address.

Example:

```
alias 16Uiu2HAmMBYpQVq7rfFxV5iP3JPXJKs1dqRe2Z6HX7zXJgwjTzjV Bob
```

Your node will assign the alias (name) to that address. You can now use that alias in commands like ping, send or open payment channel, instead of typing the full address.

For example, if you want to ping the address `16Uiu2HAmMBYpQVq7rfFxV5iP3JPXJKs1dqRe2Z6HX7zXJgwjTzjV` which was assigned to `Bob`, you will now only need to execute the command:

```
ping Bob
```

You can also check all your alias by typing command:

```
alias
```

Output will look similar to this:

```
me ->  16Uiu2HAm1HQ9f5g1DC75Un92MDDHaEmggNEiiiDHSasR2kWXzoij
Bob -> 16Uiu2HAmMBYpQVq7rfFxV5iP3JPXJKs1dqRe2Z6HX7zXJgwjTzjV
```

:::info

Note that these aliases are not available publicly, and will reset when you restart your node.

:::

## balance

Type balance to display your current HOPR and native balances.

```
balance
```

:::info

The HOPR client is running by default on xDAI, so balance will show the xDAI balance as well as the wxHOPR balance.

:::

## channels

Type channels to see your currently open channels. You’ll see the node that each channel is open to and the amount with which they’re funded.

```
channels
```

## close

This command will let you close an open channel.

```
close [HOPR address]
```

**HOPR address** - to close payment channel with HOPR node you have previously opened. Address starts with **16Uiu2HA...**.

Once you’ve initiated channel closure, you have to wait for a specified closure time, it will show you a closure initiation message with cool-off time you need to wait.

Then you will need to send the same command again to finalize closure. This is a cool down period to give the other party in the channel sufficient time to redeem their tickets.

## help

Type help to display all the commands with brief explanation.

```
help
```

## info

Type info to display information about your HOPR Node, including the bootstrap node it’s currently connected to.

```
info
```

Output will look similar to this:

```
Announcing to other nodes as: /ip4/192.168.0.1/tcp/9091/p2p/M6psb,/p2p/MWXqD/p2p-circuit/p2p/M6psb,/p2p/xwsg5/p2p-circuit/p2p/M6psb,/p2p/XwAcq/p2p-circuit/p2p/M6psb,/p2p/6JHN6/p2p-circuit/p2p/M6psb,/p2p/FaQkw/p2p-circuit/p2p/M6psb,/p2p/a3J2a/p2p-circuit/p2p/M6psb,/ip4/10.19.0.5/tcp/9091/p2p/M6psb,/ip4/10.114.0.2/tcp/9091/p2p/M6psb
Listening on: /ip4/0.0.0.0/tcp/9091/p2p/M6psb
Running on: xdai
HOPR Token: 0xD4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1
HOPR Channels: 0x4f98F01cb02083eb5457CA0DDC6B37073ec5388a
Channel closure period: 5 minutes
```

## open

Open a payment channel to the specified node and fund it with the specified amount of HOPR tokens. Make sure you have sufficient native tokens to pay the gas fees.

```
open [HOPR address] [amount]
```

**HOPR address** - starts with **16Uiu2HA...**.

**amount** - the amount of HOPR tokens you are willing to fund the payment channel.

Example:

```
open 16Uiu2HAmMBYpQVq7rfFxV5iP3JPXJKs1dqRe2Z6HX7zXJgwjTzjV 0.04
```

## peers

Type peers to see a list of nodes your node has discovered and established a connection to.

```
peers
```

Output will look similar to this:

```
current nodes:
- id: jTzjV, quality: 1.00 (backoff 2, 99% of 514)
- id: qmf5r, quality: 1.00 (backoff 2, 94% of 503)
- id: T7ZVk, quality: 1.00 (backoff 2, 96% of 511)
- id: MWXqD, quality: 1.00 (backoff 2, 86% of 493)
...

22 peers have announced themselves on chain:

/ip4/34.65.239.77/tcp/9091/p2p/T7ZVk
/ip4/34.65.27.47/tcp/9091/p2p/qmf5r
/ip4/34.65.38.133/tcp/9091/p2p/XwAcq
/p2p/SZiPp
/ip4/34.65.75.129/tcp/9091/p2p/MWXqD
/ip4/176.63.10.184/tcp/32013/p2p/6JHN6
/ip4/34.65.83.15/tcp/9091/p2p/jTzjV
...
```

**Current nodes**: Your node will use this list of peers when it attempts to send and route messages and automatically open payment channels.

**Nodes which announced themselves on chain**: This is a list of peers which announced themselves on chain. This means that not all the peers are connected with your node, but announced it's existence on the chain.

## ping

Type ping and HOPR address to attempt to ping another node.

```
ping [HOPR address]
```

**[HOPR address]** - starts with **16Uiu2HA...**.

Output will look similar to this:

```
Pong received in: 87 ms
```

You should receive a pong and a latency report. This can be used to assess the health of the target node and your own node.

## redeemTickets

Type redeemTickets to attempt to redeem your earned tickets for HOPR. Make sure you have sufficient native currency in your balance to cover the gas fees.

```
redeemTickets
```

## send

There are different types of sending a message.

- send a direct message
- send a HOP message

### Send a direct message

```
send ,[HOPR address] [message]
```

:::info

When running a command, make sure to use HOPR address and write a message without brackets "**[]**".
Between comma sign (**,**) and HOPR address can't be any space.

:::

Example:

```
send ,16Uiu2HAkuiKZnPRoeV8qjXNJaazo2D6G89UG4bLoyUmza1hM6psb Hello Bob!
```

Output will look similar to this:

```
Message sent
```

Sending a direct message means it is free of charge, because it is not mixed and private. If you want receiver to know who sent it, ensure that you’ve set your `includeRecipient` setting to **true**.

### Send a HOP message

```
send [HOP address],[Recipient address] [message]
```

**HOP address** - HOPR node address through which data packed will be relayed.

**Recipient address** - HOPR node address of a recipient, who will receive a message.

:::info

When running a command, make sure to use HOPR addresses and write a message without brackets "**[]**".
Between comma sign (**,**) and HOPR addresses can't be any space.

:::

Example:

```
send 16Uiu2HAmHxdher8zPBKLqRJjTH24e5YmzhZJHwrCYzcFmeGMWXqD,16Uiu2HAkuiKZnPRoeV8qjXNJaazo2D6G89UG4bLoyUmza1hM6psb Hello Bob!
```

Sending a HOP message means it will cost you some HOPR tokens, because it is mixed and your message data packets were relayed through a `[HOP address]`.

## settings

Type settings to see your current settings.

```
settings
```

Output will look similar to this:

```
includeRecipient  false    Prepends your address to all messages (true|false)
strategy          passive  Set an automatic strategy for the node. (passive|promiscuous)
```

This will show whether you’re currently including your address with sent messages (_includeRecipient true / false_), and your current channel opening strategy (_promiscuous / passive_).

To change your `includeRecipient` setting, type:

```
settings includeRecipient true
```

or

```
settings includeRecipient false
```

To change your funding strategyn type:

```
settings strategy promiscuous
```

or

```
settings strategy passive
```

**Passive and Promiscuous strategies**

By default, hoprd runs in **passive** mode, this means that your node will not attempt to open or close any channels automatically. When you set your strategy to **promiscuous** mode, your node will attempt to open channels to a _randomly_ selected group of nodes which you have a healthy connection to. At the same time, your node will also attempt to close channels that are running low on balance or are unhealthy.

## sign

Sign your personal non-custodial wallet address to receive rewards in NFT's. This feature is available on the certain events. Please follow the current release if it has this feature.

```
sign [address]
```

**address** - starts with **0x...**

## tickets

Type tickets to display information about your redeemed and unredeemed tickets. Tickets are earned by relaying data and can be redeemed for HOPR tokens.

```
tickets
```

Output will look similar to this:

```
finding information about tickets...

Tickets:
- Pending:          0
- Unredeemed:       1
- Unredeemed Value: 0.01 txHOPR
- Redeemed:         0
- Redeemed Value:   0.00 txHOPR
- Losing Tickets:   0
- Win Proportion:   0%
- Neglected:        0
- Rejected:         0
- Rejected Value:   0 txHOPR
```

This will show Tickets statistics on the HOPR node.

## version

Type version to see the version of hoprd that you’re running.

```
version
```

Output will look similar to this:

```
1.84.1
```

## withdraw

```
withdraw [amount] [native/hopr] [address]
```

**amount** - the amount is for native or hopr tokens to withdraw.

**native/hopr** - you can withdraw **native** tokens (currently on xDai chain it will be xDai token) and **hopr** tokens (currently on xDai chain it will be wxHOPR tokens).

**address** - the receiver address which starts with **0x...**

Example sending xDai tokens:

```
withdraw 0.1 native 0xc8Aa5a085c23dfEa903312a73EfC30888bB61f0B
```

Example sending HOPR tokens:

```
withdraw 0.1 hopr 0xc8Aa5a085c23dfEa903312a73EfC30888bB61f0B
```

Ensure you have sufficient native tokens in your balance to pay for the gas fees (native tokens, current chain xDai tokens).
