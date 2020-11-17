---
description: An introduction to bootstrap nodes inside HOPR Chat
---

# Bootstrap Nodes

**HOPR Chat** currently relies on **Bootstrap Nodes** to work. These are nodes created with the `bootstrap` setting enabled, functioning solely as relayers for other nodes in the network.

**Bootstrap Nodes** are only meant to serve as an initial relayer between nodes. Once communication has been established between two or more **HOPR Chat** nodes, it is no longer necessary to communicate with a **Bootstrap Node**.

As an analogy, think of **Bootstrap Nodes** as the hosts at a party. They introduce guests to each other, and those guests can then talk directly.

To run **HOPR Chat** as a bootstrap node, pass a `-b` flag to the run command.

## Available Bootstrap Nodes

Feel free to use any \(or all\) of the following URLs as your `BOOTSTRAP_SERVERS` parameter in your **HOPR Chat** Docker image. Each of our **Bootstrap Nodes** are located in different countries and serve a specific environment. You can find our Bootstrap Nodes by querying our `TXT` records from our `hoprnet.org` `_dnsaddr.bootstrap` subdomain. Quickly see those in [https://www.whatsmydns.net/\#TXT/\_dnsaddr.bootstrap.testnet.hoprnet.org](https://www.whatsmydns.net/#TXT/_dnsaddr.bootstrap.testnet.hoprnet.org).
