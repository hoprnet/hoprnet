---
id: hoprd-commands
title: Hopr-admin commands
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

Output:

```
HOPR Balance:    3.0
Native Balance:  0.009595814498026512
```

:::info

The HOPR client is running by default on xDAI, so balance will show the xDAI balance as well as the wxHOPR/mHOPR balance.

:::

## channels

Type channels to see your currently open channels. You’ll see the node that each channel is open to and the amount with which they’re funded.

```
channels
```

Output:

```
fetching channels...

Outgoing Channel:  0xb0e3f7d81f0bd6d1783f3d44cf11653128e4f9ee95b98d49a07e4a8323cceb01
To:                jTzjV
Status:            Open
Balance:           0.2

Incoming Channel:  0x781ffaf4c4fe17f7c10349db8a17e2559369b08304c391e8ef06fe43f3d6d115
To:                pV3Zs
Status:            Open
Balance:           0.03
```

An `Outgoing Channel` is a channel from your node to another. The funds staked in this channel can be used to pay the node the channel is connceted to.

An `Incoming Channel` is a channel from another node to your node. The funds staked in ths channel can be used to pay your node for relaying data,

You may also see nodes with the status `PendingToClose` if they are closing or `WaitingForCommitment` if they are in the process of opening.

## close

This command will let you close an open channel from either sides.

```
close [HOPR address] [direction "incoming" or "outgoing"]
```

**HOPR address** - the address of the node you have an open channel with.
**direction** - you can close payment channels from either side: outgoing or incoming. (Currently only the outgoing direction is enabled)

Example use:

```
close 16Uiu2HAmMrqMH5xS1PeavQBY7WrQLiAKQXxBgEiMUtEWFzDZA9Mc outgoing
```

Output:

```
Closing channel to "ZA9Mc"..

Initiated channel closure, the channel must remain open for at least 5 minutes. Please send the close command again once the cool-off has passed. Receipt: "0x952e3b308302cd98ca05b3c99e62efac3047e05fb75df16b9055ea4018a29ec6".
```

Once you’ve initiated channel closure, you have to wait for a specified closure time, it will show you a closure initiation message with cool-off time you need to wait.

Then you will need to send the same command again to finalize closure. This is a cool down period to give the other party in the channel sufficient time to redeem their tickets.

Final Output after sending the same command to finalize closure:

```
Closing channel to "ZA9Mc"..

14:31:19.113Z
Closed channel. Receipt: 0xbb0adc621eac15fdf81b0284234a0e024eebd2ca87ca00fe8d31020ef89ac71a
```

## fund

You can use the fund command to add tokens to either side of the payment channel. The fund command also opens payment channels by default if they don't exist, (note: this will cost native tokens to pay the gas fee)

```
fund [HOPR address] [Amount of HOPR tokens for outgoing channel] [Amount of HOPR tokens for incoming channel]
```

**HOPR address** - open payment channel with specified node. Address starts with **16Uiu2HA...**

**Amount of HOPR tokens for outgoing channel** - funds payment channel from your HOPR node to the counterparty's node.

**Amount of HOPR tokens for incoming channel** - funds payment channel from the counterparty's node to your node.

Example: funding an outgoing channel:

