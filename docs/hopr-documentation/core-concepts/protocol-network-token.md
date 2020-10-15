---
description: >-
  An introduction to the motivations behind the HOPR Protocol, Network and
  Token, alongside the ideology behind incentivised communication protocols
  using decentralised technologies.
---

# Protocol, Network and Token

## Background

Messaging should be done in a _secure_ way. It might seem intuitive what _secure_ means, but digging deeper quickly reveals how _secure_ communication is a complex issue:

1. _Secure_ communications should prevent unauthorised parties from learning the content of the message. This security goal is known as **confidentiality**, \*\*\*\*and can be achieved by reasonable encryption schemes like [AES](https://en.wikipedia.org/wiki/Advanced_Encryption_Standard).
2. _Secure_ communications allow sending a message in a way that such message arrives unchanged, or at least that any manipulations to the message are observable by the designated receiver. This property is known as **integrity** and can be achieved by using a suitable scheme that generates message authentication codes like [HMAC](https://en.wikipedia.org/wiki/HMAC).

The combination of both schemes \(i.e. **confidentiality** and **integrity**\) yields a _construction_ that allows a sender to **hide the content of the message** and **make the integrity of the message verifiable**.

However, this _construction_ does not hide the fact that a particular sender \(e.g.`Alice`\) and receiver \(e.g`Bob`\) pair have exchanged messages. Unfortunately, this _construction_ leaks an _upper bound_ that shows how much communication took place. A possible adversary, therefore, might also distinguish short conversations from longer ones. If the adversary were also able to observe actions that follow the reception of messages, they might be able reason about the content of the observed encrypted data -- all without breaking the encryption scheme. This shows that **in some cases confidentiality and integrity are not sufficient, and thus, it is also necessary to protect metadata**.

### Anonymity services

To hide who is communicating with whom \(i.e. communication metadata\), a sender and a receiver **need other parties to help them hide this metadata**. Without these additional parties, any communication done between the sender and receiver can be inspected, and as described before, susceptible to [timing attacks](https://en.wikipedia.org/wiki/Timing_attack), independently of the encryption or authentication scheme used in the channel.

To protect themselves, the sender and the receiver need to rely on **anonymity service providers**. In most \(if not all\) cases, these anonymity service providers incur into **economical costs**. These costs include not only buying the necessary hardware, but also recurring costs like electricity or bandwidth. Additionally, there may also be legal costs of dealing with abuse complaints. Perhaps due to their ideological beliefs, these parties may choose to deliver this service for free \(which in this context mean that these parties pay any incurred costs\), but this would rely entirely on the good faith of these anonymity services providers.

### Economical incentives

If there is no incentive to run a service that provides anonymity, people that use these services need to rely on the altruistic nature of the providers who offer such a service for the greater good. Such providers certainly exist, but it would be naive to count on them existing in large enough numbers to support a widespread and reliable network. Furthermore, this becomes especially problematic for anyone who relies on such a network or who wants to run an application on top of such a network at scale.

As a result, it is necessary to encourage participation **by compensating the providers who provide anonymity as a service**. Thus, to protect the network user’s metadata, and to support a secure communication channel, a **compensation scheme** needs to be created to **incentivise** anonymity service providers to run their systems. Only by having an economic-rewarding incentive, we can have self-sustainable communication networks that can be relied on.

### Introducing HOPR

**HOPR** \(pronounced 'hopper'\) is

* a privacy-preserving messaging protocol \(**HOPR Protocol**\),
* a decentralised network \(**HOPR Network**\),
* with an economical incentive \(**HOPR Token**\) on top of a blockchain.

Software and hardware implementations of the **HOPR Protocol** can function as **HOPR Nodes**, which in turn create the _decentralised network_ known as the **HOPR Network.** Users of the **HOPR Network** can relay messages through multiple “hops” using different **HOPR** **Nodes**. In exchange of running stable **HOPR Nodes**, these _intermediate nodes_ get paid via a blockchain using _payment channels_, in the form of a probability of earning a digital token \(**HOPR Token**\) as an economical incentive for their services.

Messages relayed in the **HOPR Network** use a secure packet format to avoid leaking any data about their contents, so neither the sender nor the recipient have to trust **HOPR** **Nodes** in the network. As a result, **HOPR Nodes** are unable to inspect the data relayed, but upon successfully relaying a message, are rewarded in the form of a _probability_ of earning digital currency \(**HOPR Token**\).

## Architecture

The **HOPR Protocol** consists of two main layers: a **message** delivery layer and a **payment** delivery layer.

### Message Delivery

Messages transferred using the **HOPR Protocol** are embedded within SPHINX packet format that provably hides the relation between sender and receiver. These messages are transferred via a network layer created via a peer-to-peer connection between HOPR Nodes. Under the hood, the HOPR Protocol implementation uses `libp2p` in combination with WebRTC to bypass NATs. This allows HOPR Nodes to become intermediate nodes that relay messages and earn HOPR Tokens.

### Payment Layer

The payment layer uses off-chain payments via payment channels to settle balances for HOPR Node operators. In order to process transactions, HOPR Node operators need to stake assets, and upon successful data relay, are given a _ticket_ which has a _probability_ of earning HOPR Tokens. These tokens can then be settled upon closing the payment channel, and used within the network as means of payment for requesting services from other HOPR Nodes.

