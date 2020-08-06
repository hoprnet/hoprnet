# HOPR net

HOPR is a privacy-preserving messaging protocol that incentivizes users to participate in the network. It provides privacy by relaying messages via several relay nodes to the recipient. Relay nodes are getting paid via payment channels for their services.

## hopr-core-connector-interface

This repo defines the interface between [`hopr-core`](https://github.com/hoprnet/hopr-core) and blockchains that are able to process HOPR incentives. The details of the on-chain logic are left to the implementor and must follow the [whitepaper](https://github.com/hoprnet/hopr-whitepaper), examples are [`hopr-polkadot`](https://github.com/hoprnet/hopr-polkadot) and [`hopr-ethereum`](https://github.com/hoprnet/hopr-ethereum).

## **Chain requirements**

The HOPR protocol comes with some requirements that must be met by chains that process its incentives. As HOPR is meant to be open and inclusive, the requirements for on-chain operations are intentionally kept very basic in order to not exclude any specific chain. Nevertheless, the following requirements are mandatory:

- On-chain hash verification of custom bitstrings
- On-chain signature verification of custom bitstrings
- On-chain binary operations on custom bitstrings
- A reasonable instruction limit to support small programs
- A general-purpose programming language to define the on-chain logic

### Probably unsupported chains (in alphabetical order):

- Bitcoin
- Bitcoin Cash (and derivates)

This holds also for all other blockchain that rely on Bitcoin Script and / or come with a very restrictive instruction limit.

## Implementors (in alphabetical order):

- [Ethereum](https://ethereum.org), see [`hopr-core-ethereum`](https://github.com/hoprnet/hopr-core-ethereum)
- [Polkadot](https://polkadot.network), see [`hopr-core-polkadot`](https://github.com/hoprnet/hopr-core-polkadot)
