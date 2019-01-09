# HOPR (working title)

> Encryption is for free, but you have to pay for anonymity

A privacy-preserving messaging protocol that incentivizes users to participate in the network.

### For further details, see the full [protocol specification on the wiki](../../wiki)

## Technical Demo
There is a standalone demo to showcase the functionality:

Please make sure that `solc`, the Solidity compiler, is available in your environment path, see [here](https://solidity.readthedocs.io/en/latest/installing-solidity.html#binary-packages) how to install `solc` on your platform.

### Executing
```sh
git clone https://github.com/validitylabs/messagingProtocol.git
cd messagingProtocol
yarn install
yarn start
```
### Demo Script
The demo will
- compile the contracts in `contracts/` and deploy them to a local Ganache-core instance
- generate four key pairs
- create four test nodes and equip them with the previously generated key pairs
- start all four nodes such that they listen on some port on your machine
- establish connections between the nodes such that all nodes are transitively connected to each other and DHT lookup is working
- fund the account of each nodes with some test-ether
- create 5 messages and
    - crawl the network recursively
    - sample a random path through the "network"
    - send every other successively through the network
- the nodes will 
    - decrypt the message, process the SPHINX header, extract the embedded information
    - forward the messages and the embedded money
    - wait for an acknowledgement to be able to decrypt the encrypted transactions that they've received during the protocol execution
    - open a payment channel to the next hop in the case there is not yet one
- let one party initiate a payout which will
    - settle & close the payment channels of that party
    - let the nodes listen to the on-chain ClosedChannel event and post a better transaction in case that a malicious party tries to close the channel with an unprofitable transaction
    - withdraw the money