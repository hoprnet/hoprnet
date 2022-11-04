---
id: starting-local-cluster
title: HOPR Cluster Development Setup
---

The best way to learn and develop the HOPR protocol is by interacting with a HOPR node connected inside the HOPR network.
A series of HOPR nodes fully interconnected with each other is called a HOPR cluster.

You can run a HOPR cluster locally for development. This will let you replicate core functionality such as ticket redemption and packet relaying.

## Requirements

:::info INFO

MacOS M1 users will need to follow an extra set of instructions from [NVM](https://github.com/nvm-sh/nvm#macos-troubleshooting) to allow them to use Node.js 16.

Node.js should be compiled under x64 architecute.

:::

You can run a HOPR cluster locally or use [Playground](https://playground.hoprnet.org/) without any installations. A local setup has only been tested for Linux and macOS, if you are using Windows it is recommended to either use Playground or a VM.

## Use the latest release

Due to the rapid development done on the project, you might be better off using a stable
release. The latest stable release known at the time of writing is [`bogota`](https://github.com/hoprnet/hoprnet/archive/refs/heads/release/bogota.zip).

## Local Setup

Running a local setup will give you a similar setup to the one the HOPR team works with on a daily basis. After all dependencies are installed,
this configuration will allow you to develop HOPR apps offline.

1. **Download the latest version of the repository**: Download a local version of our [GitHub repository monorepo](https://github.com/hoprnet/hoprnet/tree/release/bogota)
   with the latest release (`bogota` at the time of writing) and unzip it in your local folder (roughly `~30 Mb` at the time of writing). For the next tasks, we will assume you are within that folder.

```bash
wget https://github.com/hoprnet/hoprnet/archive/refs/heads/release/bogota.zip
unzip bogota.zip
cd hoprnet-release-bogota
```

2. **Install the dependencies of the project and build it**:

In order to install and build the required packages and project modules, make sure you have installed:

- `nodejs@16` (we suggest installing it via [nvm](https://github.com/nvm-sh/nvm), i.e., `nvm install lts/gallium`), and `yarn` (included in `nodejs@16` by running `corepack enable`)
- [Rust toolchain](https://www.rust-lang.org/tools/install) (at least 1.60)

Ideally you will also have basic development toolsets[^1] set up on your computer. If you have installed the above, run the following command. Please bear in mind that this process will take at least 5-10 minutes depending on your computer.

```bash
make deps build
```

3. **Run the one-line setup script**:

Before running the script make sure you have:

- Both `curl` and `jq` installed
- A version of `bash` running that is `5.x` or higher, which for most macOS devices will require an upgrade. The easiest way to do this is via `brew bash`

Run the following script:

```bash
./scripts/setup-local-cluster.sh -i topologies/full_interconnected_cluster.sh
```

Please wait while this script creates
the local blockchain network and deploys the project contracts. On average, the process can take between 2-6 minutes, depending on your computer.

If you are planning to run [MyneChat](http://app.myne.chat/)
alongside your cluster, then make sure to pass the `-m` flag with your MyneChat instance URL, i.e.:

```bash
./scripts/setup-local-cluster.sh -m "http://app.myne.chat" -i topologies/full_interconnected_cluster.sh
```

As the script runs, a set of accounts with their respective HTTP REST API, HOPR Admin, and WebSocket interfaces will be displayed
on your screen. As soon as the script finishes starting up the local cluster, it will suggest that you `source` the `local-cluster.env` file.
This should be done in each terminal you'll be communicating with the local cluster nodes as it will set up all the environment variables
necessary for the following pages.

Alternatively, you can copy and paste these URLs and `export` them to your terminal:

```bash
export apiToken=^^LOCAL-testing-123^^ HOPR_NODE_1_HTTP_URL=http://127.0.0.1:13301 HOPR_NODE_1_WS_URL=ws://127.0.0.1:19501 HOPR_NODE_2_HTTP_URL=http://127.0.0.1:13302 HOPR_NODE_2_WS_URL=ws://127.0.0.1:19502 HOPR_NODE_3_HTTP_URL=http://127.0.0.1:13303 HOPR_NODE_3_WS_URL=ws://127.0.0.1:19503 HOPR_NODE_4_HTTP_URL=http://127.0.0.1:13304 HOPR_NODE_4_WS_URL=ws://127.0.0.1:19504 HOPR_NODE_5_HTTP_URL=http://127.0.0.1:13305 HOPR_NODE_5_WS_URL=ws://127.0.0.1:19505
```

## Playground

Instead of setting up a cluster locally, you can launch a cluster without any installations at [playground.hoprnet.org](https://playground.hoprnet.org/)

Clusters launched through Playground are fully interconnected and prefunded similar to the described local setup.
But will only run for 20 minutes at a time before closing. This is a good alternative for testing and devloping dApps. Simply use the nodes API URL and key when connecting to a node.

[^1]: If you have installed and built another `node.js` application from your computer in the past, you likely will not need to do anything else. However, in the case your are only starting to develop in `node.js`, there's a high chance you might need to install a few extra tools. For instance, in `Linux`-based OS, you will likely also need to install `build-essentials` (e.g. in Ubuntu do `apt-get install build-essentials`), whereas in `macOS` you need Xcode developer tools, installable via `xcode-select --install`.
