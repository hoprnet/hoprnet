---
description: Key features of HOPR.
---

# Key features

- Security properties:
  - Integrity
  - Confidentiality through **end-to-end** encryption
  - Sender\* **&** receiver anonymity\*\*
- Incentivations for relay operators that:
  - preserve the previously mentioned security guarantees
  - allow efficient & automatic on-chain verification of inappropriate transactions in order to punish the sender of that transaction
  - allow them to have a **working business model** that covers their expenses
  - gets along without using inefficient cryptographic building blocks like [homomorphic encryption](https://en.wikipedia.org/wiki/Homomorphic_encryption) and [zero-knowledge proofs](https://en.wikipedia.org/wiki/Zero-knowledge_proof)
- Decentralized message delivery & decentralized directory service through [WebRTC](https://webrtc.org) in combination with [libp2p](https://libp2p.io)
- Token & blockchain agnostic:
  - token must be transferrable through payment channels
  - blockchain must support smart contract that allow: string/byte concatenation, hashing & signature verification of chosen messages
