---
id: hoprd
title: Using wxHOPR in HOPRd
---

> Requires `wxHOPR` tokens.

The first thing to understand is that HOPR nodes currently run on wrapped xHOPR (wxHOPR), a wrapped version of the xHOPR token that lives on xDAI Chain. wxHOPR is the only version of the token you should send to your node. wxHOPR exists because gas fees on mainnet Ethereum are currently prohibitively expensive, but HOPR nodes need an ERC-777 compatible token to run, and xHOPR is an ERC-677 token.

Learn how to run and fund a HOPRd node in the following guides:

1. [Installing a HOPR node](https://docs.hoprnet.org/node/start-here)
2. [Running a HOPR node](https://docs.hoprnet.org/node/guide-using-a-hoprd-node)
