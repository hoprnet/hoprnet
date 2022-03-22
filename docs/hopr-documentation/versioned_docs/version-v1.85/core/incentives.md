---
id: incentives
title: Incentives
---

Encryption, mixing, remixing, and relaying is very computationally expensive compared to just sending data directly and openly. We need a way to cover these costs or the network can never grow and thrive.

Some people value online privacy enough that they’re willing to shoulder these costs for no reward. The Tor Project is an example of this model, and it also shows its limitations — while interest in decentralization and online privacy has grown exponentially over the past five years, the number of Tor relays and bridges has stayed [basically constant](https://metrics.torproject.org/networksize.html?start=2016-07-16&end=2021-07-16).

Relying on altruism isn’t sustainable or fair. If we want to build a privacy network which can scale to cover the whole Internet, then nodes need to be paid for the work they do. But who should pay? The obvious answer is the ones who benefit most from the network: the people who use it to transmit data privately.

So why not just do that? You have a group of people who want to pay to send data privately (**users**), and another group who provide that service (**node runners**). We just need to connect them.

## A New Centralization Problem

There are two ways we might try:

- **A subscription model**. Users pay to use the service, the funds are gathered into a pot and then distributed to paying node runners.

- **Direct payment**. Users pay node runners directly as they send data through the mixnet.

The first is conceptually simple, but introduces a huge problem. Who oversees the administration of gathering and distributing the funds? Introducing a centralized authority in charge of managing a comprehensive list of users and node runners would undermine all the work we’ve done so far to build a decentralized mixnet. Even if that authority could be trusted to manage the data (it can’t), it would be a huge target for hacks and external pressure to sell or otherwise disclose user details.

So we need to try and find a way for users to pay node runners, each time they use the network. Since we can’t trust a third-party administrator or service, the most elegant solution is to use the HOPR network itself to manage these payments. After all, a transaction is just another kind of data.

## Pay it Forward

Let’s remember what we’ve learned about the HOPR mixnet in [previous](https://medium.com/hoprnet/hopr-basics-episode-3-anonymous-routing-d9278f7de129) [episodes](https://medium.com/hoprnet/hopr-basics-episode-4-mixnets-dfbac2e6560). Let’s imagine Alejandro is sending data to Zoë. For each packet, Alejandro (or more precisely his node) selects a route which hops through the mixnet via one or more relayers. At each hop, that relayer removes one layer of encryption and sends the rest on to the next hop.

It’s these relayers who need to be paid, so the most logical idea is to include the payment in the data packet. Then as each relayer “unwraps” the packet, they can claim their share of the payment and forward the rest on down the chain. A bit like a digital version of [Pass the Parcel](https://en.wikipedia.org/wiki/Pass_the_parcel).

![Mixing packets](/img/core/mixing_packets.gif)

_Alejandro pays to send data down the whole chain. At each hop, the relay node claims their share of payment then sends the rest to the next relayer._

In doing this, we need to make sure that we don’t introduce payment metadata on the blockchain that allows an attacker to make identifying links between anyone along the chain. If someone can track the payment trail, they could discover that Alejandro was sending data to Zoe. HOPR obscures this payment metadata using tickets with a randomized reward, rather like a lottery ticket. We’ll talk about how and why that works in a future episode.

## Why Play Ball?

For now, there’s a more important problem to consider — one that might seem a bit paradoxical. If nodes have full anonymity and data transmission can’t be tracked, how can we be sure nodes are actually doing the relay work they’re paid for?

If I’m Betty in the example above, and the mixnet is completely anonymous, why should I bother paying to relay the data I receive to Chao, the next node in the chain? Why not take the money and run, confident that the anonymity of the network means no-one can stop me? Here the power of HOPR’s metadata privacy actually works against it. If we need to rely on altruism to ensure data is actually delivered, we’re back to square one.

We need a way to ensure that nodes can only get paid **after** they’ve relayed the data. This is actually one of the main innovations HOPR brings to the table: Proof of Relay.
