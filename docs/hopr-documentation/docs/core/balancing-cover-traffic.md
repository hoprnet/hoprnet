---
id: balancing-cover-traffic
title: Balancing Cover Traffic
---

Cover traffic both provides a blanket of data that ensures all users can use the HOPR network privately, and also serves to provide an income to people who stake HOPR tokens, over and above the fees they earn from relaying packets from actual users.

Tying cover traffic to staking solves a lot of problems, but it also introduces new wrinkles:

- Cover traffic can’t be too predictable. It should emulate real traffic, or it doesn’t provide a cover at all, as a sufficiently powerful adversary could start to tag traffic as real or fake based on observed patterns. Just prioritising nodes by stake would be fully predictable and defeat the purpose.

- HOPR’s primary goal is to provide privacy to its users. Staking information lives on the blockchain, and thus is publicly available. If the link between stake and receiving cover traffic rewards is too direct, then it would be possible for adversaries to start using the metadata to make links between HOPR activity and real-world identities.

- But if there is NOT a direct link between stake and amount of cover traffic received, then what’s the point of staking? If I stake 10,000 HOPR tokens, and you stake 1,000,000 HOPR tokens, then intuitively you should receive 100 times the reward.

- Although there are general reasons to think that level of stake will correlate with node reliability — for example, people who hold more HOPR tokens are more likely to support the goals of the project, and more likely to have the assets necessary to run and maintain high quality nodes — this isn’t necessarily true. While we learned last week that tying cover traffic to staking allows HOPR to avoid the issue of paring down cover traffic to the absolute minimum, we still don’t want to waste resources sending cover traffic to dead nodes. We also don’t want to cut out high quality nodes with lower stakes.

This last point ties into broader problems of centralization and network health. While cover traffic isn’t the primary way nodes receive data to relay — it’s just a cover for the real traffic, after all — the added staking element means cover traffic nodes can play an outsized role in network routing. And while normal HOPR users have full control over the routing their data takes, in reality most users will use automated routing strategies, which are likely to disproportionately include nodes favoured by cover traffic.

If we make node quality the same as node stake, then we end up with the problem we’ve seen countless times in crypto: centralization around a few whales. The whales would get all the cover traffic through their nodes, which would mean they get the most opportunity to demonstrate that their nodes are, which would mean regular users will choose them to relay their real data, further boosting their node’s quality. There would be very few opportunities for other nodes to make an impact on the network, which would harm its growth.

## Quality vs Quantity

This all comes down to a problem of routing. Each cover traffic node has a certain amount of packets to transmit, and associated HOPR tokens to distribute in the form of fees from the [proof of relay](https://medium.com/hoprnet/hopr-basics-proof-of-relay-31ec686e9c11) mechanism. Each time the node transmits a packet, it needs to pick a route through the network to send it. How should it choose? Just using the list of stakes doesn’t work, for the reasons listed above. Regular nodes use a criterion based on the perceived _quality_ of a node (Does it respond to pings? Does it participate in proof of relay quickly and reliably, do other nodes agree it is reliable? etc. etc.). But this is completely divorced from stake, and so not a fair way to distribute rewards.

The answer, perhaps not surprisingly, is to combine both weightings to ensure a balance. Cover traffic nodes form a list of candidate nodes primarily weighted by stake, but if something happens to drastically reduce a node’s quality score, it is removed from the candidate list until its quality is restored. So your node with 1,000,000 HOPR tokens staked is very likely to be chosen to receive cover traffic, but if it drops offline or fails to properly relay a packet, it will take several cycles of repeated pings and other checks before it can be selected again.

## The Random Factor

Finally, there’s a random multiplier to inject sufficient randomness to disrupt any attempt to determine patterns in cover traffic distribution, but not so much that it breaks the link between amount of stake and amount of rewards received. This is similar to the mechanism used for [probabilistic payments](https://medium.com/hoprnet/hopr-basics-probabilistic-payments-3af787fc177), and indeed these two pieces of randomness will stack: since getting a winning ticket out of cover traffic requires overcoming both the cover traffic AND ticket random factors, the data that eventually reaches the blockchain when you claim your reward is unlinkable. Of course, all of this happens so frequently that the randomness very quickly averages out from a node runner’s perspective, meaning no-one is left out of pocket if they run a node for long enough.

In brief, this weighting approach to routing will ensure your node with a 1,000,000 HOPR stake will receive proportionally more than my node with 10,000 HOPR tokens staked, provided it stays online and keeps performing its relaying duties. Randomness will ensure that privacy is maintained for everyone, but without being so random that stake becomes irrelevant.
