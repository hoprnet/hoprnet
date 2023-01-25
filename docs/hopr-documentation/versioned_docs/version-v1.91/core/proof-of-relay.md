---
id: proof-of-relay
title: Proof of Relay
---

There’s a paradox at the heart of building an incentivized mixnet: how do you ensure that nodes properly perform their task of relaying data, rather than just taking the incentive and doing none of the work?

HOPR’s solution is called **proof of relay**.

## Trustless Privacy

We can’t rely on people to follow the rules because it’s the right thing to do. We need to make following the rules the rational and profitable thing to do. This is the cornerstone of many crypto systems, and what makes them **trustless** (a slightly confusing term that means a system is trustworthy because you don’t need to trust anyone member of it). Basically, when the most selfish way to behave is also the way the system wants you to behave, we don’t need to rely on anyone being nice.

In HOPR’s case, we need a way to ensure everyone is fairly rewarded, but only **after** they’ve relayed the data. This mechanism needs to keep everyone honest without explicitly tracking and checking on everyone, or all the privacy work that went into designing the mixnet would be undone.

## Keeping Nodes Honest

In the last episode, we looked at a naive approach to payments where the user would preload payment for the entire route through the network, and at each hop, a node runner would take their share and pass the rest on down the chain.

The problem with this is there’s no guarantees. Why not just take the money and not forward the data? The anonymity of the HOPR network would allow nodes to steal from the system without doing their relay work. Remember, only the sender knows the full route a data packet is supposed to take, and the mixing means there would be no way to track where a route failed.

So we need a way to ensure nodes only get paid **after** they’ve done their relaying. This is an intuitive solution. After all, people normally get paid after doing work, not before. But how can it be achieved?

One simple approach would be to try and have each node in a chain pay the **previous** node. So when Dmytro receives data from Chāo, he sends Chāo’s payment as a reward. But now we run into the same problem again. Why should Dmytro bother? Why not just save the effort and not send the money back? Again, the privacy of the network means there’s no way to get caught.

(This is a slight oversimplification — after all, Chāo knows he sent some data to Dmytro and knows he didn’t receive any payment. But he couldn’t **prove** it was malicious. Things go wrong in networks all the time! And even if Dmytro developed a reputation for not forwarding payments, he could just drop out of the network and start again with a new identity. Again, privacy becomes a double-edged sword.)

So how can we solve this?

## Proof of Relay

HOPR’s innovation is to make each consecutive pair of nodes in a chain **reliant on each other for payment**. Chāo can’t claim payment until he relays data to Dmytro, but Dmytro **also** can’t claim payment until he unlocks Chāo’s share.

This is achieved through a cryptographic technique, but it’s actually simple to understand.

When data is sent through the HOPR network, a payment is generated for each node in the chain. This is locked with a cryptographic key. If you have the whole key, you can claim your payment. But if any part of it is missing, it’s worthless.

These keys are split in half, so you can only claim a payment once you have both halves.

![Proof of Relay](/img/core/proof_of_relay.gif)

As the data passes along the chain, consecutive pairs of nodes swap key halves with each other. Chāo swaps the first half of his key for the second half of Betty’s. He swaps the first half of Dmytro’s key for the second half of his. He can then claim his payment, but only because the data has successfully hopped from Betty to Chāo to Dmytro.

This forces everyone to play by the rules. If I’m a node receiving data from you, neither of us can claim any money unless we swap key halves. There’s no benefit to shirking your responsibility, no loophole for stealing money. The selfish thing to do is to cooperate, meaning everyone’s incentives are perfectly aligned.

## Preserving Privacy

This simple but extremely powerful innovation unlocks a whole world of possibilities. With proof of relay, we can finally build a fully incentivized private mixnet that can grow to an unlimited scale, because we don’t have to rely on finding trustworthy altruistic people to run it.

We’re not out of the woods yet, though. If each payment generates a transaction on a public blockchain, we’re getting dangerously close to accidentally providing a database of everything that happens in the network. HOPR uses **probabilistic payments** to decouple HOPR’s payment layer from the messaging layer, ensuring privacy is preserved.