```
fund 16Uiu2HAmMBYpQVq7rfFxV5iP3JPXJKs1dqRe2Z6HX7zXJgwjTzjV 0.1 0
```

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
Announcing to other nodes as           /ip4/127.0.0.1/tcp/19091/p2p/N7vkA  /ip4/172.17.0.4/tcp/19091/p2p/N7vkA  /p2p/ZZdJQ/p2p-circuit/p2p/N7vkA  /p2p/VpKCW/p2p-circuit/p2p/N7vkA  /p2p/uLoho/p2p-circuit/p2p/N7vkA  /p2p/Utqar/p2p-circuit/p2p/N7vkA
Listening on                           /ip4/0.0.0.0/tcp/19091/p2p/N7vkA
Running on                             hardhat
Using HOPR environment                 hardhat-localhost
Channel closure period                 1 minutes
HOPR Token Contract Address            0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512
HOPR Channels Contract Addresss        0xC3bBD9F2c8553AeDe3f5EF612ED455047bF70150
HOPR NetworkRegistry Contract Address  0xA51c1fc2f0D1a1b8494Ed1FE312d7C3a78Ed91C0
NetworkRegistry Eligibility            true
Connectivity Status                    Yellow
```

**Channel closure period** - The amount of time you have to wait after sending the close command before the channel officially closes.

**NetworkRegistry Eligibility** - True or false, tells you whether or not you are registered on the Network Registry for this network.

**Connectivity Status** - Unknown, Red, Orange, Yellow or Green. Depending on the health of your connection to the network.

- `unknown`: Initial value when the node is started. It means the connectivity could not be assessed.
- `Red`: No connection to any nodes at all.
- `Orange`: Low-quality (<= 0.5) connection to at least one public node.
- `Yellow`: High-quality connection to at least one public node.
- `Green`: High-quality connection to at least one public node and at least one non-public node.

The `connection`, in this case, means a node's ability to complete a ping/pong regardless of whether they are sending or receiving the ping.

And connection quality is measured from 0 to 1 based on the ratio of successful pings to the total number of pings. E.g. a node that responds to half of all pings it receives from node A will have a connection quality of 0.5 to node A.

Low-quality connection: <= 0.5
High-quality connection: > 0.5

**Note:** You can not transition to the state `unknown`, only from. All other states can be transitioned to/from in both directions.

## open

Open a payment channel to the specified node and fund it with the specified amount of HOPR tokens. Make sure you have sufficient native tokens to pay the gas fees.

```
open [HOPR address] [amount]
```

**HOPR address** - the HOPR of the node you want to open a payment channel with.

**amount** - the amount of HOPR tokens you want to stake in the payment channel.

Example:

```
open 16Uiu2HAmMBYpQVq7rfFxV5iP3JPXJKs1dqRe2Z6HX7zXJgwjTzjV 0.04
```

This will open a payment channel to `16Uiu2HAmMBYpQVq7rfFxV5iP3JPXJKs1dqRe2Z6HX7zXJgwjTzjV` with 0.04 HOPR staked in the channel.

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

**[HOPR address]** - The HOPR address of the node you want to ping, (usually starts with **16Uiu2HA...**).

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
includeRecipient   false    Prepends your address to all messages (true / false)
strategy           passive  Set an automatic strategy for the node (passive / promiscuous)
autoRedeemTickets  false    By default auto redeeming tickets are disabled. (true / false)
```

This will show whether you’re currently including your address with sent messages (_includeRecipient true / false_), your current channel opening strategy (_promiscuous / passive_) and setting tickets auto redemtion (_autoRedeemTickets true / false_).

To change your `includeRecipient` setting, type:

```
settings includeRecipient true
```

or

```
settings includeRecipient false
```

If includeRecipient is `true`, the recipient of your message will know you sent them a message. Your HOPR address will be visible before the message:

```
#### NODE RECEIVED MESSAGE ####

Message: M6psb:Hello Bob!

Latency: 170 ms
```

To change your channel management strategy:

```
settings strategy promiscuous
```

or

```
settings strategy passive
```

**Passive and Promiscuous strategies**

By default, hoprd runs in **passive** mode, this means that your node will not attempt to open or close any channels automatically. When you set your strategy to **promiscuous** mode, your node will attempt to open channels to a _randomly_ selected group of nodes which you have a healthy connection to. At the same time, your node will also attempt to close channels that are running low on balance or are unhealthy.

To enabling or disabling auto redeeming, type (This setting is currently DISABLED):

```
settings autoRedeemTickets true
```

or

```
settings autoRedeemTickets false
```

By setting autoRedeemTickets to `true`, when your node receives a ticket, it will be auto-redeemed.

## sign

Sign your personal non-custodial wallet address to receive rewards in NFT's. This feature is available on the certain events. Please follow the current release if it has this feature.

```
sign [address]
```

**address** - Your personal ETH wallet address.

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

This will show the ticket statistics for your HOPR node.

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
withdraw 0.1 NATIVE 0xc8Aa5a085c23dfEa903312a73EfC30888bB61f0B
```

Example sending HOPR tokens:

```
withdraw 0.1 HOPR 0xc8Aa5a085c23dfEa903312a73EfC30888bB61f0B
```

Ensure you have sufficient native tokens in your balance to pay for the gas fees (native tokens, currently xDai tokens).
