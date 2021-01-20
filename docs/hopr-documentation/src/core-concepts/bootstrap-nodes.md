<!-- ---
description: An introduction to bootstrap nodes inside HOPRd
--- -->

# Bootstrap Nodes

**HOPRd** currently relies on **Bootstrap Nodes** to work. These are nodes created with the `bootstrap` setting enabled, functioning solely as relayers for other nodes in the network.

**Bootstrap Nodes** are only meant to serve as an initial relayer between nodes. Once communication has been established between two or more **HOPRd** nodes, it is no longer necessary to communicate with a **Bootstrap Node**.

As an analogy, think of **Bootstrap Nodes** as the hosts at a party. They introduce guests to each other, and those guests can then talk directly.

To check the bootstrap node you're currently connected to, type `info`.
