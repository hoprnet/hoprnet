<a href="#"><img src="hopr.png"></a>

---

HOPR is a privacy-preserving messaging protocol that incentivizes users to participate in the network. It provides privacy by relaying messages via several relay nodes to the recipient. Relay nodes are getting paid via payment channels for their services.

For further details, see the [protocol specification on the wiki](../../wiki).

Note that the documentation is under active development and does not always represent the latest version of the protocol.

## Table of Contents

- [Setup](#setup)
  - [Dependencies](#dependencies)
  - [Get hopr-core](#get-hopr-core)
  - [Get hopr-ethereum](#get-hopr-ethereum)
- [Proof of Concept - Local Testnet](#proof-of-concept---local-testnet)
  - [Local Ethereum Node (Ganache)](#local-ethereum-node-ganache)
  - [Deploy Contracts](#deploy-contracts)
  - [Fund Demo Accounts](#fund-demo-accounts)
  - [Start Bootstrap Node](#start-bootstrap-node)
  - [Run HOPR](#run-hopr)
  - [Use HOPR](#use-hopr)
    - [Crawl Network](#crawl-network)
    - [Check Your Addresses](#check-your-addresses)
    - [Send Message](#send-message)
- [Public Testnet - Here Be Dragons!](#public-testnet---here-be-dragons)
  - [Get Kovan Ether](#get-kovan-ether)
  - [Set Network](#set-network)
  - [Connect Node (e.g. Infura)](#connect-node-eg-infura)
  - [Bootstrap Node](#bootstrap-node)
  - [Optional API Keys](#optional-api-keys)
  - [HOPR Contract Address](#hopr-contract-address)
  - [Accounts and Keys](#accounts-and-keys)
    - [Single Account Mode (Recommended)](#single-account-mode-recommended)
    - [Multi Account Mode](#multi-account-mode)
  - [Use HOPR Testnet](#use-hopr-testnet)
- [Project Structure](#project-structure)
- [HOPR on Polkadot](#hopr-on-polkadot)
  - [Dependencies](#dependencies-1)
  - [**General information**](#general-information)
  - [Get Substrate module](#get-substrate-module)
  - [Get HOPR](#get-hopr)

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

We will start by cloning two repositories, `hopr-core` and `hopr-ethereum`.

## Get hopr-core

Start by cloning this repository, let `yarn` install the dependencies and change the filename of the example settings file (don't worry, to do a quick local test you don't need to touch the content of the file and default settings work!):

```
$ git clone -b jigsaw https://github.com/hoprnet/hopr-core.git
$ cd hopr-core

# in case you are using NVM (Node Versioning Manager), run the following two commands:
$ nvm install 12
$ nvm use

$ yarn install

$ mv .env.example .env
```

## Get hopr-ethereum

```
$ git clone -b develop https://github.com/hoprnet/hopr-ethereum.git
$ cd hopr-ethereum

# in case you are using NVM (Node Versioning Manager), run the following two commands:
$ nvm install 12
$ nvm use

$ yarn install
```

**DO NOT USE THE DEFAULT PRIVATE KEYS ON MAIN NET - YOU WILL LOOSE ALL FUNDS ASSOCIATED WITH THOSE KEYS!**

# Proof of Concept - Local Testnet

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

## Local Ethereum Node (Ganache)

To start a local Ganache-based testnet, run `yarn network` in `hopr-ethereum`.

```
$ cd hopr-ethereum
$ yarn network
// Successfully started local Ganache instance at 'ws://[::]:9545'.
```

## Deploy Contracts

Once Ganache is up and running, open another terminal (in many terminal applications you can simply open a new tab in the terminal via `[Command]` + `[t]`) and run `yarn migrate --network development` to deploy the smart contract. Just FYI, HOPR is using the account associated with the `FUND_ACCOUNT_PRIVATE_KEY` to deploy the smart contract.

```
$ yarn migrate --network development
```

You now have a blank version of the HOPR smart contracts running on your local Ganache node. You can keep using the same terminal for the bootstrap node.

## Fund Demo Accounts

```
$ cd hopr-core
$ yarn fundAccounts
```

## Start Bootstrap Node

HOPR is a decentralized network, so in order to bootstrap the network and tell recently joined nodes about the participants of the network, there needs to be a bootstrap node that is known to all nodes. The default settings that in your `.env` file are pre-configured to work with the existing keys and is visible to other HOPR nodes running on the same machine, so you can just start it.

To start a bootstrap node, run `npx ts-node hopr -b`

```
$ npx ts-node hopr -b
// Welcome to HOPR!
//
// Available under the following addresses:
//  /ip4/127.0.0.1/tcp/9091/ipfs/16Uiu2HAm5xi9cMSE7rnW3wGtAbRR2oJDSJXbrzHYdgdJd7rNJtFf
//  /ip4/192.168.1.106/tcp/9091/ipfs/16Uiu2HAm5xi9cMSE7rnW3wGtAbRR2oJDSJXbrzHYdgdJd7rNJtFf
//
// ... running as bootstrap node!.
```

This node allows the other nodes on the network to find each other. We will start these nodes next.

## Run HOPR

Now that everything is set up, open a new terminal window (or tab) and you should be able to run a new HOPR node via

```
$ npx ts-node hopr 0

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

# Public Testnet - Here Be Dragons!

The first public testnet has a few caveats and is likely going to be frustrating to test. You have been warned but you will get extra love for moving on. One major issue that we are currently solving is the missing NAT traversal which currently prevents you from receiving messages behind a router, mobile hotspot or alike. You also need to fund your account and adjust settings as outlined below.

## Get Kovan Ether

The public HOPR testnet is running on the Ethereum Kovan testnet so that you do not have to pay real Ether. Get yourself some [Kovan Ether from the faucet](https://gitter.im/kovan-testnet/faucet).

## Set Network

Configure your HOPR node to interface the Kovan testnet by setting the network in the `.env` file

```
# Network
NETWORK = kovan
```

## Connect Node (e.g. Infura)

In order to perform any on-chain interactions, you will need a connection to an Ethereum node running the Kovan testnet. Note that using a third party like Infura limits your privacy! For our testnet purposes it is however sufficient.

1. Sign up for [`Infura and obtain your Project ID`](../../wiki/Setup/#Infura).
2. Insert the Infura Project ID into the `.env` file:

```
# Infura config
INFURA_PROJECT_ID = 0123456789abcdef0123456789abcbde
```

## Bootstrap Node

Uncomment the line of the public bootstrap node in your `.env` file and comment the line with the local bootstrap node that you used for the local testnet. In the end that section of the settings file should look as follows:

```
# Bootstrap servers
BOOTSTRAP_SERVERS = /dns4/hopr.network/tcp/${PORT}/ipfs/16Uiu2HAm5xi9cMSE7rnW3wGtAbRR2oJDSJXbrzHYdgdJd7rNJtFf
# BOOTSTRAP_SERVERS = /ip4/127.0.0.1/tcp/${PORT}/ipfs/16Uiu2HAm3WbbQzwcaN7bVG5LRfYPSVXZydkMN6wHk7FZT69heSg7
```

This allows you to configure your own bootstrap node. For the public testnet we are running a public bootstrap node at hopr.network.

## Optional API Keys

If you want to verify the smart contracts that you deploy on Etherscan and you have set `NETWORK` to something different than `ganache` like `mainnet` or `kovan`, please make sure that you have set an `ETHERSCAN_API_KEY` in your `.env` file such that the contract gets verified on Etherscan. This is only required if you want to deploy the HOPR smart contracts in some public Ethereum net.

## HOPR Contract Address

Make sure to set the smart contract address of the HOPR payment channel. Ensure that this address is valid on the network that you chose above. For the Ethereum Kovan testnet we are currently using the smart contract at address [0xebdb4082b08bcef3e286ac4dfc44b8ca61adcc4f](https://kovan.etherscan.io/address/0xebdb4082b08bcef3e286ac4dfc44b8ca61adcc4f).

## Accounts and Keys

HOPR allows can be run in two modes: single account mode or multi account mode. The single-account mode is closer to a production setting and hence recommended for the public testnet while the multi-account mode is useful for testing multiple accounts on the same machine but it has plaintext private keys in the settings file which is a security risk.

### Single Account Mode (Recommended)

Start hopr without any number parameter and it will ask you to provide a password to encrypt the generated private key. It will then generate your HOPR identity and a corresponding Ethereum address.

```
$ npx ts-node hopr
Welcome to HOPR!

(node:42198) ExperimentalWarning: queueMicrotask() is experimental.
Please type in the password that will be used to encrypt the generated key. ***************

Done. Using peerId 16Uiu2HAmNPVU9L5Fcb4Xn5W1Ws6e4TFSTUonX4YJkGrKyLjSRpdj

<Buffer f8 b6 a0 2c a9 32 b0 1b 16 c8 df 60 a4 6b c9 8a 94 53 86 42 58 44 da 6e 51 73 cf 3e 39 42 b0 88 f2 83 f0 8c 23 3a 37 bd cb 4a 1b dc 89 10 60 72 b8 86 ... 134 more bytes>

Available under the following addresses:
 /ip4/127.0.0.1/tcp/9091/ipfs/16Uiu2HAmNPVU9L5Fcb4Xn5W1Ws6e4TFSTUonX4YJkGrKyLjSRpdj
 /ip4/192.168.1.2/tcp/9091/ipfs/16Uiu2HAmNPVU9L5Fcb4Xn5W1Ws6e4TFSTUonX4YJkGrKyLjSRpdj

Own Ethereum address: 0x94C1289565A62371CaC82D90bAEAA469eb2E77B7
Funds: 0 HOPR
Stake: 0 HOPR

Staked Hopr is less than 0.1 HOPR. Do you want to increase the stake now? (Y/n):
```

You cannot stake yet as your account has no funds, so go ahead and send some Kovan Ether to the Ethereum address shown on your screen. Then stop the HOPR node via `ctrl` + `c` and restart it once the Kovan Ether arrived.

Next time you start the HOPR node (remember to use the same password as before) you will see your balance.

### Multi Account Mode

For demonstration and testing purposes, `hopr` allows you to run multiple instances of itself in the same folder and from the same machine so that you can chat with yourself with the integrated PoC chat app. It will create individual database folders for each instance. This is what we were running in the local testnet when starting hopr with the additional parameter `noder hopr 2` accessing the third private key from the `.env` file. You can

In case you intend to use demo instances with custom keys, make sure that you insert the private keys of these accounts into your `.env` file.

```
DEMO_ACCOUNTS = <number of demo accounts, e.g. 6>
DEMO_ACCOUNT_<number>_PRIVATE_KEY = <private key, e.g. 0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef>
```

If you need help when creating Ethereum accounts and/or obtain Testnet Ether, follow these [instructions](../../wiki/Setup/#PrivateKeyGeneration).

## Use HOPR Testnet

Stake some Kovan Ether and then proceed testing in the same way as with the [local testnet](#use-hopr).

Please keep in mind that currently you can only use the public HOPR testnet with a public IP. If you are using a computer behind wifi router or mobile hotspot then this is usually not the case. Consider testing HOPR on a cloud machine with a public IP.

# Project Structure

The most relevant project folders and files are as follows:

```
.
├── db # generated at startup
├── migrations # Truffle migration scripts
├── src # the hopr source code
|   └── ...
├── .env # configuration
├── config.js # parses the .env file
├── hopr.js #  command-line interface
└── ...
```

# HOPR on Polkadot

## Dependencies

The implementation of HOPR is written in Typescript so you need:

- [`Node.js`](https://nodejs.org/en/download/) >= 12
- [`Typescript`](https://www.typescriptlang.org/index.html#download-links)
- [`yarn`](https://yarnpkg.com/en/docs/install) >= 1.22.0, a package manager for Node.JS
- [`Rust`](https://rust-lang.org), install by following these [instructions](https://rustup.rs)
- [`Substrate 1.0`](https://www.parity.io/substrate/), **the installation is described later**.

You might need to setup further operating system dependent, please refer to the wiki links below for more details:

- [Ubuntu](../../wiki/Setup#Ubuntu)
- [macOS](../../wiki/Setup#macOS)
- [Windows](../../wiki/Setup#Windows)

**Javascript build utilities**

HOPR is written in Typescript and since Node.JS cannot execute Typescript directly, all modules require a transpilation from Typescript to plain Javascript.

You might want to install some dependencies globally:

```
$ npm install -g typescript
$ npm install -g ts-node

# for unit tests
$ npm install -g mocha
```

## **General information**

The setup consists of multiple steps:

1. Download & build the Substrate module
2. Start the Substrate-based chain
3. Fund the accounts that we are going to use
4. Start the HOPR bootstrap node
5. Start (at least) four individual HOPR nodes

First of all, start by creating a new directory, e.g.

```
mkdir hopr
```

## Get Substrate module

```
$ git clone https://github.com/hoprnet/hopr-polkadot
$ cd hopr-polkadot

# Install Rust, if not yet happened
$ curl https://sh.rustup.rs -sSf | sh

# Install required build tools
$ ./scripts/init.sh

# Build the WebAssembly runtime
# REMARK: Substrate will not work without doing this!
$ ./scripts/build.sh

# Build Substrate & Substrate module, this might take some time
$ cargo build

# Check if everything is working
$ cargo test -p hopr-polkadot-runtime

# Purge existing chain records and start new chain
$ ./target/debug/hopr-polkadot purge-chain --dev -y && cargo run -- --dev
```

**Wait approx. 5 seconds before proceeding**

Open another terminal window and run:

```
$ git clone https://github.com/hoprnet/hopr-core-polkadot.git

$ cd hopr-core-polkadot

# In case you have NVM (Node.js Version Manager) installed, run
$ nvm use

$ yarn install

# Check that everything is working
# This will start the Substrate chain and check whether the interactions are working.
$ npx mocha

# Fund the accounts that we are going to use
$ ts-node ./src/scripts/fundAccounts.ts
```

## Get HOPR

Start by cloning this repository, let `yarn` install the dependencies and change the filename of the example settings file (don't worry, to do a quick local test you don't need to touch the content of the file and default settings work!):

```
$ git clone -b jigsaw https://github.com/hoprnet/hopr-core.git
$ cd hopr-core

# in case you are using NVM (Node Versioning Manager), run the following two commands:
$ nvm use

$ yarn install
# if not already installed
# yarn add https://github.com/hoprnet/hopr-core-polkadot\

# print current working directory
$ cwd // e.g. /Volumes/DEV/hopr/hopr-core

# start HOPR bootstrap node
$ rm -Rf ./db && ts-node hopr -b -n polkadot
```

Open four other terminal windows:

```
$ cd <working_directory>

$ nvm use
$ ts-node hopr -n polkadot 0
```

```
$ cd <working_directory>

$ nvm use
$ ts-node hopr -n polkadot 1
```

```
$ cd <working_directory>

$ nvm use
$ ts-node hopr -n polkadot 2
```

```
$ cd <working_directory>

$ nvm use
$ ts-node hopr -n polkadot 3
```

Pick one client and type in `crawl`. It should find three other nodes:

```
Crawling results:
    contacted nodes::
        16Uiu2HAmEv6BiCD3p6uwYLQ8Pv3bSLo4SXVsmEExfnZMKYVtUDDL
        16Uiu2HAkzuoWfxBgsgBCr8xqpkjs1RAmtDPxafCUAcbBEonnVQ65
        16Uiu2HAmRfR1Qus69Lhn4t9gCkHjdsrWHpnwJwN7Dbjw197rZLqk
        16Uiu2HAmU5yD9T6behtHat9wThbVajcpQqkP8m4Lqdx75hNkgWma
    new nodes: 3 nodes
    total: 3 nodes
```

Now check your balance by executing `balance`

```
Account Balance:   1152921504606822526 HOPR tokens
```

Check your address:

```
polkadot:  GYrUJGMNYgbG94HUwRf8TUvZ833xHumcjj9XhufhRva
HOPR:      16Uiu2HAmVbdbHbu7ziqzwV8xvdrtSRouzuFb53wEukQmisTi8W3M
```

Send a message by executing `send`. Hint: Use tab completion for the ID of the recipient.

```
Sending message to 16Uiu2HAkzuoWfxBgsgBCr8xqpkjs1RAmtDPxafCUAcbBEonnVQ65
Type in your message and press ENTER to send:
Hello World!
Crawling results:
    contacted nodes::
        16Uiu2HAmRfR1Qus69Lhn4t9gCkHjdsrWHpnwJwN7Dbjw197rZLqk
        16Uiu2HAmU5yD9T6behtHat9wThbVajcpQqkP8m4Lqdx75hNkgWma
        16Uiu2HAmEv6BiCD3p6uwYLQ8Pv3bSLo4SXVsmEExfnZMKYVtUDDL
        16Uiu2HAkzuoWfxBgsgBCr8xqpkjs1RAmtDPxafCUAcbBEonnVQ65
    new nodes: 0 nodes
    total: 4 nodes
---------- New Packet ----------
Intermediate 0 : 16Uiu2HAmU5yD9T6behtHat9wThbVajcpQqkP8m4Lqdx75hNkgWma
Intermediate 1 : 16Uiu2HAmRfR1Qus69Lhn4t9gCkHjdsrWHpnwJwN7Dbjw197rZLqk
Destination    : 16Uiu2HAkzuoWfxBgsgBCr8xqpkjs1RAmtDPxafCUAcbBEonnVQ65
```

Node `16Uiu2HAkzuoWfxBgsgBCr8xqpkjs1RAmtDPxafCUAcbBEonnVQ65` should display something like:

```
===== New message ======
Message: Hello World!
Latency: 52217
========================
```
