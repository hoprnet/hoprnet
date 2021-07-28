<!-- ---
description: Your 5 minutes hello world using HOPR.
--- -->

# Getting started

This page will guide you installing and running a HOPR node in your computer on a local testnet.

## Local environment

The following instructions are meant for you to run a local HOPR node. As a result, you will not be able to connect to other peers other than yourself and test your node locally. To learn how to run a "live" HOPR node, please go to [Install a HOPR node](../install-hoprd/index.md).

To get a HOPR node and start working testing the HOPR protocol, follow the next instructions.

### 1. Fetch the latest version

Start by fetching the latest version of `hoprd` from our [repository](https://github.com/hoprnet/hoprnet). You will need to have `node.js@v16`, `yarn` and other additional requirements to get started. For more information, read the repository's [README](https://github.com/hoprnet/hoprnet#readme).

```bash
$ git clone git@github.com:hoprnet/hoprnet.git hoprnet && cd "$_"
```

### 2. Install all dependencies

Before you can do anything, you need to run `yarn`, which will default to `yarn install` and retrieve `hoprd`'s dependencies from [npmjs.org.](http://npmjs.org/)

```bash
$ yarn
```

### 3. Build the source code

Upon installation, you need now to compile the source code. Please bear in mind this process takes a bit of time and it is not unexpected to see it hanging at times.

```bash
$ yarn build
```

### 4. Run a local network

You are ready to build and deploy the **HOPR Channels** contract in your local network, the settlement layer used by HOPR nodes to issue and redeem tickets worth HOPR tokens.

```bash
$ yarn run:network
```

### 5. Run alice and bob

Using different terminals for each process, run now `alice` and `bob`, our pre-configured nodes which can be used to test the application.

```bash
# running normal node alice (separate terminal)
$ DEBUG=hopr* yarn run:hoprd:alice

# running normal node bob (separate terminal)
$ DEBUG=hopr* yarn run:hoprd:bob
```

### 6. Fund your nodes

With all your nodes up and running, you now need to have local HOPR tokens for your nodes to be able to send messages. Use our local `faucet` which would mint both `1 ETH` and `1 HOPR` token for you to use.

```bash
$ yarn run:faucet:all
```

You are ready to go! Go to either `localhost:3000` (bootstrap server), `localhost:3010` (alice) or `localhost:3020` (bob) and type `help` to learn more about what `hoprd` can do.
