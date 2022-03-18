---
id: about-hopr
title: What is HOPR?
---

The HOPR network is a decentralised and incentivized peer-to-peer network open to anyone who wants to join and run a node. Because of its decentralisation, there is no entity that controls it. No one with special administrator rights, no master node or server to control traffic or access. Instead, all the nodes are peers that, through the simple expedient of running the HOPR protocol, work together to run the network in communal fashion.

Decentralisation ensures that the network is independent, with no one in a position to unduly infuence its development or manipulate outcomes to their advantage. (Not even members of the HOPR Association, who are tasked with managing the network, can control, censor or intercept its traffc.) It also makes the network resilient, able to keep running even if a majority of nodes are damaged or compromised and very diffcult, if not impossible, to shut down.

<div class="embed-container">
<iframe src="https://player.vimeo.com/video/508840889" frameborder="0" allow="autoplay; fullscreen; picture-in-picture" allowfullscreen></iframe>
</div>

## Message Layer

The message layer of the HOPR protocol is designed to solve the problem of how to send a message – or, to be more technically precise, a “data packet” – from one point in a network to another without revealing from where, from whom or when the packet was sent, or where it is going. This is tricky, analogous to posting a letter with - out a to or from address and no stamp, and expecting it to be delivered to the right place on time.

HOPR solves this problem by not sending packets directly from point A to B, but rather through a series of intermediate steps that can be described as from A to receiver Z by way of nodes B, C and D. This process is known as hopping, and gives HOPR its name.

![Packet spliting](/img/core/packet_spliting.gif)
_HOPR gets its name from the fact that it provides metadata privacy by sending data packets through multiple nodes - or “hops” – in the network._

## Payment Layer

The problem that the HOPR payment layer solves is how to reward node operators for forwarding data packets without inadvert - ently revealing metadata about those packets and so defeating the purpose of the network. This has long been recognised as a serious problem in trying to introduce incentivisation for a mixnet, and there are those in the academic community who argue that it is impossible to create a completely private/anonymous payment scheme for an incentivised mixnet.

Here, too, we are very aware of the theoretical, though by no means insignifcant, risks involved. From the outset we have however believed strongly that you cannot have true network-level privacy without a decentralised network, that you cannot have a truly decentralised network without a decentralised means to reward node oper - ators for relaying messages, and that it is possible to devise an approach that mitigates the risks to a more than acceptable degree.

HOPR is a Swiss-based project started by an experienced team of cryptographers and blockchain experts. The HOPR network is designed to provide network-level privacy both by the use of a messaging protocol that obscures metadata so that individual data packets do not reveal trackable information, and a decentralised network that is managed by its users, so that there is no danger of a controlling entity abusing its power. To compensate individuals for the work of running a node, HOPR also contains a native currency and incentivisation scheme.

The details of the payment layer are complex, but at the heart of it lies our proofof-relay scheme. On a high level it can be understood as follows: Every time a node operator forwards a packet they earn the right to receive a payment. Relayers, however, only receive half the information they need to try and claim a reward when they receive the packet. They are given the other half when they have successfully forwarded the packet to the next node on the journey.

## Proof of Relay

HOPR’s innovation is to make each consecutive pair of nodes in a chain reliant on each other for payment. This is achieved through a cryptographic technique, but it’s actually simple to understand.

When data is sent through the HOPR network, a payment is generated for each node in the chain. This is locked with a cryptographic key. If you have the whole key, you can claim your payment. But if any part of it is missing, it’s worthless.

![Proof of Relay](/img/core/proof_of_relay.gif)
_As the data passes along the chain, consecutive pairs of nodes swap key halves with each other. This forces everyone to play by the rules._

This simple but extremely powerful innovation unlocks a whole world of possibilities. With proof of relay, we can finally build a fully incentivized private mixnet that can grow to an unlimited scale, because we don’t have to rely on finding trustworthy altruistic people to run it.

:::info

For more indepth information on HOPR and it’s core concepts, as well as its governance modell and future ambitions, have a look at our [Book of HOPR](https://hoprnet.org/assets/documents/Book_of_HOPR_v1.pdf)

:::

## Node Running

Node runners can stake HOPR tokens in their nodes to be rewarded for this relaying of data. They will receive fees in HOPR for the data they help to pass on, including a large number of tokens to be distributed in the form of [cover traffic](core/cover-traffic).

:::info

To install a hoprd node follow [here](node/start-here).

If you already installed a hoprd node, you can follow a simple guide on how to use it, follow [here](node/guide-using-a-hoprd-node).

:::
