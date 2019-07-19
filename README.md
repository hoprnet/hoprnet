# HOPR
HOPR is a privacy-preserving messaging protocol that incentivizes users to participate in the network. It provides privacy by relaying messages via several relay nodes to the recipient. Relay nodes are getting paid via payment channels for their services.

### For further details, see the full [protocol specification on the wiki](../../wiki)

## Proof of Concept
The following is an early and unstable proof of concept that highlights the functionality of HOPR. Use it at your own risk. While we're giving our best to buidl a secure and privacy-preserving base layer of the web of today and tomorrow, we do not guaratee that your funds are safu and we do not guarantee that your communication is really metadata-private.

### Dependencies
The current implementation of HOPR is in JavaScript so you need:
- [`Node.js`](https://nodejs.org/en/download/) >= 10
- [`yarn`](https://yarnpkg.com/en/docs/install)

On Windows? 👀 here: [Windows Setup](../../wiki/Setup#Windows)

### Get HOPR!

Start by cloning this repository and install it:
```sh
git clone https://github.com/validitylabs/hopr.git
cd hopr
yarn install
```

### Setup Ethereum accounts
You need to have an account with (Ropsten testnet) Ether on it to pay relayers for their services and to open payment channels. The software needs access to sign transactions and therefore you need to provide a private key corresponding to an Ethereum address. The private key corresponding to the Ethereum address hodling some Ropsten ETH needs to be configured in the config file.

1. [`Generate an Ethereum private key`](../../wiki/Setup/#PrivateKeyGeneration). 
2. Rename the settings file `.env.example` to `.env`
3. Add your private key (make sure the private key starts with `0x`) to the settings file by replacing the value in the for e.g. private key 0 (you can easily run HOPR from multiple identities, the index of which is passed via command line parameter in the last step):
```markdown
DEMO_ACCOUNTS = 6
DEMO_ACCOUNT_0_PRIVATE_KEY = 0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
DEMO_ACCOUNT_1_PRIVATE_KEY = 0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
DEMO_ACCOUNT_2_PRIVATE_KEY = 0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
DEMO_ACCOUNT_3_PRIVATE_KEY = 0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
DEMO_ACCOUNT_4_PRIVATE_KEY = 0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
DEMO_ACCOUNT_5_PRIVATE_KEY = 0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
DEMO_ACCOUNT_6_PRIVATE_KEY = 0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
```
4. Please make sure that you have more than 0.15 ETH on each account. You may want to use the [faucet](https://faucet.ropsten.be/) to receive some Ropsten testnet Ether and transfer them to funding account.


### Setup Ethereum RPC endpoint
HOPR works with your own node running e.g. locally or a hosted service like Infura (note that this limits your privacy!)

#### Infura setup
1. Sign up for [`Infura and obtain your Project ID`](../../wiki/Setup/#Infura).
2. Replace the Infura Project ID with your own:
```markdown
# Infura config
INFURA_PROJECT_ID = 0123456789abcdef0123456789abcbde
```

#### Local Ethereum node (e.g. Geth)
Overwrite the endpoint that sets up Infura by default
```markdown
PROVIDER = ${PROVIDER_ROPSTEN}
```

with your own (e.g. local) Ethereum node:
```markdown
PROVIDER = http://localhost:8545
```

### Run HOPR!
Now that everything is set up you should be able to run HOPR via
```sh
node hopr 0
```
The parameter `0` references the index of the private key from the settings file that controls some Ropsten testnet Ether (see above).
