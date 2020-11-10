---
description: 'HOPR Games Bounty #3 - Can you get into the party?'
---

# Bouncer Bot

## Requirements

{% tabs %}

Go to [https://github.com/hoprnet/hopr-chat/releases/tag/v1.4.2](https://github.com/hoprnet/hopr-chat/releases/tag/v1.4.2) and download the HOPR Chat file corresponding to your operating system. Follow our Quick start guide if you have any troubles.

{% page-ref page="../../../hopr-chat-tutorial/quickstart.md" %}

Using your DAppStore manager, paste `QmZPsazCHREvYnGbFfDVXDSXh5AtTRrzHUGbi4eN7CejXH` to download and install the HOPR Node DAppNode app.

**Note for AVADO/DAppNode users.**

The existing version of **HOPR Node** available in the store is a bit limited, and does not have yet all the features the normal client has. We are working to sync the **HOPR Node** client with our existing **HOPR Chat** application. In the meantime, to participate with the bounty, you need to manually include your address as the command `includeRecipient` isn't available at the moment in your node. To do so, just add your address into the "message" section, followed by a colon \(`:`\).

![An example on how to send a message from a HOPR Node in an AVADO/DAppNode](../../../.gitbook/assets/image%20%2819%29%20%282%29%20%281%29%20%281%29%20%281%29%20%281%29.png)

The bot responses will show up in your terminal logs.

![](../../../.gitbook/assets/image%20%2820%29%20%282%29%20%281%29%20%281%29%20%281%29%20%281%29.png)

Follow the instructions and do not hesitate to reach us in [Telegram](http://t.me/hoprnet) for questions.

You are ready to go!

### Additional Requirements

- Twitter Account with &gt;25 followers.
- xDAI wallet \(e.g. Metamask w/xDAI RPC, [https://xdai.io/](https://xdai.io/)\)

## Sessions

- 21st of August at 4pm CEST
- 23rd of August at 12pm CEST

## Prices

- 1st Session - 150 xDAI pool, 10 xDAI per participant
- 2nd Session - 150 xDAI pool, 10 xDAI per participant

## Bot Address

1st Session - `16Uiu2HAmPJ4Rh7zBAWPJjXJLuwu4R22zxRMwafQYk4VkRcjhN3L5`

2nd Session - `16Uiu2HAm74fuLiKiWahfK7S8JyL3keSoZXHx9W2FzQGRJtWn15Mb`

## Instructions

### Background

The \#HOPRGames party celebrating the launch of the HOPR Node PC in partnership with [Avado](https://ava.do/checkout/hopr) is the hottest event in town. There’s 2x150 xDAI in giveaways to share between the first 15 people who turn up on each day. Unfortunately, BouncerBot is under strict instructions to only let people on the guest list in. But I’m sure you’ll be able to find a way.

### Steps

Download and install the latest version of HOPR Chat. If this is your first time installing, you might want to look at our quick-start guide.

{% hint style="info" %}
Important! Only HOPR Chat versions v1.4.0 and later will work with this bounty. We recommend downloading the following version - [https://github.com/hoprnet/hopr-chat/relea**ses/tag/v1.4.2**](https://github.com/hoprnet/hopr-chat/releases/tag/v1.4.2)**.**
{% endhint %}

Next, fire up **HOPR Chat**! You should be able to see a prompt to showcase everything is ready for you to type, right after the text “Connecting to bootstrap nodes” show up.

![](../../../.gitbook/assets/image%20%2815%29%20%282%29%20%281%29%20%281%29%20%281%29%20%281%29.png)

**Time to identify yourself**

Before entering the party, you need to identify yourself. To do so, run the command `includeRecipient` which will embed your address before all messages you send.

{% hint style="warning" %}
Make sure you run `includeRecipient` before proceeding! Otherwise, BouncerBot will not be able to reply to you.
{% endhint %}

You can always see whether you are including your address in **HOPR Chat** by typing `settings`.

![](../../../.gitbook/assets/image%20%2816%29%20%282%29%20%281%29%20%281%29%20%281%29%20%281%29.png)

**Name the bot**

In version `1.4.x` we have introduced a new command, called `alias`. `alias` helps you "save" the address of a user to use it later, and means you don't have to manually paste a HOPR address each time. Since you probably want to save the address of BouncerBot, you can run the following to save it for later:

```text
alias 16Uiu2HAmPJ4Rh7zBAWPJjXJLuwu4R22zxRMwafQYk4VkRcjhN3L5 bouncerbot
```

Now, you can send messages to `bouncerbot` just by typing `send bouncerbot`.

**Try to get into the party**

Time to get into the party! Message `bouncerbot` to try and enter the party. It might take multiple tries before you can get in. Good luck!

![](https://media.giphy.com/media/xT5LMWSMSsE5lji9LW/giphy.gif)
