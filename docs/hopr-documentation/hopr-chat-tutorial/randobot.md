---
description: >-
  More information about using RandoBot to verify your network connection, test
  out the messaging flow, and act as a relay node.
---

# Messaging RandoBot

Before moving on to more complicated tasks, we highly recommend talking to Randobot, our connection verification bot that responds to messages with a randomized greeting. This is a good way to test the basic features of HOPR, and if we need to troubleshoot it will help you and the HOPR team verify that your network is properly configured.

{% hint style="info" %}
RandoBot is one in a series of bots running on the HOPR network which will be familiar from participants in our regular bounties and gaming sessions.
{% endhint %}

## Step 1: Ping Randobot

First, let's make sure your node can reach RandoBot. Type:

```text
ping 16Uiu2HAmCK23BB3RW82MkTBjUjNCKya9Xcb5S85AXPvzjfk2gXNm
```

If everything if working correctly, you should receive a pong back from the bot. For example:

```text
​​​> ping 16Uiu2HAmCK23BB3RW82MkTBjUjNCKya9Xcb5S85AXPvzjfk2gXNm
Pong received in: 144 ms
```

If you can't ping the bot, please check the Troubleshooting guide or ask for help in Telegram or Discord.

## Step 2: Turn On includeRecipient

The HOPR network is fully anonymous by default. That means no-one can see who you're sending messages to, not even the recipient.

Obviously, in most use cases we want people who we contact \(but not anyone else!\) to know who is sending them data, so they know who to send data back to and where to send it.

You can manually prepend your address to messages you send, but for convenience you can also instruct HOPR Chat to do this automatically. Type:

```text
settings includeRecipient true
```

You will receive a notification confirming your settings change:

```text
> settings includeRecipient true
You have set your “includeRecipient” settings to “true”.
```

From now on, every message you send will also be sent with your address. Now when you message the RandoBot, it will know your address and will be able to reply.

To turn this off, type:

```text
settings includeRecipient false
```

Now the recipient of your messages won't be able to see your address. This has some isolated uses, but in most cases you'll want to have this set to `true`. In particular, the HOPR bots won't be able to respond if you don't turn on `includeRecipient`.

{% hint style="info" %}
You can always see whether you have turned on `includeRecipient` by running the `settings` command.
{% endhint %}

## Step 3: Set an Alias

Since HOPR Addresses can be hard to remember, we created the `alias` command, which allows you to save a HOPR Address in memory for the duration of your HOPR Chat session. Within HOPR Chat, you can simply run `alias` to learn about its usage.

```text
> alias
usage: <PeerId> <Name>
```

You will want to `alias` RandoBot's address:

```text
16Uiu2HAmCK23BB3RW82MkTBjUjNCKya9Xcb5S85AXPvzjfk2gXNm
```

{% hint style="warning" %}
Bear in mind the address of RandoBot might change over time. If you are unable to `ping` or `send` a message to it, make sure to come back to our documentation to verify its address has remained the same.
{% endhint %}

Within HOPR Chat, then run the following:

```text
alias 16Uiu2HAmCK23BB3RW82MkTBjUjNCKya9Xcb5S85AXPvzjfk2gXNm randobot
```

You'll receive a notification confirming the alias:

```text
> alias 16Uiu2HAmCK23BB3RW82MkTBjUjNCKya9Xcb5S85AXPvzjfk2gXNm randobot
Set alias 'randobot' to '16Uiu2HAmCK23BB3RW82MkTBjUjNCKya9Xcb5S85AXPvzjfk2gXNm'.
```

{% hint style="warning" %}
Aliases are just a temporary feature of your chat session. Aliases are not visible to other users, and they will reset when you shut down your node.
{% endhint %}

## Step 4: Say Hi to RandoBot

Now that you have aliased RandoBot, you are ready to say hi! Using the `send` command, write `send randobot`, press `Enter`, and simply write `hi`, and press `Enter` again.

```text
> send randobot
Type your message and press ENTER to send:
hi
```

If successful, RandoBot will pick a series of random words and say those to you. They are random, so don't get offended!

```text
---------- New Packet ----------
Destination    : 16Uiu2HAmCK23BB3RW82MkTBjUjNCKya9Xcb5S85AXPvzjfk2gXNm
--------------------------------
>
===== New message ======
Message: 16Uiu2HAmCK23BB3RW82MkTBjUjNCKya9Xcb5S85AXPvzjfk2gXNm slimy olden dove
Latency: 63 ms
========================
```

Congratulations! You have successfully sent a message through the HOPR Network!

{% hint style="info" %}
If you get a `Timeout Error,` please run `crawl` a few times. At the beginning of your HOPR Chat session, your node might have yet to discover RandoBot, so try again later or report your error to our [Discord](https://discord.gg/5FWSfq7) channel.
{% endhint %}
