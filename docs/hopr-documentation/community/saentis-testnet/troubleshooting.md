---
description: Answers to common queries
---

# Troubleshooting

{% hint style="danger" %}
HOPR Säntis has ended. Thank you to everyone who participated. We'll be launching a new testnet soon, codenamed Basòdino. Check back soon for more updates.
{% endhint %}

The Säntis testnet is still an early version of the HOPR network. One major goal of this testnet is to try and find bugs and problems. As a result, things may not work as expected, and the UI and UX are less polished than they will be at mainnet.

Here are some common issues you might run into, along with suggested solutions.

### **I can't connect to the CoverBot**

If you can't connect to the CoverBot, you may need to enable port forwarding on **port 9091** in your router.

To access your router, you'll need to find its IP address.

{% tabs %}
{% tab title="Windows" %}
In Windows, open Command Prompt by typing `cmd` in the search bar. In the terminal, type `ipconfig`. Your router's IP will be listed under `Default Gateway`.
{% endtab %}

{% tab title="MacOS" %}
In the Apple menu visit **System Preferences**, then **Network** then **Advanced Settings**. Select the **TCP/IP** tab and you should see the IP address listed under "Router"
{% endtab %}

{% tab title="Linux" %}
In most Linux distributions, you can click the Network icon, then Connection Information \(or something like it\) and you will see the IP address next to "Default Gateway" or "Default Route"
{% endtab %}
{% endtabs %}

Once you have your router's IP address, enter it into a browser to access your router settings. You will probably need the router admin password, which is usually found on the back.

Router settings vary by model, but you should find port forwarding in the settings menu. Make a new rule with your device's IP address and Port 9091 as both the start and end of the port range.

### **HOPR Chat won't start**

Make sure you're using the latest version of HOPR Chat. If you're getting an error message about funding your node, you'll need to send 0.02 xDAI to the address in the message to proceed.

### **My node went offline, and now I'm not getting points**

If your node is offline for long enough, the bot will remove your address from its database. Just send it the URL with your tweet again and you'll be added back and will start earning points again

### **CoverBot won't recognize my tweet**

CoverBot looks for three things when it parses your tweet:

- a valid HOPR address
- the @hoprnet tag
- the \#HOPRNetwork hashtag

Make sure you have all three of those in your tweet correctly. If you don't know your HOPR address, type `myAddress` at the prompt in HOPR Chat or your hardware node.

Make sure you're sending the full URL, including https:// . Don't send anything else in your message to the bot. Make sure you have includeRecipient turned on.

If you're sure you've done all this correctly, but it's still not working, try making the tweet as short as possible, including nothing except the three pieces of information required.

{% hint style="info" %}
If these solutions don't work, or you have a problem that isn't listed here, please head to our [Discord channel](https://discord.gg/wUSYqpD) where you can get support for a community member or one of the development team.
{% endhint %}
