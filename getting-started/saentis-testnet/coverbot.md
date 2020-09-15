---
description: 'More information about CoverBot, one of the bots overseeing the Säntis testnet'
---

# CoverBot

{% hint style="info" %}
CoverBot is the latest in a series of bots running on the HOPR network which will be familiar from participants in our regular bounties and gaming sessions. We'll be adding more bots to Säntis as the testnet progresses, including more ways to earn points.
{% endhint %}

### Step 3: Send a Tweet

Next, you need to send a tweet which will be used to verify your participation in the network. This is just a basic check to prevent people from entering multiple times.  
  
Your tweet needs to include your HOPR address, tag the @hoprnet account, and include the hashtag \#HOPRNetwork. An example is show below.   
  
The bot should be able to parse any tweet which includes these three things, so feel free to add comments and even emojis. But if you're having difficulty registering, try again with a simpler tweet.

{% hint style="info" %}
**Don't delete this tweet!** The CoverBot will continuously check that the tweet associated with your address still exists. If the check fails, your address will be removed from the bot's database and you won't be able to earn any more points.
{% endhint %}

### Step 4: Turn On includeRecipient

The HOPR network is fully anonymous by default. That means no-one can see who you're sending messages to, not even the recipient.  
  
Obviously, in most use cases we want people who we contact \(but not anyone else!\) to know who is sending them data, so they know who to send data back to and where to send it.

Type `includeRecipient`and then type `y` to confirm. From now on, every message you send will also be sent with your address. Now when you message the CoverBot, it will know your address and will be able to reply and add you to its database.

### Step 5: Register With The Bot

Now that you've sent your tweet, and turned on `includeRecipient`, you need to register with the bot. Copy the full URL of your tweet and send it as a message in HOPR Chat to the bot.  
  
Type `send 16Uiu2HAmAdZNE1VqtbPt4uk9rTypbifLG9eesb8PVb5NjAywGQ4j` then press Enter. Now paste the URL of your tweet and press Enter again. The bot will now check your tweet and will send you a verification message if you're successful.

![](../../.gitbook/assets/verification-1.png)

{% hint style="info" %}
If this is the first time you've registered this node / tweet in the testnet, you'll receive 10 points!
{% endhint %}

### Step 6: Stay Online to Earn Points!

Now that your address is whitelisted with the bot, you can score points by relaying cover traffic and receiving xHOPR tokens.  
  
Every 60 seconds, the bot will randomly select an address from the whitelist and ping it to check that it's online. If is, it will send cover traffic to itself via that node.

If the CoverBot selects your address, you'll receive an xHOPR token and a point will be added to your score. You'll also receive a verification message.

![](../../.gitbook/assets/verification-2.png)

{% hint style="info" %}
The more your node is online, the greater your chance of scoring points, so try to maximize your node uptime. But if your node does go offline, you can re-register with the bot by sending it your tweet again.
{% endhint %}





  


