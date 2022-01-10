---
id: mixnets
title: Mixnets 
---

Splitting the data into packets and sending each one on a different route through a network of relay nodes is a good first step to solving the problem of sending data privately, but it’s still possible for a powerful attacker to gather enough information to de-anonymize the network.

A mixnet like HOPR can up the complexity to such a level, where true metadata privacy starts to become a reality.

## The Power of a Network

In our simple examples last time, we looked at **routing** just one message through the network, and then at **splitting** the data into numbered **packets** to add extra protection. Those examples made the concept of routing easier to understand, but this hid an important fact we can use to our advantage: in reality, many people will be using our network at once.

![HOPR network](/img/core/hopr_network.gif)

That means there will be thousands or hundreds of thousands of data packets passing through the network simultaneously. We can use this flow of data to create extra confusion for would-be attackers.

## Mixing It Up

Instead of relaying each packet separately, each node can combine multiple packets from different messages together before sending them as a bundle to another node. This node then separates the packets and recombines them with new ones they’ve received from other nodes. This whole process is known as **mixing**.

Here’s what it might look like from the perspective of a single node:
![Mixing packets](/img/core/mixing_packets.gif)

Here a HOPR node is receiving packets that come from three *different* data transmissions, represented in yellow, light blue and dark blue. As the packets come into the node in groups they are split apart, recombined, and these new combinations of packets are sent out to different nodes as they head to their destination.

**Remember:** although the individual packets are separated in the animation above so you can more easily follow the mixing process, they’re actually all lumped together on each connection. An outsider can’t track any one packet’s route through the node. Some packets are even temporarily held back until future mixing cycles, to disrupt any link between timings of packets entering and leaving a node.

To further obscure this [metadata](https://medium.com/hoprnet/hopr-basics-episode-2-what-is-metadata-8b973ae24871), the HOPR mixnet uses a packet format called **Sphinx** to ensure that all these bundles of packets are easily mixable and indistinguishable. The Sphinx format means every packet passing through the network is the same number of bytes, so it’s impossible to track data based on size.

By creating a large enough mixnet with sufficient data traffic, true privacy for online communication is within reach.

## Problem Solved?

So is this the solution then? Unfortunately not. In fact, it’s just half the problem, and maybe only the easier half. A mixnet is, as you can see, an extremely complicated system. All this encryption, mixing, remixing, and relaying is very computationally expensive compared to just sending data directly and openly. A mixnet is nice in theory, but in reality, someone has to pay for all of this extra effort.

This is known as incentivization, and it’s not as simple as just sending money to nodes. We need a way to provide payments for data transfers which don’t undo all the strong privacy work we’ve done so far in our design. We also need to make sure that unscrupulous nodes can’t use the shroud of anonymity the mixnet provides to take payments without actually relaying data.
