# HOPR

HOPR is a privacy-preserving messaging **protocol** which enables the creation of a secure communication network via relay nodes powered by economic incentives using digital tokens.

## chat-bot

Given a [hopr-server](https://github.com/hoprnet/hopr-core/tree/develop/server) `API_URL` it will listen messages send to and reply back with a randomly generated sentence.

This is a very simple example on how to use [hopr-server](https://github.com/hoprnet/hopr-core/tree/develop/server).

## Payload format

In order for the bot to know to which address it should send a reply back to, we need to include the sender's address in the payload.
The bot expects that the first `53 bytes` of the payload will be the sender's address in a base58 format, for example `16Uiu2HAkwsN4GVHQr1szVurz6u4V6uB9ZJacey471Pg2nTxHvP47`, everything after that is the actual message.
This is implemented in [message.ts](./src/message.ts).

### Example (using hopr-chat)

```terminal
# find out your address
> myAddress
ethereum:  0xa51e98e0d9b6c387139e5ce06c2580e922d1a46b
HOPR:      16Uiu2HAkwsN4GVHQr1szVurz6u4V6uB9ZJacey471Pg2nTxHvP47

> send 16Uiu2HAm6rVeEmviiSoX67H5fqkTQq2ZyP2QSRwrmtEuTU9pWeKj

# include your address in the message
> 16Uiu2HAkwsN4GVHQr1szVurz6u4V6uB9ZJacey471Pg2nTxHvP47hello world
```

## Setup
* to setup the payment channel munually create a campaign at [linkdrop](https://dashboard.linkdrop.io/). Choose the same account and erc20 token address that you provide in the .env file. Make sure it is a campaign id is 1 to perform one to one transactions multiple times. You can choose any other campaign id but then the payment channel would be generated only for the number of links decided on the time of campaign creation. 
* copy the .env.example to .env and add the required variables.

## Start

1. Install dependancies: `yarn`
2. Start bot: `API_URL=127.0.0.1:50051 yarn start`
