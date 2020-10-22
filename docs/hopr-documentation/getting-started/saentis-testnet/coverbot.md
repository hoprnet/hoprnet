---
description: 'More information about CoverBot, one of the bots overseeing the Säntis testnet'
---

# Registering with CoverBot

{% hint style="danger" %}
HOPR Säntis has ended. Thank you to everyone who participated. We'll be launching a new testnet soon, codenamed Basòdino. Check back soon for more updates.
{% endhint %}

To participate in the testnet, you'll need to register with CoverBot and open a payment channel. This will cost a very small amount of xDAI. Make sure you've completed the previous setup sections and have funded your node with 0.02 xDAI \(around \$0.02\).

{% hint style="info" %}
CoverBot is the latest in a series of bots running on the HOPR network which will be familiar from participants in our regular bounties and gaming sessions. We'll be adding more bots to Säntis as the testnet progresses, including more ways to earn points.
{% endhint %}

## Step 1: Find Your HOPR Address

With your node up and running, you are now ready to type and communicate with other users on the **HOPR Network**, including the bot which oversees the testnet.

Communication is achieved using **HOPR addresses.** To find your node's address, type `myAddress` and press `Enter` inside **HOPR Chat** or your **HOPR Node PC** app**.** Something like the following will show up:

```text
> myAddress
ethereum:  0x9e95cdcb480f133b0c1af70613d1488ee01bf53e
HOPR:      16Uiu2HAm34oK6EyA2SuFo9rHXpm5Kwy6C8MeJ26JaRFBzgdQqVpZ
```

{% hint style="info" %}
There are two addresses associated with your node: the Ethereum address is used to fund your account and pay for gas fees on the blockchain, the HOPR address is where you'll send messages and receive HOPR tokens.

For the purposes of the testnet, Ethereum is actually xDAI and HOPR is actually xHOPR.
{% endhint %}

## Step 2: Send a Tweet

First, you need to send a tweet which will be used to verify your participation in the network. The bot will be able to read this tweet and check your eligibility. This is just a basic check to prevent people from entering multiple times.

{% hint style="warning" %}
Your Twitter handle will be your name on the Säntis scoreboard.
{% endhint %}

Your tweet needs to include your HOPR address, tag the @hoprnet account, and include the hashtag \#HOPRNetwork. An example is show below.

The bot should be able to parse any tweet which includes these three things, so feel free to add comments and even emojis. But if you're having difficulty registering, try again with a simpler tweet. Click [here](https://twitter.com/intent/tweet?original_referer=https%3A%2F%2Fsaentis.hoprnet.org%2F&ref_src=twsrc%5Etfw&related=hoprnet&text=Signing%20up%20to%20earn%20%24HOPR%20on%20the%20%23HOPRnetwork.%20My%20%40hoprnet%20address%20is%3A%20&tw_p=tweetbutton) to get a template.

{% hint style="danger" %}
**Don't delete this tweet!** The CoverBot will continuously check that the tweet associated with your address still exists. If the check fails, your address will be removed from the bot's database and you won't be able to earn any more points.
{% endhint %}

## Step 3: Turn On includeRecipient

The HOPR network is fully anonymous by default. That means no-one can see who you're sending messages to, not even the recipient.

Obviously, in most use cases we want people who we contact \(but not anyone else!\) to know who is sending them data, so they know who to send data back to and where to send it.

Type `includeRecipient`and then type `y` to confirm. From now on, every message you send will also be sent with your address. Now when you message the CoverBot, it will know your address and will be able to reply and add you to its database.

![](../../.gitbook/assets/include-recipient.png)

## Step 4: Register With The Bot

Now that you've sent your tweet, and turned on `includeRecipient`, you need to register with the bot. Copy the full URL of your tweet and send it as a message in HOPR Chat to the bot.

Type `send 16Uiu2HAmRE4fVtp8dF6H62NzRcx6LGUTL5fBRTdnAfZXjveP5Kz9` then press Enter. Now paste the URL of your tweet and press Enter again. The bot will now check your tweet and will send you a verification message if you're successful.

![](../../.gitbook/assets/verification-1.png)

{% hint style="info" %}
If this is the first time you've registered this node / tweet in the testnet, you'll receive 100 points!
{% endhint %}

## Step 5: Stay Online to Earn Points!

Now that your address is whitelisted with the bot, you can score points by relaying cover traffic and receiving xHOPR tokens.

Every 30 seconds, the bot will randomly select an address from the whitelist and ping it to check that it's online. If is, it will send cover traffic to itself via that node.

If the CoverBot selects your address, you'll receive an xHOPR token and 10 points will be added to your score. You'll also receive a verification message.

![](../../.gitbook/assets/verification-2.png)

{% hint style="info" %}
The more your node is online, the greater your chance of scoring points, so try to maximize your node uptime.

More ways to earn points will be introduced as the testnet proceeds.
{% endhint %}
