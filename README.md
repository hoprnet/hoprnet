# HOPR
HOPR is a privacy-preserving messaging protocol that incentivizes users to participate in the network. It provides privacy by relaying messages via several relay nodes to the recipient. Relay nodes are getting paid via payment channels for their services.

### For further details, see the full [protocol specification on the wiki](../../wiki)

## Technical Demo
There is a standalone demo to showcase the functionality:

### Software Requirements
- `Node.js` >= 10 (this already ships with `npx`)

On Windows? 👀 here: [Windows Setup](../../wiki/Setup#Windows)

### Account Requirements
- [`Ethereum Key Pair`](../../wiki/Setup/#PrivateKeyGeneration)
- [`Infura API Key`](../../wiki/Setup/#Infura) (Infura calls this a `Product ID`)

### Executing

```sh
git clone https://github.com/validitylabs/messagingProtocol.git
cd messagingProtocol
yarn install
```

Setup the configuration file below before preceding. Copy and paste the sample `.env.example` 
into an `.env` file and update the setting values in the .env with your own. For more information
on how to generate some of those, see the Account Requirements section before:

```sh
$ cp .env.example .env // Then update the valid setting values in the .env file
```

```sh
INFURA_ROPSTEN_URL=https://ropsten.infura.io/v3/
INFURA_API_KEY=
INFURA_ROPSTEN_WSS_URL=wss://ropsten.infura.io/ws/v3/
FUND_ACCOUNT_ETH_ADDRESS=
FUND_ACCOUNT_PRIVATE_KEY=
ROPSTEN_MNEMONIC=
ROPSTEN_HOST=127.0.0.1
RINKEBY_MNEMONIC=
RINKEBY_HOST=127.0.0.1
DEMO_ACCOUNTS=3
DEMO_ACCOUNT_0_PRIVATE_KEY=
DEMO_ACCOUNT_1_PRIVATE_KEY=
DEMO_ACCOUNT_2_PRIVATE_KEY=
DEMO_ACCOUNT_3_PRIVATE_KEY=
```

Please make sure that you:
- have whitelisted the contract ~~`0xd215A90a15Fede2C126352E200999fFE7D32A614`~~ `0x8AB0452dc88EE3BabC9ba40E47eF963000C4bEbb` in your Infura account
- got some funds on your Ropsten testnet account, you may want to use the [faucet](https://faucet.ropsten.be/) to receive test ether.

Now you can run the demo script via:
```
yarn demo
```

### Demo Script
The demo will
- generate four key pairs
- create four test nodes and equip them with the previously generated key pairs
- start all four nodes such that they listen to some port on your machine
- establish connections between the nodes such that all nodes are transitively connected to each other and DHT lookup is working
- fund the corresponding ropsten testnet account of each node with some test ether
- crawl the network to find enough nodes in order to create a path of the desired length
- create 4 messages and send them through the network
- the nodes will 
    - decrypt the message, process the SPHINX header, extract the embedded information
    - forward the messages and the embedded money
    - wait for an acknowledgement to be able to decrypt the encrypted transactions that they've received during the protocol execution
    - open a payment channel to the next hop in the case there is no one yet
- let one party initiate a payout which will
    - settle the payment channels of that party
    - let the nodes listen to the on-chain Settle event and post a better transaction in case that a malicious party tries to close the channel with an unprofitable transaction
    - withdraw the money
