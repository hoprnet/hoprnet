---
id: using-hopr-admin
title: How to use hopr-admin
---

This is a guide on how to use `hopr-admin`. It is not exhaustive and is intended only as a brief overview of its functionality and use.

:::caution Warning

The HOPR client software (hoprd) is still under heavy development. Please do not add funds to the node that you cannot lose.

:::

Make sure you have installed a `hoprd` node either through Docker or with a hardware node such as Avado or Dappnode. If you have not completed the installation process, please [start here](start-here).

:::info Tip
Please be aware that it can take 10 minutes for your `hoprd` node to boot up.
:::

**Network Registry:** If you have registered your node on the network registry, you will have been airdropped mHOPR/xDAI along with your NFT. In **step three** of Admin UI & Funding below, make sure to use the **mHOPR** and **xDAI** provided instead of purchasing **wxHOPR**.

## Admin UI & Funding

If you used Docker to install your node, you should be able to access it at: [http://localhost:3000](http://localhost:3000) (replace `localhost` with your server IP address if you are using a VPS). Otherwise, locate the HOPR client on your hardware node's associated browser. You should end up with an interface that looks like this:

**_Note:_** You may be greeted with a yellow screen asking you to check your settings. You can fix this by entering the correct API endpoint and token (see steps 1 & 2).

![hopr-admin user interface](/img/node/admin-UI-first-2.png)

(**1**) First, click on the gear icon. This should give you a pop-up similar to the one below.

![API info](/img/node/API-info.png)

(**2**) From here, make sure you have the correct information. By default, `API endpoint` should be set to `http://localhost:3001`, but you may need to replace `localhost` with your server IP address if you used a VPS and change the port if you adjusted the mapping on installation.

If you are using an Avado or Dappnode then the endpoints are `http://hopr.my.ava.do:3001` and `http://hopr.dappnode:3001` respectively.

The `API Token` is whatever you set your security token as in the installation process.

(**3**) You will see a newly generated ETH & Node address. Use the ETH address to send [xDAI](https://www.xdaichain.com/for-users/get-xdai-tokens) and [HOPR tokens](/staking/how-to-get-hopr) to in order to fund your node. **Your node will not start until it is funded.**

**Monte Rosa release:** If you are participating in the Monte Rosa release, send the **mHOPR** and **xDAI** you have been airdropped.

**Local network:** If using a local network, we recommend you fund your node with **0.01 xDai & 10 wxHOPR**.

After funding your node, you will have to wait a few minutes for it to start. When the process is complete, you should see the output: **Node has started!**

:::info Tip

Please be aware that we have three types of tokens on the **xDAI/Gnosis Chain network**:

- `mHOPR`, the ERC-20 token used by the monte rosa environment,
- `wxHOPR`, the ERC-777 token used by local networks,
- `xHOPR`, the ERC-677 token, the xDAI/Ethereum bridged `HOPR` token

If you are using the Monte Rosa environment, use `mHOPR`.

Otherwise, you can use the [cross-chains](/staking/convert-hopr) bridge to convert from HOPR to xHOPR or vice versa.

And you can always use our [token wrapper](https://wrapper.hoprnet.org/) to wrap/unwrap xHOPR/wxHOPR.

:::

![hopr-admin user interface](/img/node/admin-UI-second-2.png)

(**4**) You will see your `Network Health Indicator`. Depending on your connection to the network, this can be either Red, Orange, Yellow or Green. It's normal for it to be **Red** when you first start your node, you should wait a few minutes to see if it improves, but this is not required.

(**5**) Click on the HOPR logo to see your node's connected peers.

(**6**) You can type commands and execute actions here.

(**7**) Output is displayed here.

:::info

Before we start, you can find all the commands explained here: [Hopr-admin commands](hoprd-commands).

Mentioned HOPR and ETH addresses are **examples**. Make sure you replace them with the addresses you are interacting with.

:::

## Interacting with your node

Now that you have started your node, what exactly is your node and what are its features? There is a lot that goes into making the HOPR node function but let's start with the following properties:

- Identity file
- ETH address & Peer ID
- Balance

### Identity file

The **_identity file_** contains your private key and is essentially your wallet for the node. When you installed your node, you supplied `--identity` and `--password` arguments.

```bash
docker run --pull always --restart on-failure -m 2g -ti -v $HOME/.hoprd-db-bogota:/app/hoprd-db -p 9091:9091 -p 3000:3000 -p 3001:3001 -e DEBUG="hopr*" gcr.io/hoprassociation/hoprd:bogota --environment monte_rosa --init --api --admin --identity /app/hoprd-db/.hopr-id-bogota --data /app/hoprd-db --password 'open-sesame-iTwnsPNg0hpagP+o6T0KOwiH9RQ0' --apiHost "0.0.0.0" --apiToken 'YOUR_SECURITY_TOKEN' --adminHost "0.0.0.0" --healthCheck --healthCheckHost "0.0.0.0"
```

`--identity` is a path to where the file is located, and `--password` is used to decrypt the file.

If a file exists at the specified location and is decrypted using the provided password, then your existing private key is used to access your funds and start up your node.

If one doesn’t exist, it is created, encrypted and stored at the given location with a newly generated private key.

### Backing up your identity file

If you used Docker to install your node, then your identity file will be stored on your OS at the path you specified: `/<computer username>/.hoprd-db/.hopr-id-bogota`, so you can skip this step.

If you are using Dappnode or Avado, you can download your identity file on their interfaces.

**_Note:_** You should download your identity file as soon as possible. As downloading the backup or DB folder will also download the database, which can get quite large in size if you’ve been running your node for a while.

**_DAppNode –_** Find HOPR in your packages and navigate to the backup section. From there, all you have to do is click 'Download backup'. This will download a `.zip` file containing your identity file. For DAppNode, you should use this zipped file to [restore your node](using-dappnode#restoring-an-old-node) if needed.

![dappnode backup](/img/node/dappnode-backup.png)

#### How to view your DAppNode identity file:

You will not be able to use the identity file alone to [restore your old node](using-dappnode#restoring-an-old-node) on DAppNode and should use the entire zipped backup file. The instructions below are simply to view your identity file.

(**1**) Extract the zipped file downloaded to see the DB folder and identity file.

(**2**) Once extracted, open the folder: `db`.

![dappnode db folder](/img/node/Dappnode-DB-folder.png)

(**3**) You will see the file `identity` if hidden files are visible.

**_Avado –_** For Avado, you have to specify you want to download /app/hoprd-db in the Avado UI. Locate your HOPR package and click on the 'manage' icon.

![avado manage](/img/node/avado-manage.png)

From here, scroll down to the file manager and enter `/app/hoprd-db` in the field under `Download from DNP`. Then click 'Download'. This will download a `.zip` file.

![avado download](/img/node/avado-db.png)

#### How to view your Avado identity file

(**1**) Make sure hidden files are visible on your OS.

(**2**) Extract the zipped folder you just downloaded.

![Avado db folder](/img/node/Avado-DB-folder.png)

(**3**) Once extracted, open the folder `hoprd-db`.

![Avado identity file](/img/node/Avado-identity-file.png)

(**4**) If hidden files are visible on your OS, you should see a file named `.hopr-identity`. Use this to [restore your node](using-avado#alternative-method-using-your-identity-file) if needed.

**_Note:_** Make sure you enter `/app/hoprd-db` and not `/app/hoprd-db/`.

Now that you’ve backed up your identity file and have noted your password, you will always be able to access your private key and node (as long as you keep them safe). You can also use this file to [import your funds to your MetaMask wallet.](using-hopr-admin#importing-wallet-to-metamask)

### ETH address & peerID

From the private key, your **_ETH address_** and **_HOPR address_** are generated.

Your HOPR address, aka your **_peer ID,_** is what other nodes on the network will use to interact with your node—an address for them to ping or send data.

To view this information, type `address` into the admin command line.

```
address
```

Expected output:

```
HOPR Address:  M6psb
ETH Address:   0x81057b10E5ed35949C1a75b114818dc553755016
```

By default, your HOPR address will only show the last five characters. Click them to expand the full address.

### Balance

Now that you have funded your node, you can check your node's balance by typing `balance`.

```
balance
```

Expected output:

```
HOPR Balance:     0.12
Native Balance:   0.9915287
```

**HOPR Balance** – Either wxHOPR or mHOPR, depending on the network. These HOPR tokens fund payment channels/pay nodes to relay data.

**ETH Balance** – This will show the native tokens used to pay gas fees, currently xDAI. For example, opening and closing payment channels would require on-chain transactions paid for in xDAI.

## Interacting with other nodes

Now that we have gone through a few of the properties of your node let's try and interact with other nodes on the network.

### Finding other nodes

First, let’s look at the available nodes for you to connect to. Type `peers` into the command line:

```
peers
```

Expected output:

```
122 peers have announced themselves on chain:

/ip4/34.65.239.77/tcp/9091/p2p/T7ZVk
/ip4/34.65.27.47/tcp/9091/p2p/qmf5r
/ip4/34.65.38.133/tcp/9091/p2p/XwAcq
/p2p/SZiPp
/ip4/34.65.75.129/tcp/9091/p2p/MWXqD
/ip4/176.63.10.184/tcp/32013/p2p/6JHN6
/ip4/34.65.83.15/tcp/9091/p2p/jTzjV
...
```

This shows all the nodes that have announced themselves onto the network. The HOPR addresses are contracted, so click on them to expand the full address.

For this tutorial, if you aren’t using [Playground](/dapps/playground), you will have to find two or more responsive nodes on the network. You can do this by pinging other nodes.

### Pinging other nodes

Replace the following address with one from your list of `peers`

```
ping 16Uiu2HAmMBYpQVq7rfFxV5iP3JPXJKs1dqRe2Z6HX7zXJgwjTzjV
```

Expected output:

```
Pong received in: 84 ms
```

You should receive a pong and a latency report. This indicates the health of the target node as well as your own.

But the main thing to see is if you received a response or not. If your output was: `Could not ping node. Timeout`, it means you could not ping the node, and you should keep pinging other nodes until you find a responsive one.

### Setting aliases for other nodes

You can now give one of the responsive nodes an alias:

```
alias 16Uiu2HAmMBYpQVq7rfFxV5iP3JPXJKs1dqRe2Z6HX7zXJgwjTzjV Betty
```

Expected output:

```
Set alias 'Betty' to 'jTzjV'.
```

This is useful as you can use the alias in place of their HOPR address when using commands.

At this point, you should ideally have found two responsive nodes and given them both an alias. I will be using Betty and Chāo.

### Check aliases

Type alias to see a list of all the aliases you have set.

```
alias
```

Expected Output:

```
me ->  M6psb
Betty -> jTzjV
Chāo -> FskLs
```

By default, you will have the alias **me** set to your own address. If you don't see this you should set it manually as I will be using it quite often in this tutorial.

All of these aliases can be used in place of their corresponding HOPR address. For example, if you want to ping Betty’s node **16Uiu2HAmMBYpQVq7rfFxV5iP3JPXJKs1dqRe2Z6HX7zXJgwjTzjV**, you can use:

```
ping Betty
```

**_Note:_** you can assign multiple aliases to a single node, but you cannot assign the same alias to multiple nodes. E.g. two separate HOPR addresses can not be aliased `Betty`

If you want to remove an alias, use:

```
alias remove Betty
```

### Sending a direct/0-HOP message

Now that we have found nodes we want to interact with, we can send a simple direct message to one of them.

```
send ,Betty Hello Betty!
```

Expected output:

```
Message sent
```

**_Note:_** Don't forget to add the comma and use the correct alias. Betty is an alias I assigned to a previously found responsive node. Otherwise, you can just write their HOPR address:

```
send ,16Uiu2HAmMBYpQVq7rfFxV5iP3JPXJKs1dqRe2Z6HX7zXJgwjTzjV Hello Betty!
```

This is a direct message or a 0-HOP message. It sends the message directly to another node without intermediaries and is costless.

The node you messaged (in my case, `Betty`) will receive an output that looks like this:

```
#### NODE RECEIVED MESSAGE ####

Message: Hello Betty!

Latency: 295 ms
```

### Change includeRecipient type

If you want the recipient to know you sent the message, you can change your includeRecepient settings.

```
settings includeRecipient true
```

Expected output:

```
Settings updated.
```

Now, if you message a node, it will receive an output including your address:

```
#### NODE RECEIVED MESSAGE ####

Message: M6psb:Hello Betty!

Latency: 170 ms
```

By default, your includeRecipient type is set to false. You can check its current setting by typing `settings`:

```
settings
```

Expected output:

```
includeRecipient  true     Prepends your address to all messages (true|false)
strategy          passive  Set an automatic strategy for the node. (passive|promiscuous)
```

Ignore the strategy setting. For now, this is explained down below. If you want to go back to being anonymous, simply reset it to false:

```
settings includeRecipient false
```

### Payment channels & path selection

So far, you’ve only sent a direct (0-HOP) message to another node. A direct message is not mixed or private. To make your message private, you must send it through other nodes.

To use another node on the network to relay data, you have to pay them for their service in HOPR tokens. This is where payment channels come in.

### Payment channels

Payment channels are funded edges between two nodes. They are a link between two nodes with some HOPR tokens staked in them to pay the nodes that relay data for the sender.

![payment channel](/img/node/payment-channel-2.png)

### Opening a channel

When opening a channel, you need to choose a node to open the channel with and the amount of HOPR tokens you want to stake. You should stake at least 0.2 HOPR tokens to complete this tutorial.

You can open a payment channel with Betty by using:

```
open Betty 0.2
```

The general format is:

```
open [HOPR address] [amount]
```

Expected output:

```
Opening channel to node "jTzjV".

Successfully opened channel to node "jTzjV".
```

This will open a channel from you to Betty with **0.2 HOPR** staked in it. You can use these tokens to pay Betty and all the other nodes in any relay you use, where Betty is the first intermediary node.

**_Note:_** Channels are unidirectional; opening this channel does not mean a channel from Betty to your node exists.

![Channel direction](/img/node/channel-direction-2.png)

Only one channel can exist in a single direction between two nodes. You can have both a channel from Betty → Chāo & Chāo → Betty but not more than one channel from Betty → Chāo.

Once you have opened a channel to Betty, trying to open another one will fail.

```
open Betty 0.01
```

Expected output:

```
Opening channel to node "jTzjV".

Failed to open a channel to jTzjV.
```

### Closing a channel

You should have a channel open with `Betty` or either one of your responsive nodes by now with at least 0.2 HOPR tokens staked.

If you have underfunded the channel linked to Betty, you can `close` the channel and retrieve all the funds staked before opening a new channel with Betty:

```
close Betty
```

Expected output:

```
Closing channel to "jTzjV".

Initiated channel closure, the channel must remain open for at least 1 minute. Receipt: "0x4c529ee1d44249e42633b14036d9c037daf4d9f077ea853ef02ac37e458b41ba".
```

This will take a minute as your funds need to be retrieved. You can view the progress of the channel closure by checking your open channels.

### Check open channels

You can check on all your open channels by entering `channels`:

```
channels
```

You should have a single outgoing channel to Betty’s address and no incoming channels (if you haven't closed the channel).

```
fetching channels...

Outgoing Channel:       0xb0e3f7d81f0bd6d1783f3d44cf11653128e4f9ee95b98d49a07e4a8323cceb01
To:                     jTzjV
Status:                 Open
Balance:                0.2

No open channels to node.
```

If you see an incoming channel, someone has opened a channel with your node, which might have happened, but won’t affect this walkthrough too much.

You may also see channels with the status `PendingToClose` if they are closing or `WaitingForCommitment` if they are opening.

**_Note:_** If you closed your channel with Betty, make sure you have reopened it with 0.2 HOPR tokens staked before continuing this tutorial.

### Send a 1-HOP message

Now that you have an open channel with Betty, you can use them as an intermediary node to relay your message.

```
send Betty,me This is a message for me!
```

The general format is:

```
send [HOP address],[Recipient address] [message]
```

Where the `HOP address` is the node you want to relay your data through. Make sure you don't add spaces around the comma (`,`).

Expected output:

```
Message sent

#### NODE RECEIVED MESSAGE ####

Message: This is a message for me!

Latency: 445ms
```

**_Note:_** I have includeRecipient set to false. Your output might look slightly different.

In this example, we’re using Betty’s node to relay a message back to ourselves. This works because the last HOP to the receiver doesn’t require funding. So is possible without an open payment channel.

This is also why 0-HOP/direct messages are possible without open payment channels.

![1-HOP message](/img/node/1-hop-2.png)

This is a manually selected 1-HOP path. If you try and replicate this with Chāo, it should fail as you have no open channels with Chāo.

```
send Chāo,me This is a message for me!
```

Expected output:

```
Failed to send message.
```

### Maximum HOP length

You can use more than one node as an intermediary, with a maximum of three. The HOPR network will only select 3-HOP paths when you use automatic pathing; all longer paths will not be considered and will also fail in manual path selection.

Longer paths require more information to be stored in packet headers, which makes them distinguishable from standard relays. This difference in packet header is a metadata leak that HOPR tries to avoid.

0-HOP, 1-HOP and 2-HOP paths use padded headers to stay consistent with this requirement but are not as mixed or as private as a 3-HOP path. But for the purpose of this walkthrough they are fine.

### Send a 2-HOP message

Now let’s try and send a 2-HOP message. For this to work, every node in the path must have a channel open with the next node in the path, excluding the last channel to the receiver.

So a 2-HOP message to yourself through Betty and Chāo: me → Betty → Chāo → me would require channels to be open from me → Betty & Betty → Chāo (me → Betty → Chāo). The final channel from Chāo → me is not required as the last HOP of a relay is not incentivised. We assume that the reciever has an inherant desire to receive messages.

![2-hop-success](/img/node/2-hop-success-3.png)

You can try and send a 2-HOP message by typing:

```
send Betty,Chāo,me Hi!
```

**_Note:_** make sure these aliases exist for you or replace Betty & Chāo with whatever aliases you are using (or just the HOPR addresses of the nodes you want to use)

If it fails to send, it is likely, that Betty does not have a channel open to Chāo (Betty → Chāo) since you should have a channel open to Betty (me → Betty) with sufficient funds staked. A successful message costs 0.01 HOPR tokens per HOP currently.

![2-hop-fail](/img/node/2-hop-fail-3.png)

### Path directionality

Even if the message succeeds, you should note that you won’t be able to make this 2-HOP message in the other direction as you don’t have an open channel with Chāo. And Chāo may not have an open channel with Betty.

![Reverse route](/img/node/reverse-directionality-3.png)

Here the first route is viable, whereas the second route will fail.

You want to connect to other well-connected nodes to increase your pathing options. But if you just want to experiment with different paths without the hassle, you can use [Playground](https://playground.hoprnet.org/). It will let you control five fully interconnected nodes costlessly without any installations.

**_Note:_** If using Playground, you will need to use the `close` command to remove channels and recreate incomplete paths.

### Path with consecutively repeating nodes

You can not have consecutively repeating nodes. For example, me → Betty → Betty → Zoë.

![Consecutively repeating node](/img/node/consecutively-repeating-3.png)

This is also why the first node specified on a path cannot be yourself, as you are also the sending node.

Try using the following route. It should fail:

```
send me,Betty Hi!
```

### Automatic pathing

So far, we have used manually selected paths by entering the whole path into the command. Instead of this, we can instead let HOPR find a path for the relay by specifying just the receiver **_with no comma:_**

```
send Betty Hi!
```

Automatic pathing will only look for 3-HOP paths from you to the receiver. If none exist or you don’t have sufficient funds staked in the first channel of the relay, it will fail.

```
Failed to send message.
```

**_Note:_** Automatic pathing will discard any repeating nodes even if they are non-consecutive. With manual path selection, you can repeat nodes non-consecutively: me → Betty → Chāo → Betty → me

But this will also throw a warning as it is less than ideal for most relays.

The easiest way to increase your pathing options is to switch your strategy from passive to promiscuous.

### Settings strategy

By default, your strategy is set to passive which means your node will not try to open or close channels automatically.

When you change the strategy to promiscuous, your node will try to open channels with a randomly selected group of nodes you have a healthy connection to. And at the same time, close channels with nodes that are low on balance or considered unhealthy.

You can change your strategy to promiscuous by entering:

```
settings strategy promiscuous
```

You can always switch it back to passive whenever you want:

```
settings strategy passive
```

## Tickets

Although you spend HOPR tokens to relay data, you are actually paid in tickets. Some tickets contain a range of HOPR tokens, but most are useless. The point of this is that over a sizeable amount of tickets, the payment for your services will converge to the amount you would have received.

But with the added benefit of:

- massively reduced on-chain transactions (letting you keep more of the payment)
- And a decoupling of interactions on the HOPR network from on-chain data (increasing privacy)

### Checking your tickets

You can check how many tickets you have earned by typing `tickets`:

```
tickets
```

Expected output:

```
Tickets:
- Pending:          0
- Unredeemed:       0
- Unredeemed Value: 0.00 txHOPR
- Redeemed:         0
- Redeemed Value:   0.00 txHOPR
- Losing Tickets:   0
- Win Proportion:   0%
- Neglected:        0
- Rejected:         0
- Rejected Value:   0.00 txHOPR
```

You should have earned tickets if your node was used as an intermediary to relay data. If you have earned none, try to set your strategy to promiscuous, so you are more likely to be used for automatic pathing.

### Ticket redemption

Tickets are redeemed automatically, so the tickets which contain value will be converted to HOPR tokens and added to the balance of the node used for that relay. The rest are discarded with no trace left on the blockchain.

If a channel exists in both directions between consecutive nodes on the relay, the ticket is redeemed into the following nodes channel instead of its balance.

![tickets-channels](/img/node/tickets-channels-3.png)

In the above example, you, as the sender, will create two tickets of value 0.02 HOPR to pay for the entire relay. Since no channel exists from Betty -> me, the tickets are redeemed into Betty's node. Betty now generates a ticket of value 0.01 HOPR to pay for the remaining relay, and since a channel does exist from Chāo -> Betty, the ticket is redeemed into this channel instead of Chāo's balance.

Chāo then sends the message to Zoë and does not generate a ticket for the last HOP of the relay.

By redeeming tickets into channels, nodes are keeping healthy connections funded. In the long run, this means your node will be more active on the network earning more HOPR!

When channels are closed, all staked tokens are added to your balance and from there can be withdrawn to an external wallet.

### Withdrawing funds

You can withdraw funds from your balance using:

```
withdraw [amount] [NATIVE/HOPR] [address]
```

**Amount –** the number of tokens you want to withdraw

**NATIVE/HOPR –** which token you want to withdraw

**Address –** the wallet address you want to send your tokens to

Example sending native tokens –

```
withdraw 0.01 NATIVE 0xc8Aa5a085c23dfEa903312a73EfC30888bB61f0B
```

This will withdraw 0.01 xDAI from your balance and send it to **0xc8Aa5a085c23dfEa903312a73EfC30888bB61f0B**

Example sending hopr tokens –

```
withdraw 0.01 HOPR 0xc8Aa5a085c23dfEa903312a73EfC30888bB61f0B
```

This will withdraw 0.01 wxHOPR or mHOPR from your balance and send it to **0xc8Aa5a085c23dfEa903312a73EfC30888bB61f0B**

**Please use your own ETH address to withdraw funds and not the example address**

### Importing wallet to MetaMask

If you have [backed up your identity file,](using-hopr-admin#backing-up-your-identity-file) you can convert it to a `.json` file and import it as an account to your MetaMask to access your funds.

**Note:** If you are using macOS or Windows, then you should make sure you can see hidden files. Otherwise, the identity file will not be visible to you.

#### For Avado/Dappnode

(**1**) Locate the `.hopr-identity` file inside the db folder

(**2**) Rename the file to `hopr-identity.json`

#### For local or VPS users

(**1**) find the folder `.hoprd-db-bogota` on your machine or VPS

(**2**) locate the file `.hopr-id-bogota`

(**3**) Rename the file to `hopr-id-bogota.json`

#### Importing the JSON file

(**1**) Open Metamask and click on the accounts icon

(**2**) Go to import accounts

(**3**) Select `JSON file` on the dropdown list

(**4**) Browse through your files and select the renamed `.json` identity file.

(**5**) Click import, and you are all done!

You should now have your funds accessible in your new MetaMask account.
