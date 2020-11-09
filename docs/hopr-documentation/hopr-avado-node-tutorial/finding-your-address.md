---
description: 'Checking for other online nodes, and finding your HOPR node address.'
---

# Exploring The Network

Now that your AVADO node is up and running, it's time to explore the network.

Type `crawl` and you will see a message detailing how many other nodes are visible to your node.

![Crawling the HOPR network](../.gitbook/assets/avado-crawl%20%281%29%20%281%29%20%281%29.png)

{% hint style="info" %}
If your node has just started, it may not have built a full picture of how many nodes are available to communicate with. If the number of peers seems low, try crawling again to find more.
{% endhint %}

Next, type `listConnectedPeers` to see the addresses of the other nodes which are currently online and visible to your node.

![List of connected peers on a HOPR AVADO node](../.gitbook/assets/list-connected-peers-avado%20%281%29%20%281%29.png)

{% hint style="info" %}
The list of connected peers will sometimes show fewer nodes than the number returned by `crawl.` This is because nodes which go offline are not immediately removed from your node's list of possible connections.
{% endhint %}

Addresses are shown in their shortened form, showing only the last five characters. Click on any shortened address to expand it to show the entire address string.

![The expanded addresses from the example above](../.gitbook/assets/avado-expanded-addresses%20%281%29%20%281%29.png)

## Finding Your Own Address

To find your own address, type `myAddress`. Once again, you'll need to click the short-form address to expand it. The `myAddress` command will also show the wallet address you used to fund your account.

![](../.gitbook/assets/avado-myaddress%20%281%29%20%281%29.png)

Now that we've scoped out the network, it's time to start sending some messages. The next section will explain how.

