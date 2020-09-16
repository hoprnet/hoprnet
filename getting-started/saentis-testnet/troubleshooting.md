---
description: Answers to common queries
---

# Troubleshooting

The Säntis testnet is still an early version of the HOPR network. One major goal of this testnet is to try and find bugs and problems. As a result, things may not work as expected, and the UI and UX are less polished than they will be at mainnet.  
  
Here are some common issues you might run into, along with suggested solutions.

### **HOPR Chat won't start**

Make sure you're using the latest version of HOPR Chat. If you're getting an error message about funding your node, you'll need to send 0.02 xDAI to the address in the message to proceed.

### **My node went offline, and now I'm not getting points**

If your node is offline for long enough, the bot will remove your address from its database. Just send it the URL with your tweet again and you'll be added back and will start earning points again

### **I can't connect to the CoverBot**

If you can't connect to the CoverBot, you may need to enable port forwarding on **port 9091** in your router.

### **CoverBot won't recognize my tweet**

CoverBot looks for three things when it parses your tweet:

* a valid HOPR address
* the @hoprnet tag
* the \#HOPRNetwork hashtag

Make sure you have all three of those in your tweet correctly. If you don't know your HOPR address, type `myAddress` at the prompt in HOPR Chat or your hardware node.  
  
Make sure you're sending the full URL, including https:// . Don't send anything else in your message to the bot. Make sure you have includeRecipient turned on.

If you're sure you've done all this correctly, but it's still not working, try making the tweet as short as possible, including nothing except the three pieces of information required.

{% hint style="info" %}
If these solutions don't work, or you have a problem that isn't listed here, please head to our [Discord channel](https://discord.gg/wUSYqpD) where you can get support for a community member or one of the development team.
{% endhint %}



