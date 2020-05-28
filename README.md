# HOPR net

HOPR is a privacy-preserving messaging protocol that incentivizes users to participate in the network. It provides privacy by relaying messages via several relay nodes to the recipient. Relay nodes are getting paid via payment channels for their services.

For further details, see the [protocol specification on the wiki](../../wiki).

Note that the documentation is under active development and does not always represent the latest version of the protocol.

## Development Branch

Please bear in mind the existing documentation does not reflect our actual state in this branch (i.e. `develop`). To follow the existing documentation, please visit our `master` [branch](https://github.com/hoprnet/hopr-core/tree/master).

## Table of Contents

- [HOPR net](#hopr-net)
  - [Table of Contents](#table-of-contents)
- [Setup](#setup)
  - [Dependencies](#dependencies)
  - [Setup hopr-core](#setup-hopr-core)
  - [Joining the Public Testnet](#joining-the-public-testnet)
    - [Start HOPR as a client](#start-hopr-as-a-client)
    - [Get Kovan Ether](#get-kovan-ether)
    - [Get Kovan HOPR Tokens](#get-kovan-hopr-tokens)
    - [Start HOPR as a client again](#start-hopr-as-a-client-again)
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
- [`yarn`](https://yarnpkg.com/en/docs/install) >= 1.22.0

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

### Start HOPR as a client

`hopr-core` ships with a default [.env](.env) file which is configured to connect to the public testnet, let's start `hopr-core` as a client:

```
$ yarn hopr 0

// Congratulations - your HOPR testnet node is ready to go! 
// Please fund your Ethereum Kovan account 0xB0E66Ac9F4A6E04Af5cEF49040443eF8af484Ac4 with some Kovan ETH and Kovan HOPR test tokens
// You can request Kovan ETH from https://faucet.kovan.network
// For Kovan HOPR test tokens visit our Telegram channel at https://t.me/hoprnet

```

You should have received a message similar to this, by default, `hopr-core` will generate a random account for you, you will need the account's address in order to fund the node.

You can find the account's address in the message printed above, ex: `Please fund your Ethereum Kovan account <your address>`

Time to fund your Ethereum Kovan account.

### Get Kovan Ether

The public HOPR testnet is running on the Ethereum Kovan testnet so that you do not have to pay real Ether. Get yourself some [Kovan Ether from the faucet](https://faucet.kovan.network/), and use the address above.

### Get Kovan HOPR Tokens

Ask for HOPR tokens in our [telegram](https://t.me/hoprnet), please provide your address so that an admin can send you tokens.

### Start HOPR as a client again

```
$ yarn hopr 0

// Connecting to bootstrap node...
```

That's it, you are now connected to the public testnet, type `help` to see all available commands to you, or check out the [using hopr guide](#use-hopr).

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

Before we start the bootstrap node, we need to edit the [.env](.env) file to connect to our local testnet, the highlighted lines in `red` need to be replaced with the lines in `green`:

```diff
# Default: false
+ DEVELOP_MODE = true
- DEVELOP_MODE = false

# Bootstrap servers
+ BOOTSTRAP_SERVERS = /ip4/127.0.0.1/tcp/9091/p2p/16Uiu2HAmNqLm83bwMq9KQEZEWHcbsHQfBkbpZx4eVSoDG4Mp6yfX
- BOOTSTRAP_SERVERS = /dns4/bootstrap01.hoprnet.io/tcp/9091/p2p/16Uiu2HAmM8So1akXf5mp4Nkvmq3qgwC6f38oxEfmdrnnr4ichi1T

# Ethereum provider
+ ETHEREUM_PROVIDER = 'ws://127.0.0.1:9545/'
- ETHEREUM_PROVIDER = 'wss://kovan.infura.io/ws/v3/f7240372c1b442a6885ce9bb825ebc36'
```

To start a bootstrap node, run `yarn hopr -b`

```
$ yarn hopr -b
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
$ yarn hopr 0

// Welcome to HOPR!
//
// Available under the following addresses:
//  /ip4/127.0.0.1/tcp/9091/ipfs/16Uiu2HAkzuoWfxBgsgBCr8xqpkjs1RAmtDPxafCUAcbBEonnVQ65
//  /ip4/192.168.0.167/tcp/9091/ipfs/16Uiu2HAkzuoWfxBgsgBCr8xqpkjs1RAmtDPxafCUAcbBEonnVQ65
//  /ip4/192.168.0.14/tcp/9091/ipfs/16Uiu2HAkzuoWfxBgsgBCr8xqpkjs1RAmtDPxafCUAcbBEonnVQ65
```

Repeat this step 3 more times so that you have a total of 4 HOPR nodes running. Make sure to change the parameter `0` that you entered the first time to `1`, `2`, `3` the following times. This starts the HOPR nodes with different private keys and lets you send messages from one to another.

Check out the [using hopr guide](#use-hopr).

## Use HOPR

### Crawl Network

You can actively crawl the network which asks the bootstrap node for available connections:

```terminal
> crawl
Crawling results:
    contacted nodes:: 
        16Uiu2HAmM8So1akXf5mp4Nkvmq3qgwC6f38oxEfmdrnnr4ichi1T
        16Uiu2HAkyFkiCv3dzd5DzHWn9rhFsc9nypwwuZaixGFwsUnPtfTy
        16Uiu2HAmELUVMRA8c6kpESeUUm5AtXFxgYZJppjtMNCbS2dbQCbr
        16Uiu2HAmCpGGTrZg734ydGMcXRugt6iu7AJyCYUK92METkcZwoMr
        16Uiu2HAkvPH7QyxoQWWyHaKh8zvuGMyzSLLVsqWKKdBtPcBMJw3q
    new nodes: 4 nodes
    total: 4 nodes
```

### Check Your Addresses

Sometimes it is good to know your own Ethereum and HOPR address:

```
> myAddress
ethereum:  0x764779b4b1664980157997df3a2bbefb14dc0fdb
HOPR:      16Uiu2HAmHr35rzSigjAba5fkT1mDAEPcU8EBcdRFUVL4gAY7FXE3
```

You can use the HOPR address to send messages to this client from another client.

### Send Message

We can send a message to another node by copying it's HOPR address (we find that by using the `crawl` command).

Type `send [HOPR_ADDRESS]` and hit `[enter]`:

```
> send 16Uiu2HAkyFkiCv3dzd5DzHWn9rhFsc9nypwwuZaixGFwsUnPtfTy
// Do you want to manually set the intermediate nodes? (y, N)
```

Type `N` and hit `[enter]`:

> Note that the two intermediate nodes are chosen at random.

```
> Type in your message and press ENTER to send:
```

Type the message you want to send  and hit `[enter]`:

```
---------- New Packet ----------
Intermediate 0 : 16Uiu2HAmELUVMRA8c6kpESeUUm5AtXFxgYZJppjtMNCbS2dbQCbr
Intermediate 1 : 16Uiu2HAkvPH7QyxoQWWyHaKh8zvuGMyzSLLVsqWKKdBtPcBMJw3q
Destination    : 16Uiu2HAkyFkiCv3dzd5DzHWn9rhFsc9nypwwuZaixGFwsUnPtfTy
--------------------------------
```

There is a lot going on during this first message that we sent. HOPR needs to open payment channels to the next downstream node via which they relay the message to the recipient.

The destination node will show an output like this:

```
---------- New Message ----------
Message "Hello from HOPR!" latency 1019 ms.
---------------------------------
```

The latency of the initial message is always longer than the following messages as we open payment channels on demand. The production release will open payment channels before sending messages to reduce latency but on the other hand there will be some (configurable) artificial delay to allow for efficient packet mixing and thus privacy.
