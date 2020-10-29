<p align="center"><a href="https://hoprnet.org" target="_blank" rel="noopener noreferrer"><img width="100" src="https://github.com/hoprnet/hopr-assets/blob/master/v1/logo/hopr_logo_padded.png?raw=true" alt="HOPR Logo"></a></p>
<h2 align="center">HOPR Chatbot</h2>

**HOPR** is a privacy-preserving messaging **protocol** which enables the creation of a secure communication network via relay nodes powered by economic incentives using digital tokens. **HOPR Chatbot** is a proof-of-concept aimed to showcase the capabilities of a **HOPR Node** by using our gRPC-enabled **HOPR Server**, which listens to messages and replies to the recipients of them, whenever their address is included in the original message.

### Existing Bots

- [🃏 Randobot](./src/randobot/index.ts): Generates random words when messaged.
- [🐦 Tweetbot](./src/tweetbot/index.ts): Asks for a tweet including your address, #HOPRGames and @hoprnet in it.

### Bots in Progress

- [🥊 Bouncerbot](./src/bouncerbot/index.ts): Blocks you from entering a “party”, but will give you some hints to enter afterwards. See [issue](https://github.com/hoprnet/hopr-chatbot/issues/9).

## Requirements

- Docker
- Node.js >=v12

## Setup

### HOPR Server

We have bundled a working version of [**HOPR Server**](https://github.com/hoprnet/hopr-server) in this repository as a `docker-compose.yml`. Just run `docker-compose up server` to get an instance running locally at `127.0.0.1:50051`.

### Chatbot

1. Install dependancies: `yarn`
2. Start bot: `API_URL=127.0.0.1:50051 yarn start`

## Additional information

### LinkDrop Setup

- To setup LinkDrop, you have to manually create a campaign at [linkdrop](https://dashboard.linkdrop.io/). Choose the same account and erc20 token address that you provide in the .env file. Make sure it is a campaign id is 1 to perform one to one transactions multiple times. You can choose any other campaign id but then the payment channel would be generated only for the number of links decided on the time of campaign creation.

- Copy the .env.example to .env and add the required variables.

### Payload format

In order for the bot to know to which address it should send a reply back to, we need to include the sender's address in the payload.
The bot expects that the first `53 bytes` of the payload will be the sender's address in a base58 format, for example `16Uiu2HAkwsN4GVHQr1szVurz6u4V6uB9ZJacey471Pg2nTxHvP47`, everything after that is the actual message.
This is implemented in [message.ts](./src/message.ts).

As of version `1.3.0` of [**HOPR Chat**](https://github.com/hoprnet/hopr-chat), the command `includeRecipient` will effectively add the recipient address into every message.

#### Example (using hopr-chat)

**<1.3.0**

```terminal
# find out your address
> myAddress
ethereum:  0xa51e98e0d9b6c387139e5ce06c2580e922d1a46b
HOPR:      16Uiu2HAkwsN4GVHQr1szVurz6u4V6uB9ZJacey471Pg2nTxHvP47

> send 16Uiu2HAm6rVeEmviiSoX67H5fqkTQq2ZyP2QSRwrmtEuTU9pWeKj

# include your address in the message
> 16Uiu2HAkwsN4GVHQr1szVurz6u4V6uB9ZJacey471Pg2nTxHvP47hello world
```

**>=1.3.0**

```terminal
# call includeRecipient
> includeRecipient
Are you sure you want to include your address in your messages? (y, N):
You have set your “includeRecipient” settings to yes

> send 16Uiu2HAm6rVeEmviiSoX67H5fqkTQq2ZyP2QSRwrmtEuTU9pWeKj

# send message as normal
> hello world
```
