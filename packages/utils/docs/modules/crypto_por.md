[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / crypto/por

# Module: crypto/por

## Table of contents

### References

- [deriveAckKeyShare](crypto_por.md#deriveackkeyshare)

### Variables

- [POR\_STRING\_LENGTH](crypto_por.md#por_string_length)

### Functions

- [createFirstChallenge](crypto_por.md#createfirstchallenge)
- [createPoRString](crypto_por.md#createporstring)
- [preVerify](crypto_por.md#preverify)
- [validateAcknowledgement](crypto_por.md#validateacknowledgement)

## References

### deriveAckKeyShare

Re-exports: [deriveAckKeyShare](crypto_por_keyderivation.md#deriveackkeyshare)

## Variables

### POR\_STRING\_LENGTH

• `Const` **POR\_STRING\_LENGTH**: *number*

Defined in: [crypto/por/index.ts:9](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/por/index.ts#L9)

## Functions

### createFirstChallenge

▸ **createFirstChallenge**(`secrets`: Uint8Array[]): *object*

Takes the secrets which the first and the second relayer are able
to derive from the packet header and computes the challenge for
the first ticket.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `secrets` | Uint8Array[] | shared secrets with creator of the packet |

**Returns:** *object*

| Name | Type |
| :------ | :------ |
| `ackChallenge` | Uint8Array |
| `ownKey` | Uint8Array |
| `ticketChallenge` | Uint8Array |

the challenge for the first ticket sent to the first relayer

Defined in: [crypto/por/index.ts:20](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/por/index.ts#L20)

___

### createPoRString

▸ **createPoRString**(`secrets`: Uint8Array[]): *Uint8Array*

Creates the bitstring containing the PoR challenge for the next
downstream node as well as the hint that is used to verify the
challenge that is given to the relayer.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `secrets` | Uint8Array[] | shared secrets with the creator of the packet |

**Returns:** *Uint8Array*

the bitstring that is embedded next to the routing
information for each relayer

Defined in: [crypto/por/index.ts:44](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/por/index.ts#L44)

___

### preVerify

▸ **preVerify**(`secret`: Uint8Array, `porBytes`: Uint8Array, `challenge`: Uint8Array): ValidOutput \| InvalidOutput

Verifies that an incoming packet contains all values that
are necessary to reconstruct the response to redeem the
incentive for relaying the packet

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `secret` | Uint8Array | shared secret with the creator of the packet |
| `porBytes` | Uint8Array | PoR bitstring as included within the packet |
| `challenge` | Uint8Array | ticket challenge of the incoming ticket |

**Returns:** ValidOutput \| InvalidOutput

whether the challenge is derivable, if yes, it returns
the keyShare of the relayer as well as the secret that is used
to create it and the challenge for the next relayer.

Defined in: [crypto/por/index.ts:80](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/por/index.ts#L80)

___

### validateAcknowledgement

▸ **validateAcknowledgement**(`ownKey`: Uint8Array \| *undefined*, `ack`: Uint8Array \| *undefined*, `challenge`: Uint8Array, `ownShare?`: Uint8Array, `response?`: Uint8Array): { `response`: Uint8Array ; `valid`: ``true``  } \| { `valid`: ``false``  }

Takes an the second key share and reconstructs the secret
that is necessary to redeem the incentive for relaying the
packet.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `ownKey` | Uint8Array \| *undefined* | key that as derived from the shared secret with the creator of the packet |
| `ack` | Uint8Array \| *undefined* | second key share as given by the acknowledgement |
| `challenge` | Uint8Array | challenge of the ticket |
| `ownShare?` | Uint8Array | own key share as computed from the packet |
| `response?` | Uint8Array | - |

**Returns:** { `response`: Uint8Array ; `valid`: ``true``  } \| { `valid`: ``false``  }

whether the input values led to a valid response that
can be used to redeem the incentive

Defined in: [crypto/por/index.ts:121](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/por/index.ts#L121)
