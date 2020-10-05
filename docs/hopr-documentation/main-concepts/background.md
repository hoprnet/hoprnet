---
description: An introduction to the motivations behind HOPR.
---

# Background

Messaging should be done in a _secure_ way. It might seem intuitive what _secure_ means, but digging deeper quickly reveals how _secure_ communication is a complex issue:

1. _Secure_ communications should prevent unauthorised parties from learning the content of the message. This security goal is known as _confidentiality_ and is achieved by reasonable encryption schemes like [AES](https://en.wikipedia.org/wiki/Advanced_Encryption_Standard).
2. The sender of a message also wants to ensure that their message arrives unchanged, or at least that any manipulations to the message are observable by the designated receiver. That property is known as _integrity_ and can be achieved by using a suitable scheme that generates message authentication codes like [HMAC](https://en.wikipedia.org/wiki/HMAC).

The combination of both schemes yields a construction that allows a sender to hide the content of the message and make the integrity of the message verifiable. However, this construction does not hide the fact that _this particular_ sender and receiver pair have exchanged messages. It also leaks an upper bound that shows how much communication took place. A possible adversary might therefore also distinguish short conversations from longer ones. If the adversary were also able to observe actions that follow the reception of messages, they might be able reason about the content of the observed encrypted data -- all without breaking the encryption scheme. This shows that in some cases _confidentiality_ and _integrity_ are not sufficient; it is also necessary to protect metadata.

To hide who is communicating with whom, sender and receiver need other parties to help them hide this metadata. More precisely, sender and receiver will always rely on services provides by other parties. Perhaps due to their ideological beliefs, these parties may choose to deliver this service _for free_, which in this context mean that these parties pay any incurred costs. These costs include not only buying the necessary hardware but also recurring costs like electricity or bandwidth. In addition, there may also be [legal costs](https://trac.torproject.org/projects/tor/wiki/TorRelayGuide#Legalconsiderationsforexitrelayoperators) of dealing with abuse complaints.

If there is **no incentive** to run a service that provides anonymity, people that use this network need to rely on altruistic parties who offer such a service for the greater good. Such parties certainly exist, but it is naive to count on them existing in large enough numbers to support a widespread and reliable network. This becomes especially problematic for anyone who relies on such a network or who wants to run an application on top of such a network at scale.

It is therefore necessary to encourage participation by compensating the parties who provide anonymity as a service.
