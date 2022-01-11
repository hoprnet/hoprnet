---
id: cover-traffic
title: Cover Traffic
---

All the effort to make data packets indistinguishable is only truly effective when there is sufficient traffic. It’s really just a matter of scale: a leaf in the forest is practically invisible, while a leaf in the middle of an otherwise empty floor is going to stand out.

This is particularly problematic for growing networks like HOPR, but even mature global networks have fluctuating levels of traffic. A sufficiently powerful **global adversary** — an attacker who has access to information about the entire network, rather than just traffic through a single node — might be able to gather enough metadata to start linking users to their traffic. We need a way to ensure that everyone is protected even in periods of lower usage.

How can we do that? Well, returning to our leaf analogy, when there’s no forest to hide your leaf in, what do you do? The answer is to grow your own forest. If there isn’t enough data to confuse a global adversary, we need to make more data from somewhere.

Mixnets like HOPR ensure a constant level of privacy by sending arbitrary data through the network. This is called “cover traffic”, because the arbitrary data provides cover for legitimate packets.

Data packets sent in this way function just like normal packets, except they have no meaningful content. So relayers still get paid for relaying them, and they get mixed in with all the data for “real” packets.

In this way, cover traffic ensures that there’s always a minimum level of traffic in the network, providing a blanket of privacy for users.

## Covering the Cost

Cover traffic is not a new concept, but in other mixnet approaches cover traffic is seen as an unfortunate cost which must be paid in exchange for privacy.

HOPR takes a different tack by integrating cover traffic with a second mechanism: staking. The HOPR token is a cryptocurrency, and as with many other cryptocurrencies, HOPR holders expect to be rewarded for holding tokens. Integrating these two concepts takes a necessary evil and turns it into a valuable incentive mechanism.

Staking works by awarding stakers with new tokens proportional to their stake. With HOPR, a node has a higher chance of being chosen to relay cover traffic the higher their stake is. This randomness is important: it prevents the routing from being fully predictable and helps preserve privacy for the staking nodes. Like with probabilistic payments, this randomness introduces some short-term volatility, but this will quickly average out.
