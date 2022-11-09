---
id: gitpod-setup
title: Setting Up Gitpod
---

<!-- Using this page to hold removed GitPod content --> 

## Gitpod Setup

The simplest and fastest way to set up a HOPR cluster is using [Gitpod](https://gitpod.io). Gitpod is a cloud tool used to create
automated developer environments in seconds. We have configured our [HOPR monorepo](https://gitpod.io/#https://github.com/hoprnet/hoprnet)
to quickly set up everything for you to get started.

<div class="embed-container">
<iframe src="https://player.vimeo.com/video/678070260?h=9ef64ca41b" frameborder="0" allow="autoplay; fullscreen; picture-in-picture" allowfullscreen></iframe>
</div>

[![Open in Gitpod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#https://github.com/hoprnet/hoprnet/tree/release/lisbon)

After Gitpod creates a virtual machine with our codebase, it will immediately start running a local cluster as described by our
[Gitpod configuration file](https://github.com/hoprnet/hoprnet/blob/release/lisbon/.gitpod.yml). The entire setup will take roughly 5-10
minutes, after which it will `export` a series of endpoint URLs which you can use later.

```bash
gitpod /workspace/hoprnet (release/lisbon) $ echo $HOPR_NODE_1_HTTP_URL
https://13301-hoprnet-hoprnet-npnjfo3928b.ws-us31.gitpod.io
gitpod /workspace/hoprnet (release/lisbon) $ echo $HOPR_NODE_1_WS_URL
https://19501-hoprnet-hoprnet-npnjfo3928b.ws-us31.gitpod.io
gitpod /workspace/hoprnet (release/lisbon) $ echo $HOPR_NODE_1_ADDR
16Uiu2HAmE9b3TSHeF25uJS1Ecf2Js3TutnaSnipdV9otEpxbRN8Q
```

### Gitpod URLs

When running a HOPR cluster inside Gitpod, all the URLs will be exposed via their own DNS service, which resolves services to ports via
URLs that look something like this: `https://13302-hoprnet-mynechat-7x6h2ghc17f.ws-us30.gitpod.io`. These URLs change every so often, and are behind
SSL certificates within Gitpod, making them susceptible to `Mixed-content` and `CORS` errors when working locally.

To avoid these issues, we recommend installing the [Gitpod Companion App](https://www.gitpod.io/docs/develop/local-companion), which will forward Gitpod's services to your workstation. You can use them via `127.0.0.1` instead of the Gitpod URLs. All our documentation
assumes this local IP, so using the app will make things easier for you as you read on.

### Replacing URLs

If you do not want to use the Gitpod Companion App, just remember to replace the URLs in the documentation with your Gitpod service URL. You
can obtain the specific URL for each port by running the `gp` tool. For example, to learn the URL behind port `13301` run the following:

```bash
gp url 13301
```

which will return something like `https://13301-hoprnet-mynechat-7x6h2ghc17f.ws-us30.gitpod.io`. Please be aware that you might need to change the protocol from `https` to `wss`, depending on whether the documentation refers to your `HTTP_URL` or `WS_URL`, .












**WebSocket clients**

- [Piesocket WebSocket Tester](https://www.piesocket.com/websocket-tester): This is a great tool to debug both listening to and sending
  messages from/to your HOPR node. Make sure to paste your `HOPR_NODE_1_WS_URL` and append your `apiToken` as a query parameter. Also,
  you'll need to change the `http` protocol to `ws`. For instance, here's how this would look in a `Gitpod.io` instance. After it's connected, you can type `balance` to see your node response.

```bash
ws://127.0.0.1:19501/?apiToken=^^LOCAL-testing-123^^
```

If you are using a Gitpod public URL, you can simply use the output of `gp url` for that particular port (`19501`) and paste it in the website.

```bash
gp url 19501
```

The output should look something like this: `wss://19501-hoprnet-mynechat-7x6h2ghc17f.ws-us30.gitpod.io/?apiToken=^^LOCAL-testing-123^^`.




## Walkthrough

If you need help completing this tutorial, you can watch our 15-minute walkthrough, which also includes the setup of the local HOPR cluster.

**Note:** The tutorial uses a GitPod setup which is currently depreciated. You should follow the tutorial through your terminal ignoring the setup process.

<figure class="video-container" style={{"marginTop": "-100px", "marginBottom": "-100px"}}>
  <iframe src="https://player.vimeo.com/video/672847960?h=bc02050298" width="640" height="564" frameborder="0" allow="autoplay; fullscreen" allowfullscreen></iframe>
</figure>


















