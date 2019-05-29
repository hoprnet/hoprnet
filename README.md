# HOPR
HOPR is a privacy-preserving messaging protocol that incentivizes users to participate in the network. It provides privacy by relaying messages via several relay nodes to the recipient. Relay nodes are getting paid via payment channels for their services.

### For further details, see the full [protocol specification on the wiki](../../wiki)

## Technical Demo
There is a standalone demo to showcase the functionality:

### Software Requirements
- [`Node.js`](https://nodejs.org/en/download/) 10.x or 11.x
- [`yarn`](https://yarnpkg.com/en/docs/install)

On Windows? 👀 here: [Windows Setup](../../wiki/Setup#Windows)

### Account Requirements
- [`Ethereum Key Pair`](../../wiki/Setup/#PrivateKeyGeneration)
- [`Infura API Key`](../../wiki/Setup/#Infura) (Infura calls this a `Product ID`)

### Executing

```sh
git clone https://github.com/validitylabs/hopr.git
cd messagingProtocol
yarn install
```

Setup the configuration file below before preceding. Copy and paste the sample `.env.example` 
into an `.env` file and update the setting values in the .env with your own. For more information
on how to generate some of those, see the Account Requirements section before:

```sh
$ cp .env.example .env
```

Then update the following values in your `.env` file.

```markdown
...

# Infura config
INFURA_PROJECT_ID = 0123456789abcdef0123456789abcbde

...

# Demo accounts
FUND_ACCOUNT_ETH_ADDRESS = 0x0123456789abcdef0123456789abcdef01234567
FUND_ACCOUNT_PRIVATE_KEY = 0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef

DEMO_ACCOUNTS = 6
DEMO_ACCOUNT_0_PRIVATE_KEY = 0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
DEMO_ACCOUNT_1_PRIVATE_KEY = 0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
DEMO_ACCOUNT_2_PRIVATE_KEY = 0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
DEMO_ACCOUNT_3_PRIVATE_KEY = 0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
DEMO_ACCOUNT_4_PRIVATE_KEY = 0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
DEMO_ACCOUNT_5_PRIVATE_KEY = 0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
DEMO_ACCOUNT_6_PRIVATE_KEY = 0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef

...
```

Please make sure that you:
- got more than 0.15 ETH on each account. You may want to use the [faucet](https://faucet.ropsten.be/) to receive some test ether and transfer it to the accounts.

Now you can run the demo script via:

```sh
yarn demo
```