---
id: guide-using-a-hoprd-node
title: Guide using a hoprd node
---

We will provide you an example on how to use `hoprd`. Provided example is just a one scenario which you can follow. There are no strict rules how to use `hoprd`.

:::caution Warning

The HOPR client software (hoprd) is still under heavy development. Please do not add funds to the node that you cannot lose.

:::

This is a hoprd admin user interface, which is browser based. To get the access to it you have to setup and run a `hoprd` node, details [here](start-here).

:::info Tip
Please be aware that it can take up to `10` minutes for your `hoprd` node instance to boot up.
:::

Access to the admin UI: [http://localhost:3000](http://localhost:3000)

![hoprd admin user interface](/img/node/hoprd-api-token.png)

(**1**) Enter the password you specified in the command.

(**2**) Fund your node (fund the address you see on admin UI, which starts with **0x...**) with **xDai** tokens (details [here](https://www.xdaichain.com/for-users/get-xdai-tokens)) and **wxHOPR tokens** (details [here](/staking/how-to-get-hopr)).

We recommend you to fund your node with **0.01 xDai & 10 wxHOPR**.

**Note:** After funding your node, please wait until your node will start and you will see this message: **Node has started!**

:::info Tip

Please be aware that we have two types of tokens, both live in the **xDAI/Gnosis Chain network**:

- `wxHOPR`, the ERC-777 token needed to run your `hoprd` instance and,
- `xHOPR`, the ERC-677 token, the xDAI/Ethereum bridged `HOPR` instance

You can use [cross-chains](/staking/convert-hopr) bridge to convert from HOPR to xHOPR or vice versa.

You can always use our [token wrapper](https://wrapper.hoprnet.org/) to see your balances and swap tokens between each other.

:::

Brief look at the admin UI:

![hoprd admin user interface](/img/node/hoprd-admin-ui.png)

(**3**) You can click on the HOPR logo, to see your node connected to the list of peers.

(**4**) You can type commands and execute actions.

(**5**) Executed commands will output the results here.

## Using hoprd example

:::info

Before we start, you can find all the commands explained here: [HOPRd commands](hoprd-commands).

Mentioned HOPR addresses and ETH addresses are **examples**.

Search HOPR addresses on your HOPR node by using command: `peers` and search for **current nodes**. Use two of the **current nodes** instead of the examples.

:::

**My node** - 16Uiu2HAkuiKZnPRoeV8qjXNJaazo2D6G89UG4bLoyUmza1hM6psb

**Bob's node** - 16Uiu2HAmMBYpQVq7rfFxV5iP3JPXJKs1dqRe2Z6HX7zXJgwjTzjV

**Alice node** - 16Uiu2HAm35DuQk2Cvp9aLpRTD43ZubLqtbAwf242w2YmAe8FskLs

### 1. Add alias for Bob and Alice

Instead of using HOPR address, we can assign HOPR address to a specific name, this action called add alias.

```
alias 16Uiu2HAmMBYpQVq7rfFxV5iP3JPXJKs1dqRe2Z6HX7zXJgwjTzjV Bob
```

The output will look similar to this:

```
Set alias 'Bob' to 'jTzjV'.
```

Do the same with Alice:

```
alias 16Uiu2HAm35DuQk2Cvp9aLpRTD43ZubLqtbAwf242w2YmAe8FskLs Alice
```

The output will look similar to this:

```
Set alias 'Alice' to 'FskLs'.
```

To check if you added alias correctly run command `alias` and it should give a list of assigned alias.

```
alias
```

The output will look similar to this:

```
me ->  M6psb
Bob -> jTzjV
Alice -> FskLs
```

### 2. Check your node balance

```
balance
```

The output will look similar to this:

```
HOPR Balance:  0.12 wxHOPR
ETH Balance:   0.9915287 xDAI
```

**HOPR Balance** - Called **hopr** tokens, will be used for opening payment channels, redeemedTickets will be converted to HOPR tokens.

**ETH Balance** - Called **native** tokens, it used only for gas fees on the blockchain the current release is running on. For example, when you will open or close the payment channel, it will use gas fees to execute this action.

:::info

Make sure you have some **hopr** tokens and **native** tokens on your node balance.

:::

### 3. Check your node address

```
address
```

The output will look similar to this:

```
HOPR Address:  16Uiu2HAkuiKZnPRoeV8qjXNJaazo2D6G89UG4bLoyUmza1hM6psb
ETH Address:   0x81057b10E5ed35949C1a75b114818dc553755016
```

**HOPR Address** - this address (starts with **16Uiu2HA...**) can be funded only from the other node, it can't be fund'ed from the external wallets (starts with **0x...**). You can send / receive messages, open / close payment channels.

**ETH Address** - this address have only one purpose, to fund your node from external wallets (starts with **0x...**). It **can't be used** internally to send / receive messages, open / close payment channels.

### 4. Ping Bob's node

```
ping 16Uiu2HAmMBYpQVq7rfFxV5iP3JPXJKs1dqRe2Z6HX7zXJgwjTzjV
```

Output will look similar to this:

```
Pong received in: 84 ms
```

This means, you have successfully pinged the node and the latency was only 84 ms. If your output was similar to this: `Could not ping node. Timeout.`, this means your node was not able to ping the node. Try another node until you will receive a successful ping pong message.

### 5. Send direct message to Bob

:::info

Sending a direct message means it is free of charge, because it is not mixed and private.

:::

```
send 16Uiu2HAmMBYpQVq7rfFxV5iP3JPXJKs1dqRe2Z6HX7zXJgwjTzjV Hello Bob!
```

Output will look similar to this:

```
Message sent
```

This means you have successfully sent a message to Bob. If your output was similar to this: `Could not send message. (E_TIMEOUT)`, this means your message wasn't sent. Try to send it again.

### 6. Open payment channel with Bob

```
open 16Uiu2HAmMBYpQVq7rfFxV5iP3JPXJKs1dqRe2Z6HX7zXJgwjTzjV 0.04
```

Here you opened payment channel with Bob, and your payment channel balance has 0.04 HOPR tokens. It means you will be able to send 4 HOP messages, because one HOP message will cost you 0.01 HOPR token.

Sending HOP messages is not the same as sending direct messages, sending HOP messages means when you use other node to relay your message. In other words you will be using other node "services" to send a message.

Output will look similar to this:

```
Opening channel...
Successfully opened channel 0xb0e3f7d81f0bd6d1783f3d44cf11653128e4f9ee95b98d49a07e4a8323cceb01

```

This means it has successfully opened channel. The hash you see in the output is your opened payment channel ID and not the hash you can check on the blockchain.

Between `Opening channel...` message and `Successfully opened channel ...` message can take some time, because it requires some block confirmations on the chain.

### 7. Check opened channels list

```
channels
```

Output will look similar to this:

```
fetching channels...

Outgoing Channel:       0xb0e3f7d81f0bd6d1783f3d44cf11653128e4f9ee95b98d49a07e4a8323cceb01
To:                     jTzjV
Status:                 Open
Balance:                0.03 wxHOPR

No open channels to node.
```

This means you have one opened payment channel which has status `Open` and it has a balance with 0.03 HOPR tokens.

If you will see payment channel status output similar to this: `WaitingForCommitment` it means it is not ready for connection and you have to wait until the channel status will change to `Open`.

### 8. Send 1-HOP message through a Bob's node to yourself

:::info

Between comma sign (**,**) and HOPR addresses can't be any space.

:::

```
send 16Uiu2HAmMBYpQVq7rfFxV5iP3JPXJKs1dqRe2Z6HX7zXJgwjTzjV,me This is a feature message for me!
```

Output will look similar to this:

```
Message sent

#### NODE RECEIVED MESSAGE [2021-12-11T19:53:37.393Z] ####

Message: This is a feature message for me!

Latency: 445ms
```

This means, you used Bob's node to send message to yourself, because you can't send message directly to yourself. Technically you used Bob's node to relay data packets. Specifically in this example if you haven't got message to yourself, this means it was not able to send message. Try several times until you will manage to send. For the next steps we will try to send 2-HOP.

After you successfully send 1-HOP message, the payment channel balance between yours and Bob should decreases with 0.01 HOPR token. Instead of 0.03 HOPR tokens it should show 0.02 HOPR tokens.

### 9. Ask Bob to check the Tickets

```
tickets
```

Output will look similar to this:

```
finding information about tickets...

Tickets:
- Pending:          0
- Unredeemed:       1
- Unredeemed Value: 0.01 wxHOPR
- Redeemed:         0
- Redeemed Value:   0.00 wxHOPR
- Losing Tickets:   0
- Win Proportion:   0%
- Neglected:        0
- Rejected:         0
- Rejected Value:   0 wxHOPR
```

This will show Tickets statistics on a Bob's node. On the previous step we sent a message through a Bob's node, this means we used his node to relay data packets. For this Bob got 1 ticket reward for relaying data. It will be automatically auto redeemed and Bob will receive 0.01 HOPR token.

### 10. Send 2-HOP message through Bob's and Alice nodes to yourself

:::info

Before sending 2-HOP message you have to open payment channel with Alice, see the [6th step](guide-using-a-hoprd-node#6-open-payment-channel-with-bob).

Note: Between comma sign (**,**) and HOPR addresses can't be any space.

:::

```
send 16Uiu2HAmMBYpQVq7rfFxV5iP3JPXJKs1dqRe2Z6HX7zXJgwjTzjV,
16Uiu2HAm35DuQk2Cvp9aLpRTD43ZubLqtbAwf242w2YmAe8FskLs,
me This is a feature message for me, but this time I used Bob and Alice nodes to relay data packets!
```

Output will look similar to this:

```
Message sent

#### NODE RECEIVED MESSAGE [2021-12-11T20:23:10.391Z] ####

Message: This is a feature message for me, but this time I used Bob and Alice nodes to relay data packets!

Latency: 845ms
```

This means, you used Bob's and Alice nodes to send message to yourself. Technically this means data packets was relayed through Bob's and Alice nodes. What we see here a bit different it's a latency, it received message with a greater latency, because data packets relayed through the two nodes instead of one.

### 11. Close payment channel with Bob

```
close 16Uiu2HAmMBYpQVq7rfFxV5iP3JPXJKs1dqRe2Z6HX7zXJgwjTzjV
```

You will **initiate** channel closure, but you will not close it finally. Once you’ve initiated channel closure, you have to wait for a specified closure time, then you may send the command again to **finalize** closure. This is a cool down period to give the other party in the channel sufficient time to redeem their tickets.

Output will look similar to this:

```
Closing channel...
Initiated channel closure, the channel must remain open for at least 5 minutes. Please send the close command again once the cool-off has passed. Receipt: 0xcae1745623f12936eff1b85a538d8dfe06704f11d4e34a03f24e0ec5f31a9ea6.
```

This means initiation of channel closure was successful and you need to wait at least 5 minutes for a cool-off.

After 5 minutes cool-off, run the same command to finalise closure.

```
close 16Uiu2HAmMBYpQVq7rfFxV5iP3JPXJKs1dqRe2Z6HX7zXJgwjTzjV
```

Output will look similar to this:

```
Closing channel...
Closing channel. Receipt: 0x2bbf3181024b1580eba7554f46378cef69e9c7612580eb331795459b6bd578e5.
```

This means you have successfully closed payment channel with Bob and received hash, which you can check on the blockchain. You can also check your opened channels list to be sure that you have close payment channel, see the [7th step](guide-using-a-hoprd-node#7-check-opened-channels-list).
