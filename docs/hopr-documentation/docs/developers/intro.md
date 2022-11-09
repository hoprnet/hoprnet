---
id: intro
title: Developing HOPR apps
---

The HOPR network is only useful if data is being transfered through it. In fact, the more data transported
through HOPR nodes, the more private the whole network becomes. The HOPR network is designed to have constant
traffic, making it impossible for malicious actors to substract meaningful information from it.

HOPR apps are a key component in providing this flow of data. Developers can build applications on top of the HOPR network to
provide users with a private channel for exchanging information. At the same time, HOPR apps create traffic within
the network, increasing the amount of noise mixers use to protect the entire ecosystem.

There are two ways to build applications with the HOPR protocol:

- dApps built on top of the [REST API](/developers/rest-api)
- protocol applications built directly with the TypeScript source code
  ![HOPR protocol stack](/img/developer/architecture.jpg)

### Use cases

Here are just some of the many use cases we believe the HOPR network is a great tool for:

- Browsing information securely from web sites without being tracked by ISPs or third-party providers.
- Sending blockchain transactions without leaking metadata to miners or other relayers.
- Creating p2p applications that require private communication (e.g., gaming, online chats).
- Proxying traffic via a SOCKS-like interface that forwards traffic via the HOPR network.

### Building HOPR apps

The easiest way to build HOPR apps is by launching your own local HOPR cluster, connecting to these nodes via a REST/WebSocket
client, and building on top of the REST API. Use our walkthrough to become familiar with this entire process, and read our
OpenAPI documentation to learn how to interact with HOPR nodes once they are up and running.

#### Walkthrough

The following three-part guide showcases how to get everything ready to build a HOPR app.

- [HOPR Cluster Development Setup](/developers/starting-local-cluster)
- [Interacting with a HOPR node](/developers/connecting-node)
- [HOPR Apps - Hello world](/developers/tutorial-hello-world)

#### OpenAPI Documentation

We use the [OpenAPI standard](https://swagger.io/specification/) to document our REST API. You can see it in our
[REST API](/developers/rest-api) section.

If you are running a hoprd node, you can see the exposed API endpoint of YOUR node at [http://localhost:3001/api/v2/\_swagger/](http://localhost:3001/api/v2/_swagger/)
