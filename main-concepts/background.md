---
description: An introduction on the motivation for HOPR.
---

# Background

Messaging should be done in a _secure_ way. It seems to be clear what _secure_ means but when digging deeper in the details, the definition of _secure_ communication becomes more complex:

1. _secure_ communications should prevent unauthorized parties from learning the content of the message. This security goal is known as _confidentiality_ and is achieved by reasonable encryption schemes like [AES](https://en.wikipedia.org/wiki/Advanced_Encryption_Standard).
2. the sender of the message also wants to make sure that manipulations to the message are observable by the designated receiver. That property is known as _integrity_ and can be achieved by using a suitable scheme that generates message authentication codes like [HMAC](https://en.wikipedia.org/wiki/HMAC).

The combination of both schemes yields a construction that allows a sender to hide the content of the message and make the integrity of the message verifiable. However, this construction does not hide _that_ sender and receiver have exchanged messages. It also leaks an upper bound that shows how much communication took place. A possible adversary might therefore also distinguish short conversations from longer ones. If the adversary also were able to observe actions that follow the reception of messages, the adversary might be able reason about the content of the observed encrypted data - without breaking the encryption scheme. This shows that in some cases _confidentiality_ and _integrity_ is not enough and it is also mandatory to protect metadata.

To hide who is communicating with whom, sender and receiver need other parties that helps them hiding that kind of information. More precisely, sender and receiver will always rely on services of other parties. There might be parties that deliver this service _for free_ in order to support their ideological beliefs. _For free_ means in that context that these parties pay the incurred costs. These costs include not only the price to buy the requirred hardware but also recurring costs like electricity or bandwidth. In addition, one might also consider [legal costs](https://trac.torproject.org/projects/tor/wiki/TorRelayGuide#Legalconsiderationsforexitrelayoperators) in order to deal with abuse complaints.

If there is **no incentive** to run a service that provides anonymity, people that use this network need to rely on altruistic parties that offer such a service for the greater good. It seems therefore reasonable to compensate the parties that provide anonymity as a service. This becomes especially important if one relies on such a network or wants to run an application on top of such a network at scale.
