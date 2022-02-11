---
id: intro
title: Developing HOPR apps
---

The HOPR network is only as useful as the data that is being transfered through it. The more data is transported
through HOPR nodes, the more private it is. This is due to the architecture of the HOPR network to have constant
traffic, making it harder for malicious actors to substract meaningful information from it.

HOPR apps are a key component in that equation. Developers can build applications on top of the HOPR network to
provide users with a private channel for exchanging information. At the same time, HOPR apps create traffic within
the network, increasing the amount of noise mixers use to protect the entire ecosystem.

### Use cases

Here are some of the use cases we believe the HOPR network is a great tool for:

- Browsing information securely from web sites without being tracked by ISP or third-party providers.
- Sending blockchain transactions with leaking metadata to miners or other relayers.
- Creating p2p applications that require private communication (e.g. gaming, online chats).
- Proxying traffic via a SOCKS-like interface that forwards traffic via the HOPR network.

### Building HOPR apps

The easiest way to build HOPR apps is by launching your own local HOPR cluster, connecting to them via a REST/WebSocket
client, and building on top of the REST API. Use our Walkthrough to get familiar with this entire process, and read our
OpenAPI documentation to learn how to interact with HOPR nodes once they are up and running.

#### Walkthrough

The following three-part guide showcases how to get everything ready for building a HOPR app.

- ["Running a local HOPR Cluster"](/developers/starting-local-cluster)
- ["Interacting with a HOPR node"](/developers/connecting-node)
- ["HOPR Apps - Hello world"](/developers/tutorial-hello-world)

#### OpenAPI Documentation

We use the [OpenAPI standard](https://swagger.io/specification/) to document our REST API. You can it in our
["REST API"](/developers/rest-api) section.

If you are running a hoprd node, you can see the actually exposed API endpoint of YOUR node at [http://localhost:3001/api/v2/_swagger/](http://localhost:3001/api/v2/_swagger/)
