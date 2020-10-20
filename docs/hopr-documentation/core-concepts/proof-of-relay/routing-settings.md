---
description: Routing options in HOPR
---

# Routing Settings

Messages sent through the HOPR network will make a user-specified number of hops via other HOPR relay nodes before reaching their destination.

There are currently two choices for routing:

* **Direct**: Messages take 0 hops, so will be sent direct to the recipient node. This won't cost HOPR tokens, but the data won't undergo any mixing. Therefore this is not recommended for anything other than testing purposes.
* **Manual:** When you send a message, you will specify the route by providing a node ID for each hop. Only nodes with open payment channels to the next downstream node are eligible choices.

{% hint style="info" %}
In the future, there will be an automatic routing option, where your node will automatically select the best route between relay nodes with open and funded payment channels.
{% endhint %}

To change routing modes type:

{% tabs %}
{% tab title="Direct" %}
```text
settings routing direct
```
{% endtab %}

{% tab title="Manual" %}
```text
settings routing manual
```
{% endtab %}
{% endtabs %}

You can check your routing settings at any time by typing `settings`

