---
id: probabilistic-payments
title: Probabilistic Payments
---

Incentivizing a private mixnet like the HOPR network allows it to scale and reward node runners for the important work they do to provide online privacy. Blockchains allow these incentives to be awarded in a trustless and decentralized way, but it’s essential that blockchain integration doesn’t compromise the metadata privacy of the underlying mixnet.

HOPR uses tickets and payment channels to reduce gas costs and introduce a delay between when data is relayed and when the relay reward is claimed. But this still results in a blockchain transaction linked to every data packet relayed. To break this link, HOPR utilizes **probabilistic payments**.

## The Power of Probability

When you successfully relay data as a HOPR node runner, you receive a cryptographic ticket for your efforts.

Previous explanations in this series simplified tickets by treating them as equivalent to payments. But they actually function more like lottery tickets. Instead of each ticket having a guaranteed reward, a few tickets can be redeemed for a large reward, while most are useless. This is the basis for **probabilistic payments**.

It might not seem desirable to introduce randomness to a payment system, but with the right probability settings and given enough time, this is equivalent to a system where each ticket can be redeemed for a reward, with the added benefit of reduced gas costs and a reduced metadata footprint.

Imagine that relaying one packet one hop through the HOPR network had a guaranteed reward of 1 HOPR. If you relayed 100 packets, you’d receive 100 tickets, which could be redeemed for 100 HOPR. But this would require one hundred blockchain transactions, and one hundred gas payments, significantly cutting into the reward (and maybe even making it worthless on an expensive chain like Ethereum). Those 100 transactions are also 100 opportunities for an adversary to gain important information about the underlying message layer.

Now instead imagine that each ticket has a 1% chance of being redeemed for 100 HOPR, and a 99% chance of being worthless. The expected return on 100 tickets is still 100 HOPR, but this time only one redemption transaction is required. It’s the same reward, but just 1% of the gas costs and 1% of the on-chain data.

Of course, once you add randomness, reality will deviate from theory. Those 100 tickets might include several winners, or none at all. In the very short run, this introduces a certain amount of volatility to the incentives. But because nodes are expected to process millions of packets a day, the odds of the rewards meaningfully deviating from the expected values quickly drops to 0.

These probabilities are just an example, and indeed can be tweaked in the protocol as demand and gas prices require. Reducing the probability of each ticket being a winner reduces the number of on-chain transactions, but also increases the volatility of the rewards. As long as the winning probability of each ticket multiplied by the ticket reward is the same, the payout is equivalent to the case where each ticket can be redeemed.

This allows HOPR to provide low-cost data transfers at high rewards even on busy chains with high gas fees.
