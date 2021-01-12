---
description: Change your routing settings to start using HOPR's privacy features
---

# Changing Your Routing Settings

Messages sent through the HOPR network will make a user-specified number of hops via other HOPR relay nodes before reaching their destination.

There are currently two choices for routing:

- **Direct**: Messages take 0 hops, so will be sent direct to the recipient node. This won't cost HOPR tokens, but the data won't undergo any mixing. Therefore this is not recommended for anything other than testing purposes.
- **Manual:** When you send a message, you will specify the route by providing a node ID for each hop. Only nodes with open payment channels to the next downstream node are eligible choices.

{% hint style="info" %}
In the future, there will be an automatic routing option, where your node will automatically select the best route between relay nodes with open and funded payment channels.
{% endhint %}

In the testnet, routing is set to direct by default. So to start testing HOPR's multihop features, we need to turn that off.

## Change Routing to Manual

Type:

```text
settings routing manual
```

You'll see the following notification:

![Changing your routing settings](.gitbook/assets/avado-manual-routing%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29%20%281%29.png)

You can check your routing settings at any time by typing `settings`.

{% hint style="info" %}
To revert to zero-hop messages, type `settings routing direct`
{% endhint %}
