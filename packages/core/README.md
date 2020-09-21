# HOPR

![Terraform](https://github.com/hoprnet/hopr-devops/workflows/Terraform/badge.svg)
![Node.js CI](https://github.com/hoprnet/hopr-core/workflows/Node.js%20CI/badge.svg)

HOPR is a privacy-preserving messaging **protocol** which enables the creation of a secure communication network via relay nodes powered by economic incentives using digital tokens.

This repository is the source code of the TypeScript implementation ([`hopr-core`](https://www.npmjs.com/package/@hoprnet/hopr-core)) of HOPR.

For further information about HOPR, please see our [online documentation](https://docs.hoprnet.io/home/).

## Testnet

Our testnet is live, and its composed of:

- **Bootstrap Nodes**, online servers available in a Swiss datacenter running instances of [HOPR Chat](https://github.com/hoprnet/hopr-chat) on _Bootstrap Mode_.

All our development is done in our [development branch](https://github.com/hoprnet/hopr-core/tree/develop).

### Testnet Nodes

Our testnet Bootstrap nodes are available in the following URLs:

```
/dns4/ch-test-01.hoprnet.io/tcp/9091/p2p/16Uiu2HAmThyWP5YWutPmYk9yUZ48ryWyZ7Cf6pMTQduvHUS9sGE7
/dns4/ch-test-02.hoprnet.io/tcp/9091/p2p/16Uiu2HAmBSzk28qQ8bfpwVgEjef4q51kGg8GjEk3MinyyTB2WTGn
/dns4/ch-test-03.hoprnet.io/tcp/9091/p2p/16Uiu2HAm4H1ZxPb9KkoYD928Smrjnr2igYP8vBFbZKs5B8gchTnT
```

These nodes are behind HOPR Services AG DNS registry. In case you want to directly access them without the DNS request, you can simply pass these directly to your **HOPR Chat** instance.

```
/ip4/34.65.237.196/tcp/9091/p2p/16Uiu2HAmThyWP5YWutPmYk9yUZ48ryWyZ7Cf6pMTQduvHUS9sGE7
/ip4/34.65.119.138/tcp/9091/p2p/16Uiu2HAmBSzk28qQ8bfpwVgEjef4q51kGg8GjEk3MinyyTB2WTGn
/ip4/34.65.120.13/tcp/9091/p2p/16Uiu2HAm4H1ZxPb9KkoYD928Smrjnr2igYP8vBFbZKs5B8gchTnT
```
