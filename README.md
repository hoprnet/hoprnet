<a href="#"><img src="hopr.png"></a>

---

HOPR is a privacy-preserving messaging protocol that incentivizes users to participate in the network. It provides privacy by relaying messages via several relay nodes to the recipient. Relay nodes are getting paid via payment channels for their services.

For further details, see the [protocol specification on the wiki](../../wiki).

Note that the documentation is under active development and does not always represent the latest version of the protocol.

## Table of Contents

- [Setup](#setup)
  - [Dependencies](#dependencies)
  - [Setup hopr-core](#setup-hopr-core)
  - [Joining the Public Testnet](#joining-the-public-testnet)
    - [Get Kovan Ether](#get-kovan-ether)
    - [Start HOPR as a client](#start-hopr-as-a-client)
- [Setting up a Local Testnet](#setting-up-a-local-testnet)
  - [Setup hopr-ethereum](#setup-hopr-ethereum)
  - [Fund Demo Accounts](#fund-demo-accounts)
  - [Start Bootstrap Node](#start-bootstrap-node)
  - [Start HOPR as a client](#start-hopr-as-a-client-1)
  - [Use HOPR](#use-hopr)
    - [Crawl Network](#crawl-network)
    - [Check Your Addresses](#check-your-addresses)
    - [Send Message](#send-message)

# Setup

## Dependencies

**For Hopr on [Polkadot](https://polkadot.network)**, please follow the instruction [here](#HOPR-on-Polkadot)

The current implementation of HOPR is in JavaScript so you need:

- [`Node.js`](https://nodejs.org/en/download/) >= 12
- [`yarn`](https://yarnpkg.com/en/docs/install) >= 1.19.0

You might need to setup further operating system dependent, please refer to the wiki links below for more details:

- [Ubuntu](../../wiki/Setup#Ubuntu)
- [macOS](../../wiki/Setup#macOS)
- [Windows](../../wiki/Setup#Windows)

## Setup hopr-core

Start by cloning this repository, and installing the dependencies:

```
$ git clone -b jigsaw https://github.com/hoprnet/hopr-core.git
$ cd hopr-core

# in case you are using NVM (Node Versioning Manager), run the following two commands:
$ nvm install 12
$ nvm use

$ yarn install
```

## Joining the Public Testnet

### Get Kovan Ether

The public HOPR testnet is running on the Ethereum Kovan testnet so that you do not have to pay real Ether. Get yourself some [Kovan Ether from the faucet](https://faucet.kovan.network/).

### Start HOPR as a client

`hopr-core` ships with a default `.env` file which is configured to connect to the public testnet, let's start `hopr-core` as a client:

```
$ ./hopr 0
```

That's it, you are now connected to the public testnet, type `help` to see all available commands to you.

# Setting up a Local Testnet

The following is an early and unstable proof of concept (PoC) running a _local_ testnet which highlights the functionality of HOPR. Use it at your own risk. While we are giving our best to build a secure and privacy-preserving base layer of the web of today and tomorrow, we do not guarantee that your funds are safu and we do not guarantee that your communication is really metadata-private.

For the proof of concept, HOPR comes with a built-in chat client that is mostly used for demonstration purposes. It will not be part of HOPR in future releases.

To test everything locally, you will end up with 6 command line tabs (or separate windows):

1. Local Ethereum testnet (Ganache)
2. HOPR bootstrap node
3. HOPR node 0
4. HOPR node 1
5. HOPR node 2
6. HOPR node 3

The HOPR PoC chat app is configured to send messages via 2 intermediate relay nodes to the recipient, thus a total of 4 HOPR nodes is the bare minimum to run the PoC.

**DO NOT USE THE DEFAULT PRIVATE KEYS ON MAIN NET - YOU WILL LOOSE ALL FUNDS ASSOCIATED WITH THOSE KEYS!**

## Setup hopr-ethereum

`hopr-ethereum` will allow us to start a local ethereum network (ganache) and deploy our contracts.

```
$ git clone -b develop https://github.com/hoprnet/hopr-ethereum.git
$ cd hopr-ethereum

# in case you are using NVM (Node Versioning Manager), run the following two commands:
$ nvm install 12
$ nvm use

$ yarn install
```

Once done, we can start our local network running run `yarn network`.

```
$ yarn network
// Successfully started local Ganache instance at 'ws://[::]:9545'.
```

Once Ganache is up and running, open another terminal (in many terminal applications you can simply open a new tab in the terminal via `[Command]` + `[t]`) and run `yarn migrate --network development` to deploy the smart contract. Just FYI, HOPR is using the account associated with the `FUND_ACCOUNT_PRIVATE_KEY` to deploy the smart contract.

```
$ yarn migrate --network development
```

You now have a blank version of the HOPR smart contracts running on your local Ganache node. You can keep using the same terminal for the bootstrap node.

## Fund Demo Accounts

HOPR uses HOPR to stake channels, we will proceed by funding our demo accounts with 100 HOPR each.

```
# go back to parent directory
$ cd ..

$ cd hopr-core
$ yarn fundAccounts
```

## Start Bootstrap Node

HOPR is a decentralized network, so in order to bootstrap the network and tell recently joined nodes about the participants of the network, there needs to be a bootstrap node that is known to all nodes.

Before we start the bootstrap node, we need to edit the `.env` file to connect to our local testnet, the highlighted lines in `red` need to be replaced with the lines in `green`:

```diff
# Default: false
- DEVELOP_MODE = false
+ DEVELOP_MODE = true

# Bootstrap servers
+ BOOTSTRAP_SERVERS = /ip4/127.0.0.1/tcp/9091/p2p/16Uiu2HAmNqLm83bwMq9KQEZEWHcbsHQfBkbpZx4eVSoDG4Mp6yfX
- BOOTSTRAP_SERVERS = /dns4/bootstrap01.hoprnet.io/tcp/9091/p2p/16Uiu2HAmM8So1akXf5mp4Nkvmq3qgwC6f38oxEfmdrnnr4ichi1T

# Ethereum provider
+ ETHEREUM_PROVIDER = 'ws://127.0.0.1:9545/'
- ETHEREUM_PROVIDER = 'wss://kovan.infura.io/ws/v3/f7240372c1b442a6885ce9bb825ebc36'
```

To start a bootstrap node, run `./hopr -b`

```
$ ./hopr -b
// Welcome to HOPR!
//
// Available under the following addresses:
//  /ip4/127.0.0.1/tcp/9091/ipfs/16Uiu2HAm5xi9cMSE7rnW3wGtAbRR2oJDSJXbrzHYdgdJd7rNJtFf
//  /ip4/192.168.1.106/tcp/9091/ipfs/16Uiu2HAm5xi9cMSE7rnW3wGtAbRR2oJDSJXbrzHYdgdJd7rNJtFf
//
// ... running as bootstrap node!.
```

This node allows the other nodes on the network to find each other. We will start these nodes next.

## Start HOPR as a client

Now that everything is set up, open a new terminal window (or tab) and you should be able to run a new HOPR node via

```
$ ./hopr 0

// Welcome to HOPR!
//
// Available under the following addresses:
//  /ip4/127.0.0.1/tcp/9095/ipfs/16Uiu2HAkzuoWfxBgsgBCr8xqpkjs1RAmtDPxafCUAcbBEonnVQ65
//  /ip4/192.168.0.167/tcp/9095/ipfs/16Uiu2HAkzuoWfxBgsgBCr8xqpkjs1RAmtDPxafCUAcbBEonnVQ65
//  /ip4/192.168.0.14/tcp/9095/ipfs/16Uiu2HAkzuoWfxBgsgBCr8xqpkjs1RAmtDPxafCUAcbBEonnVQ65
//
// Own Ethereum address: 0x32C160A5008e517Ce06Df4f7d4a39fFC52E049cf
// Funds: 100 HOPR
// Stake: 0 HOPR
```

You can then follow the on-screen instructions to stake funds (just hit `[enter]` twice to confirm that you want to stake and then add 0.1 HOPR to the payment channel smart contract).

Repeat this step 3 more times so that you have a total of 4 HOPR nodes running. Make sure to change the parameter `0` that you entered the first time to `1`, `2`, `3` the following times. This starts the HOPR nodes with different private keys and lets you send messages from one to another.

## Use HOPR

If all went well this far you have 4 HOPR nodes running (plus a bootstrap node, plus your Ganache instance which is usually a very silent window). Now you are ready to explore the HOPR command line (you can always hit `[tab]` to see available commands or auto-complete):

### Crawl Network

You can actively crawl the network which asks the bootstrap node for available connections:

```
> crawl
['16Uiu2HAkzYUU8dqzQGFkQDMwBc371X84mkYHrFKMGkdZev6tkxzi']: Received 3 new nodes.
['16Uiu2HAkzYUU8dqzQGFkQDMwBc371X84mkYHrFKMGkdZev6tkxzi']: Now holding peer information of 3 nodes in the network.
```

### Check Your Addresses

Sometimes it is good to know your own Ethereum and HOPR address:

```
> myAddress
Ethereum:	0xeb53DDd1Fa419C457956ce7a707073A13377A002
HOPR:		16Uiu2HAkzYUU8dqzQGFkQDMwBc371X84mkYHrFKMGkdZev6tkxzi
```

You can use the HOPR address to send messages to this client from another client.

### Send Message

Open one of the other 3 HOPR nodes and send a message to the first node - first type `send [HOPR_ADDRESS]` and hit `[enter`, then type the message that you want to send and again hit `[enter]`:

```
> send 16Uiu2HAm9cCi8zuTY43J63udgoRMqiUDXXb18wQhge1ztc6v8Yyj
Sending message to 16Uiu2HAm9cCi8zuTY43J63udgoRMqiUDXXb18wQhge1ztc6v8Yyj
Type in your message and press ENTER to send:
Hi from HOPR!
['16Uiu2HAkzYUU8dqzQGFkQDMwBc371X84mkYHrFKMGkdZev6tkxzi']: Received 3 new nodes.
['16Uiu2HAkzYUU8dqzQGFkQDMwBc371X84mkYHrFKMGkdZev6tkxzi']: Now holding peer information of 4 nodes in the network.
['16Uiu2HAkzYUU8dqzQGFkQDMwBc371X84mkYHrFKMGkdZev6tkxzi']: ---------- New Packet ----------
['16Uiu2HAkzYUU8dqzQGFkQDMwBc371X84mkYHrFKMGkdZev6tkxzi']: Intermediate 0 : 16Uiu2HAmBBshUFiFn3o99Kvf4CEFtujY3ZZW69ebLAkTmQvRNQoB
['16Uiu2HAkzYUU8dqzQGFkQDMwBc371X84mkYHrFKMGkdZev6tkxzi']: Intermediate 1 : 16Uiu2HAmEsGQV5Ftmfu7x4dkkBnyc8Q2mWEqgQhNu7s3q1Kwimxp
['16Uiu2HAkzYUU8dqzQGFkQDMwBc371X84mkYHrFKMGkdZev6tkxzi']: Destination    : 16Uiu2HAm9cCi8zuTY43J63udgoRMqiUDXXb18wQhge1ztc6v8Yyj
['16Uiu2HAkzYUU8dqzQGFkQDMwBc371X84mkYHrFKMGkdZev6tkxzi']: --------------------------------
['16Uiu2HAkzYUU8dqzQGFkQDMwBc371X84mkYHrFKMGkdZev6tkxzi']: Encrypting with B/iXluyoLtjBAFTllCp0BfRVGvFtMwS+h9IxUAmz3dk=.
['16Uiu2HAkzYUU8dqzQGFkQDMwBc371X84mkYHrFKMGkdZev6tkxzi']: Listening to close event of channel fd14f0694a412f99e9fd3c3d2d13e99a140da2f11b139d5e4f82ae2d2269ce29
['16Uiu2HAkzYUU8dqzQGFkQDMwBc371X84mkYHrFKMGkdZev6tkxzi']: Listening to opening event of channel fd14f0694a412f99e9fd3c3d2d13e99a140da2f11b139d5e4f82ae2d2269ce29
['16Uiu2HAkzYUU8dqzQGFkQDMwBc371X84mkYHrFKMGkdZev6tkxzi']: Opened payment channel fd14f0694a412f99e9fd3c3d2d13e99a140da2f11b139d5e4f82ae2d2269ce29 with txHash 0xbd276bada32a7bbba5de5da22250d9fd6c8f99d0acb6ceb89edf985950942542. Nonce is now 2.
['16Uiu2HAkzYUU8dqzQGFkQDMwBc371X84mkYHrFKMGkdZev6tkxzi']: Created tx with index 00000000000000000000000000000002 on channel fd14f0694a412f99e9fd3c3d2d13e99a140da2f11b139d5e4f82ae2d2269ce29.
['16Uiu2HAkzYUU8dqzQGFkQDMwBc371X84mkYHrFKMGkdZev6tkxzi']: Received acknowledgement.
```

There is a lot going on during this first message that we sent. HOPR needs to open payment channels to the next downstream node via which they relay the message to the recipient. Now is a good time to inspect the two intermediate nodes and see how they see the incoming payment channel request and open a payment channel themselves to the next downstream node.

```
> ['16Uiu2HAmEsGQV5Ftmfu7x4dkkBnyc8Q2mWEqgQhNu7s3q1Kwimxp']: Listening to opening event of channel cc5fb04ed71c2c0f5205d81511bbb8416c8ca503eae95f3fd2d16da862e154f3
['16Uiu2HAmEsGQV5Ftmfu7x4dkkBnyc8Q2mWEqgQhNu7s3q1Kwimxp']: Listening to close event of channel cc5fb04ed71c2c0f5205d81511bbb8416c8ca503eae95f3fd2d16da862e154f3
['16Uiu2HAmEsGQV5Ftmfu7x4dkkBnyc8Q2mWEqgQhNu7s3q1Kwimxp']: Opened payment channel cc5fb04ed71c2c0f5205d81511bbb8416c8ca503eae95f3fd2d16da862e154f3 with txHash 0x436913ad8a1b31ee43aeabe133bd586a7c3003930f41022b8e08782aa050241b. Nonce is now 1.
['16Uiu2HAmEsGQV5Ftmfu7x4dkkBnyc8Q2mWEqgQhNu7s3q1Kwimxp']: Database index 00000000000000000000000000000001 on channnel cc5fb04ed71c2c0f5205d81511bbb8416c8ca503eae95f3fd2d16da862e154f3.
['16Uiu2HAmEsGQV5Ftmfu7x4dkkBnyc8Q2mWEqgQhNu7s3q1Kwimxp']: Transaction index 00000000000000000000000000000002 on channnel cc5fb04ed71c2c0f5205d81511bbb8416c8ca503eae95f3fd2d16da862e154f3.
['16Uiu2HAmEsGQV5Ftmfu7x4dkkBnyc8Q2mWEqgQhNu7s3q1Kwimxp']: Payment channel exists. Requested SHA256 pre-image of 1314a83d1e587c2c950e75129c8e6e669cfe42724a8961db342e56f71cb71de9 is derivable.
['16Uiu2HAmEsGQV5Ftmfu7x4dkkBnyc8Q2mWEqgQhNu7s3q1Kwimxp']: Received 0.0000000000000001 HOPR on channel cc5fb04ed71c2c0f5205d81511bbb8416c8ca503eae95f3fd2d16da862e154f3.
['16Uiu2HAmEsGQV5Ftmfu7x4dkkBnyc8Q2mWEqgQhNu7s3q1Kwimxp']: Listening to close event of channel 08f72788bc1ee53cd9da611c3ac13f1ed41f801f7189f843fd41847b6f640723
['16Uiu2HAmEsGQV5Ftmfu7x4dkkBnyc8Q2mWEqgQhNu7s3q1Kwimxp']: Listening to opening event of channel 08f72788bc1ee53cd9da611c3ac13f1ed41f801f7189f843fd41847b6f640723
['16Uiu2HAmEsGQV5Ftmfu7x4dkkBnyc8Q2mWEqgQhNu7s3q1Kwimxp']: Opened payment channel 08f72788bc1ee53cd9da611c3ac13f1ed41f801f7189f843fd41847b6f640723 with txHash 0x0077c23a71c927d8e450aa823000e95efbbf6df4cb2b5711f74165571023585a. Nonce is now 2.
['16Uiu2HAmEsGQV5Ftmfu7x4dkkBnyc8Q2mWEqgQhNu7s3q1Kwimxp']: Created tx with index 00000000000000000000000000000002 on channel 08f72788bc1ee53cd9da611c3ac13f1ed41f801f7189f843fd41847b6f640723.
['16Uiu2HAmEsGQV5Ftmfu7x4dkkBnyc8Q2mWEqgQhNu7s3q1Kwimxp']: Forwarding to node 16Uiu2HAm9cCi8zuTY43J63udgoRMqiUDXXb18wQhge1ztc6v8Yyj.
```

The second node shows an output like this - note that the two intermediate nodes are chosen at random, therefore the output might look slightly different:

```
> ['16Uiu2HAmBBshUFiFn3o99Kvf4CEFtujY3ZZW69ebLAkTmQvRNQoB']: Listening to opening event of channel fd14f0694a412f99e9fd3c3d2d13e99a140da2f11b139d5e4f82ae2d2269ce29
['16Uiu2HAmBBshUFiFn3o99Kvf4CEFtujY3ZZW69ebLAkTmQvRNQoB']: Listening to close event of channel fd14f0694a412f99e9fd3c3d2d13e99a140da2f11b139d5e4f82ae2d2269ce29
['16Uiu2HAmBBshUFiFn3o99Kvf4CEFtujY3ZZW69ebLAkTmQvRNQoB']: Opened payment channel fd14f0694a412f99e9fd3c3d2d13e99a140da2f11b139d5e4f82ae2d2269ce29 with txHash 0xbd276bada32a7bbba5de5da22250d9fd6c8f99d0acb6ceb89edf985950942542. Nonce is now 1.
['16Uiu2HAmBBshUFiFn3o99Kvf4CEFtujY3ZZW69ebLAkTmQvRNQoB']: Database index 00000000000000000000000000000001 on channnel fd14f0694a412f99e9fd3c3d2d13e99a140da2f11b139d5e4f82ae2d2269ce29.
['16Uiu2HAmBBshUFiFn3o99Kvf4CEFtujY3ZZW69ebLAkTmQvRNQoB']: Transaction index 00000000000000000000000000000002 on channnel fd14f0694a412f99e9fd3c3d2d13e99a140da2f11b139d5e4f82ae2d2269ce29.
['16Uiu2HAmBBshUFiFn3o99Kvf4CEFtujY3ZZW69ebLAkTmQvRNQoB']: Payment channel exists. Requested SHA256 pre-image of b217cd81f3bc033cc4af3803d3c50e4619681d67c35129259b5eecfa046c46c8 is derivable.
['16Uiu2HAmBBshUFiFn3o99Kvf4CEFtujY3ZZW69ebLAkTmQvRNQoB']: Received 0.0000000000000002 HOPR on channel fd14f0694a412f99e9fd3c3d2d13e99a140da2f11b139d5e4f82ae2d2269ce29.
['16Uiu2HAmBBshUFiFn3o99Kvf4CEFtujY3ZZW69ebLAkTmQvRNQoB']: Listening to close event of channel cc5fb04ed71c2c0f5205d81511bbb8416c8ca503eae95f3fd2d16da862e154f3
['16Uiu2HAmBBshUFiFn3o99Kvf4CEFtujY3ZZW69ebLAkTmQvRNQoB']: Listening to opening event of channel cc5fb04ed71c2c0f5205d81511bbb8416c8ca503eae95f3fd2d16da862e154f3
['16Uiu2HAmBBshUFiFn3o99Kvf4CEFtujY3ZZW69ebLAkTmQvRNQoB']: Opened payment channel cc5fb04ed71c2c0f5205d81511bbb8416c8ca503eae95f3fd2d16da862e154f3 with txHash 0x436913ad8a1b31ee43aeabe133bd586a7c3003930f41022b8e08782aa050241b. Nonce is now 2.
['16Uiu2HAmBBshUFiFn3o99Kvf4CEFtujY3ZZW69ebLAkTmQvRNQoB']: Created tx with index 00000000000000000000000000000002 on channel cc5fb04ed71c2c0f5205d81511bbb8416c8ca503eae95f3fd2d16da862e154f3.
['16Uiu2HAmBBshUFiFn3o99Kvf4CEFtujY3ZZW69ebLAkTmQvRNQoB']: Forwarding to node 16Uiu2HAmEsGQV5Ftmfu7x4dkkBnyc8Q2mWEqgQhNu7s3q1Kwimxp.
```

Finally, inspect the recipient node and see how it received the message from the sender:

```
> ['16Uiu2HAm9cCi8zuTY43J63udgoRMqiUDXXb18wQhge1ztc6v8Yyj']: Listening to opening event of channel 580058d1fbb570fda7beb5c4881b566145cc04d1c75d0995fbc7752a641ce3ea
['16Uiu2HAm9cCi8zuTY43J63udgoRMqiUDXXb18wQhge1ztc6v8Yyj']: Listening to close event of channel 580058d1fbb570fda7beb5c4881b566145cc04d1c75d0995fbc7752a641ce3ea
['16Uiu2HAm9cCi8zuTY43J63udgoRMqiUDXXb18wQhge1ztc6v8Yyj']: Opened payment channel 580058d1fbb570fda7beb5c4881b566145cc04d1c75d0995fbc7752a641ce3ea with txHash 0x12647e94d90d3dd7207afc616ca14a80b3d07e53084dc4295eab177446cfaec0. Nonce is now 1.
['16Uiu2HAm9cCi8zuTY43J63udgoRMqiUDXXb18wQhge1ztc6v8Yyj']: Database index 00000000000000000000000000000001 on channnel 580058d1fbb570fda7beb5c4881b566145cc04d1c75d0995fbc7752a641ce3ea.
['16Uiu2HAm9cCi8zuTY43J63udgoRMqiUDXXb18wQhge1ztc6v8Yyj']: Transaction index 00000000000000000000000000000002 on channnel 580058d1fbb570fda7beb5c4881b566145cc04d1c75d0995fbc7752a641ce3ea.
['16Uiu2HAm9cCi8zuTY43J63udgoRMqiUDXXb18wQhge1ztc6v8Yyj']: Payment channel exists. Requested SHA256 pre-image of c3c954e054685e5b0154064439ad3f324d01309983feab374c992533b6dd6d57 is derivable.


---------- New Message ----------
Message "Hello from HOPR!" latency 1019 ms.
---------------------------------
```

The latency of the initial message is always longer than the following messages as we open payment channels on demand. The production release will open payment channels before sending messages to reduce latency but on the other hand there will be some (configurable) artificial delay to allow for efficient packet mixing and thus privacy.