---
description: 'Checking for other online nodes, and finding your HOPR node address.'
---

# Exploring The Network

Now that your node is up and running, it's time to explore the network.

Type `crawl` and you will see a message detailing how many other nodes are visible to your node. For example:

```text
> crawl
Crawled network, connected to 8 peers
```

{% hint style="info" %}
If your node has just started, it may not have built a full picture of how many nodes are available to communicate with. If the number of peers seems low, try crawling again to find more.
{% endhint %}

Next, type `listConnectedPeers` to see the addresses of the other nodes which are currently online and visible to your node. For example:

```text
> listConnectedPeers
Connected to:
 - 16Uiu2HAmV5PD9zKrNyY6cvCo7WWKXCLze7gGg3sz5XFfEooyaS9x
 - 16Uiu2HAmPLuTQ3KqQM2wGSyPm7gwXNeaR9R92mAEFCgL9r4e8uTx
 - 16Uiu2HAmHBYFwcBnv51WNLpd91VjTkysQXPVEAv9q9GVNDJN3Wci
 - 16Uiu2HAmNtoQri1X4ikUzCqjFQptRSLSVKnVzMmtZiCHCHkdWJr7
```

{% hint style="info" %}
The list of connected peers will sometimes show fewer nodes than the number returned by `crawl.` This is because nodes which go offline are not immediately removed from your node's list of possible connections.
{% endhint %}

## Finding Your Address

To find your own address, type `address`. This will also show the ETH wallet address you used to fund your account. For example:

```text
> address
HOPR Address:  16Uiu2HAmEvFnLE6cjcAaXCFv4K2zP5Sb8HHXSAEQbBVzVJyCczg5
Matic Address:  0xabe565d06c810dac59a02560ea4ea83cda2e995f
```

Now that we've scoped out the network, it's time to start sending some messages. The next section will explain how.
