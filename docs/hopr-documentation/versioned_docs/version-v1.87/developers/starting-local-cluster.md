---
id: starting-local-cluster
title: HOPR Cluster Development Setup
---

The best way to learn and develop the HOPR protocol is by interacting with a HOPR node connected inside the HOPR network.
A series of HOPR nodes fully interconnected between each other is called a HOPR cluster.

Without a HOPR cluster, app developers can only fake interactions with a HOPR node by mocking their API[^1], and
core protocol developers can't replicate critical functionality over messaging such as ticket redemption
and packet relaying.

## Requirements

:::info INFO

MacOS M1 users will need to follow an extra set of instructions from [NVM](https://github.com/nvm-sh/nvm#macos-troubleshooting) to allow them to use Node.js 16.

Node.js should be compiled under x64 architecute.

:::

To run a HOPR cluster, we suggest the following options to you based on your Operating System (OS):

- Windows: Gitpod
- Linux or macOS: Gitpod or Local

## Gitpod Setup

The simplest and fastest way to setup a HOPR cluster is using [Gitpod](https://gitpod.io). Gitpod is a cloud tool used to create
automated dev environments in seconds. We have configured our [HOPR monorepo](https://gitpod.io/#https://github.com/hoprnet/hoprnet)
to quickly setup everything for you to get started.

[![Open in Gitpod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#https://github.com/hoprnet/hoprnet)

After Gitpod creates a virtual machine with our codebase, it will immediately start running a local cluster as described by our
[Gitpod configuration file](https://github.com/hoprnet/hoprnet/blob/master/.gitpod.yml). The entire setup will take roughly 5-10
minutes, after which it will `export` a series of endpoint URLs which you can use later.

```bash
gitpod /workspace/hoprnet (master) $ echo $HOPR_NODE_1_HTTP_URL
https://13301-hoprnet-hoprnet-npnjfo3928b.ws-us31.gitpod.io
gitpod /workspace/hoprnet (master) $ echo $HOPR_NODE_1_WS_URL
https://19501-hoprnet-hoprnet-npnjfo3928b.ws-us31.gitpod.io
gitpod /workspace/hoprnet (master) $ echo $HOPR_NODE_1_ADDR
16Uiu2HAmE9b3TSHeF25uJS1Ecf2Js3TutnaSnipdV9otEpxbRN8Q
```

### Gitpod URLs

When running a HOPR cluster inside Gitpod, all the URLs will be exposed via their own DNS service, which resolves services to ports via
URLs that look like this `https://13302-hoprnet-mynechat-7x6h2ghc17f.ws-us30.gitpod.io`. These URLs change every so often, and are behind
SSL certificates within Gitpod, making them susceptible to `Mixed-content` and `CORS` erros when working locally.

To avoid these issues, we recommend installing the [Gitpod Companion App](https://www.gitpod.io/docs/develop/local-companion), which
will forward Gitpod's services to your workstation, so you can use them via `127.0.0.1` instead of the Gitpod URLs. As all our documentation
assume this local IP, so using the app will make things easier for you to read on.

### Replacing URLs

If you do not want to use the Gitpod Companion App, just remember to replace the URLs in the documentation to your Gitpod service URL. You
can obtain the specific URL per port running the tool `gp`. For knowing the URL behind port `13301` you run the following:

```bash
gp url 13301
```

which will return something like `https://13301-hoprnet-mynechat-7x6h2ghc17f.ws-us30.gitpod.io`. Please be aware that depending on whether
the documentation refers to your `HTTP_URL` or `WS_URL`, you might need to change the protocol from `https` to `wss`.

## Local Setup

The local setup will give you a similar setup to the one the HOPR team works with on a daily basis. After all dependencies are installed,
this configuration allows you to develop HOPR apps offline.

1. **Download the latest version of the repository**: Download a local version of our [GitHub repository monorepo](https://github.com/hoprnet/hoprnet)[^2]
   and unzip it in your local folder (roughly `~30 Mbs` at the time of writing). For the next tasks, we will assume you are within that folder.

```bash
wget https://github.com/hoprnet/hoprnet/archive/refs/heads/master.zip
unzip master.zip
cd hoprnet-master
```

2. **Install the dependencies of the project and build it**: Make sure you have `nodejs@16` (we suggest installing it via [nvm](https://github.com/nvm-sh/nvm), ie `nvm install lts/gallium`), and `yarn` (included in `nodejs@16` by running `corepack enable`)
   to install and build the required packages and project modules. Ideally, you also have setup your computer with basic development toolset[^3]. Please bear in mind that this process will take at least 5-10 minutes depending on your computer.

```bash
yarn && yarn build
```

3. **Run the one-line setup script**: Proceed to run the following script. If you are planning to run [MyneChat](http://app.myne.chat/)
   alongside, then make sure to pass the `-m` flag with your MyneChat instance URL. Please wait while this script creates
   the local blockchain network and deploys the project contracts. In average, the process can take between 2-6 minutes,
   depending on your computer capacity. **Important**, make sure to have both `curl` and `jq` installed in your computer
   before running the script, as both are used. Please be aware you also need a version of `bash` of `5.x` or superior,
   which in most macOS devices require an upgrade, the easiest being via `brew bash`.

```bash
./scripts/setup-local-cluster.sh -m "http://app.myne.chat" -i scripts/topologies/full_interconnected_cluster.sh
```

Afterwards, a set off accounts with their respective HTTP REST API, HOPR Admin, and WebSocket interface will be displayed
in your screen. For the next steps, we recommend copying and pasting these URLs and `export` them to your terminal so you can
make use of them in the following pages.

```bash
export apiToken=^^LOCAL-testing-123^^ HOPR_NODE_1_HTTP_URL=http://127.0.0.1:13301 HOPR_NODE_1_WS_URL=ws://127.0.0.1:19501 HOPR_NODE_2_HTTP_URL=http://127.0.0.1:13302 HOPR_NODE_2_WS_URL=ws://127.0.0.1:19502 HOPR_NODE_3_HTTP_URL=http://127.0.0.1:13303 HOPR_NODE_3_WS_URL=ws://127.0.0.1:19503 HOPR_NODE_4_HTTP_URL=http://127.0.0.1:13304 HOPR_NODE_4_WS_URL=ws://127.0.0.1:19504 HOPR_NODE_5_HTTP_URL=http://127.0.0.1:13305 HOPR_NODE_5_WS_URL=ws://127.0.0.1:19505
```

[^1]:
    The demo application [MyneChat](https://github.com/hoprnet/myne-chat) uses a
    [mock server](https://github.com/hoprnet/myne-chat/blob/cf6501b2ffa24502834f567ab575630e302e3d34/mocks/index.js#L47-L79)
    to simplify it’s development workflow. Nevertheless, to fully experience the extend of its features, it relies on a
    HOPR cluster, either a local or a public one.

[^2]:
    By using the `master` tag, you are downloading the latest version of `hoprnet` to spin up your nodes, which might be ideal
    to your particular use case. However, due to the rapid development done on the project, you might be better off using a stable
    release. The latest stable release known at the time of writing is [`athens`](https://github.com/hoprnet/hoprnet/archive/refs/heads/release/athens.zip).

[^3]: If you have installed and built another `node.js` application from your computer in the past, you likely will not need to do anything else. However, in the case your are only starting to develop in `node.js`, there's a high chance you might need to install a few extra tools. For instance, in `Linux`-based OS, you will likely also need to install `build-essentials` (e.g. in Ubuntu do `apt-get install build-essentials`), whereas in `macOS` you need Xcode developer tools, installable via `xcode-select --install`.
