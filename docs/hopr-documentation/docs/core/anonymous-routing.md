---
id: anonymous-routing
title: Anonymous Routing
---

Connection metadata can be used to build up a profile of all our activities, both on the Internet and off it.

That is a giant problem for privacy, but how can it be fixed? Metadata is an essential part of how the Internet works. It’s not just as simple as switching it off.

In this episode, we’ll learn about the basics of anonymously routing data through networks like HOPR.

## The Paradox of Anonymous Messaging

Let’s return to the envelope analogy from last episode: imagine you’re sending a message to your friend through the mail, but you don’t trust the people delivering it. How do you keep it secret? One option is to write the message in a code that only you and your friend know. Then, even if someone opens the envelope, the message inside would be nonsense to them. This is loosely equivalent to end-to-end data encryption.

But we saw last time that the problem isn’t really the data, but the metadata. What if you need to ensure that no-one even knows you’re writing to your friend at all? The code approach won’t work. If you write the delivery address in code, the letter will simply never reach its destination.

One solution is to disguise the origin and destination by sending the letter via a chain of people. You prepare a package of envelopes, one inside the other, like a Russian doll. Each one is addressed to the next person in the chain. When someone receives a packet, they open the outermost envelope and send the rest of the package on to the next address. This carries on until your friend receives the final envelope, the one with the message in. Anyone watching just one delivery wouldn’t be able to tell if it’s the destination or just a link in the chain.

![Anonymous routing](/img/core/anonymous_routing.png)

*Each person along the chain removes one layer of encryption before passing it on. No-one can tell where they are exactly in the chain.*

This is a basic analogy for how onion routing works, as used in platforms like the Tor project, which you’ve probably heard of (Tor stands for The Onion Router). In onion routing, the envelopes are actually layers of encryption, so it’s impossible for anyone to open more than one layer, or to look ahead and spy on the route the message will take (or has taken). When you receive data, you only know which node it just came from and which node it goes to next.

This process of passing a message from one recipient to the next is called **relaying**. It’s also what gives HOPR its name, because data ‘hops’ from one person to the next before reaching its destination. The whole collection of relayers is called a **network**, with each point in the network known as a **node**.

This is quite a good system, and for a human it would already be too complicated to track who is sending messages to who. But with enough analysis power it’s still possible to crack networks like this.

If you can track network activity, you can start to find patterns between nodes. If Chāo receives a data packet from Betty and then sends another packet to Dmytro, you can deduce they’re all part of the same chain. Make enough of these deductions, and you can dissolve the privacy of the network. This is even easier if Betty, Chāo or Dmytro (or even all three) are the attackers!

You see, we need to protect against two types of attacker simultaneously: someone who can see all the network activity from the outside, and someone posing as one or more honest nodes, attacking the system from within. Remember, in an open decentralized network it’s important that anyone is able to join. This is crucial for freedom, but makes it harder to defend.

## Packet Splitting

One way to improve security is by splitting our message into different numbered **packets**. When all the packets arrive at the destination, the numbering can be used to reconstruct the entire message. This system is used in standard data transmission on the Internet, but it gains new power in a network like HOPR because you can choose to send each packet via a different route. This proliferation of routes makes it far harder to track what’s going on, and also decreases the information that malicious nodes can access. 
![Packet spliting](/img/core/packet_spliting.gif)

*Splitting data into numbered packets lets us send a message using many different routes, assembling it again at the destination.*

Unfortunately, harder is not the same as impossible. Even with all this complexity, attackers can still use other metadata like timing, message size and connection logs to build up a coherent picture of traffic through the network. We need to do even better.

Luckily, we can take this approach to the next level by introducing **mixing**, which adds a whole extra layer of complexity.

