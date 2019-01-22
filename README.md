# HOPR (working title)

> Encryption is for free, but you have to pay for anonymity

A privacy-preserving messaging protocol that incentivizes users to participate in the network.

### For further details, see the full [protocol specification on the wiki](../../wiki)

## Technical Demo
There is a standalone demo to showcase the functionality:

### Software Requirements
- `solc` >= 0.5
- `Node.js` >= 11.0
- `npx` 

Please make sure that `solc`, the Solidity compiler, is available in your environment path, see [here](https://solidity.readthedocs.io/en/latest/installing-solidity.html#binary-packages) how to install `solc` on your platform.


On Windows? 👀 here: [Windows Setup](../../wiki/Setup#Windows)

### Account Requirements
- [`Ethereum Key Pair`](../../wiki/Setup/#PrivateKeyGeneration)
- [`Infura API Key`](../../wiki/Setup/#Infura) (Infura calls this a `Product ID`)

### Executing

```sh
git clone https://github.com/validitylabs/messagingProtocol.git
cd messagingProtocol
yarn install

// Do configuration steps below first before preceding! 

yarn compile
yarn demo
```

Then go to `config/` and create a `.secrets.json` similar to the following one.

```json
{
    "infuraRopstenURL": "https://ropsten.infura.io/v3/",
    "infuraApiKey": "INFURA_PRODUCT_ID",
    "infuraRopstenWssURL": "wss://ropsten.infura.io/ws/v3/",
    "fundAccountEthAddress": "YOUR_ETHEREUM_ADDRESS",
    "fundAccountPrivateKey": "YOUR_PRIVATE_KEY"
}
```

Please make sure that you have:
- whitelisted the contract `0xA2AAC5A8A1776e9c8Ef8F70C82B4e5a56Eb08605` in your Infura account
- got some funds on your Ropsten testnet account, if you don't you may want to use the [faucet](https://faucet.ropsten.be/) to receive test ether.

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
