---
id: tickets-and-payment-channels
title: Tickets and Payment Channels
---

HOPR’s **proof of relay** mechanism resolves the paradox of how to incentivize a private mixnet, by tying a node’s payment to the actions of the nodes before and after it in the relay chain. This breakthrough means we no longer need to rely on node runners’ altruism to build a privacy network: HOPR is a **trustless** system where all node runners can be relied on to cooperate because the most selfish way to behave is also the way that earns the most rewards.

But we’re not quite at the end of the tunnel. Conceptually, proof of relay provides the key to building a fully scalable private network, but there are still questions about how to implement this on a public blockchain without creating high costs or breaking privacy again.

## The Problem with Blockchain

As explained before, the theory behind **proof of relay** but didn’t talk about how this is actually implemented. It’s all very well to talk about “incentives” and “rewards”, but what form should these take? How should rewards be generated and claimed?

It’s interesting to note that, until this point, none of these episodes have talked about blockchain at all. That might seem surprising since HOPR is a crypto project and most readers will have first been introduced to HOPR via the HOPR token. But nothing about the explanation so far required a blockchain.

But HOPR does rely heavily on blockchain, for the simple reason that only a decentralized, trustless platform for transactions and smart contracts meets the privacy requirements discussed in previous episodes.

But a public blockchain creates two problems:

- The first is simply an expense. If each hop in the HOPR network triggers an on-chain transaction, then node runners would have to spend gas to claim each reward; i.e., for every packet, they relay. Since the rewards for running a node need to outweigh the costs to provide a rational incentive, the costs of transmitting data over the network would need to be extremely high.

- The second is to do with privacy. If every hop in the network automatically triggers a transaction on the blockchain, then this leaks a lot of metadata about network usage. Attackers could use the publicly available transaction records to build a picture of who was running a node and using the HOPR network.

## Just the Ticket

HOPR introduces several mechanisms to eliminate metadata leakage in its payment layer. The most important is that relaying data doesn’t automatically trigger a payment. Instead, relaying data results in a cryptographic **ticket**. These tickets can be exchanged via a HOPR smart contract on the blockchain for a reward at any time.

The simple act of introducing an unknown delay already makes it much harder for attackers to learn about the HOPR network from the blockchain data. If relaying a packet automatically created a blockchain transaction, then you could be fairly sure that each reward transaction associated with an address happened at roughly the same time as the node associated with that address related to some data (give or take a few blocks for delays). You could maybe also start to make links between other HOPR transactions, to build a picture of which nodes were sending data at the time rewards were being claimed.

But if a node can wait an arbitrary amount of time before claiming their rewards, then the connection between the time the reward was earned and the time it was redeemed is severed. You could still get an estimate of how much relaying a particular node had done, but on its own, this is much less useful.

## Payment Channels

HOPR also makes use of payment channels to reduce the amount of on-chain data. Payment channels are a common technique in crypto to reduce the amount of transactions needed. Two users fund a payment channel, make transactions between each other, and then when the payment channel closes, only the relative balance differences are recorded to the chain.

This is usually a cost-saving measure, but it also has the advantage of decoupling the on-chain data from the precise transactions involved.

![Payment channels](/img/core/payment_channels.gif)

_Payment channels and tickets work together to create an anonymous payout._

Betty and Chao open a payment channel between their nodes, and fund this with HOPR tokens.

As data is relayed between their nodes, their relative balance in the payment channel changes, and each relay generates a new ticket for either Betty or Chao. These tickets accumulate until one node is ready to claim all their rewards. At this point, the channel is closed, and both nodes claim their rewards for all the relaying they’ve done since the channel was opened.

When the channel closes, the two ending balances are recorded on-chain, and these balances are returned to Betty and Chao’s wallets. The ticket redemptions are also recorded as a transaction, but these can theoretically be aggregated into a single transaction, further reducing gas and metadata (current implementations of HOPR do not yet have ticket aggregation).

## Further Improvements

Using payment channels and tickets goes a long way to decoupling the blockchain transaction data from what’s actually happening in the HOPR network, but there’s still a problem in that each relayed packet generates its own reward which needs to be redeemed on chain. This is very inefficient, and introduces an unacceptable privacy week. HOPR fixes this by using **probabilistic payments**, a method to ensure everyone receives the same reward as in the one ticket, one reward system, but with far fewer on-chain transactions.
