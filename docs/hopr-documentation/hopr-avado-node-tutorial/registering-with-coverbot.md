---
description: Register with the CoverBot to start participating in the testnet.
---

# Registering With CoverBot

Mixnets need cover traffic to ensure that users retain their privacy even when organic network usage is low. HOPR will employ federated and then fully decentralized cover traffic as development progresses, but for these early testnets cover traffic is simulated by a bot, CoverBot.

To earn points in the testnet, you'll need to register with CoverBot, who will then try to relay data via a random registered user every thirty seconds. Successfully relaying data back to CoverBot will earn you a ticket, which can be redeemed for HOPR.

{% hint style="info" %}
CoverBot is the latest in a series of bots running on the HOPR network which will be familiar from participants in our regular bounties and gaming sessions. We'll be adding more bots as our testnets progress, including more ways to earn points.
{% endhint %}

## Step 1: Check Your HOPR Address and Your Settings

Before you register with CoverBot, you'll need the following things:

- Your HOPR address
- `includeRecipient` turned on
- A Twitter account

The first two items were covered in the previous sections in this tutorial, [**Exploring the Network**](../hopr-chat-tutorial/exploring-the-network.md#finding-your-address) and [**Messaging RandoBot**](../hopr-chat-tutorial/randobot.md#step-2-turn-on-includerecipient), respectively.

## Step 2: Send a Tweet

First, you need to send a tweet which will be used to verify your participation in the network. CoverBot will be able to read this tweet and check your eligibility. This is just a basic check to prevent people from entering multiple times.

{% hint style="warning" %}
Your Twitter handle will be your name on the testnet scoreboard.
{% endhint %}

Your tweet needs to include:

- your HOPR address
- the @hoprnet handle
- the \#Basodino hashtag

An example is shown below:

![](../.gitbook/assets/example-tweet%20%281%29%20%281%29%20%281%29.png)

The bot should be able to parse any tweet which includes these three things, so feel free to add comments and even emojis. But if you're having difficulty registering, try again with a simpler tweet.

{% hint style="danger" %}
**Don't delete this tweet!** The CoverBot will continuously check that the tweet associated with your address still exists. If the check fails, your address will be removed from the bot's database.
{% endhint %}

## Step 3: Register With The Bot

Now that you've sent your tweet, you need to register with the bot. Copy the full URL of your tweet and send it as a message in HOPR Chat to the bot. Type:

```text
send 16Uiu2HAm79TuiHcEtjcELXAwcjEX6Sh7qGDbiqZbDvHSicRyPm9R [URL of your tweet]
```

Then press Enter.

![](../.gitbook/assets/coverbot-avado-success%20%281%29%20%281%29%20%281%29.png)

The bot will now check your tweet and will send you a verification message if you're successful. If there's an error, the bot will try and explain what has gone wrong so you can fix it.

## Step 4: Stay Online to Earn Tickets!

Now that your address is whitelisted with the bot, you can earn tickets by relaying data.

Every 30 seconds, the bot will randomly select an address from the whitelist and ping it to check that it's online. If is, it will send cover traffic to itself via that node.

![](../.gitbook/assets/coverbot-avado-relaying%20%281%29%20%281%29%20%281%29.png)

If the CoverBot selects your address, you'll receive a ticket which can be redeemed for HOPR. You'll also receive a verification message.

{% hint style="info" %}
The more your node is online, the greater your chance of earning tickets, so try to maximize your node uptime.
{% endhint %}

## Check Your Score

You can see the current leaderboard at [**https://network.hoprnet.org**](https://network.hoprnet.org)\*\*\*\*
