[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / crypto/por/keyDerivation

# Module: crypto/por/keyDerivation

## Table of contents

### Functions

- [deriveAckKeyShare](crypto_por_keyderivation.md#deriveackkeyshare)
- [deriveOwnKeyShare](crypto_por_keyderivation.md#deriveownkeyshare)
- [sampleFieldElement](crypto_por_keyderivation.md#samplefieldelement)

## Functions

### deriveAckKeyShare

▸ **deriveAckKeyShare**(`secret`: Uint8Array): *Uint8Array*

Comutes the key share that is embedded in the acknowledgement
for a packet and thereby unlocks the incentive for the previous
relayer for transforming and delivering the packet

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `secret` | Uint8Array | shared secret with the creator of the packet |

**Returns:** *Uint8Array*

Defined in: [crypto/por/keyDerivation.ts:30](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/por/keyDerivation.ts#L30)

___

### deriveOwnKeyShare

▸ **deriveOwnKeyShare**(`secret`: Uint8Array): *Uint8Array*

Computes the key share derivable by the relayer

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `secret` | Uint8Array | shared secret with the creator of the packet |

**Returns:** *Uint8Array*

the key share

Defined in: [crypto/por/keyDerivation.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/por/keyDerivation.ts#L15)

___

### sampleFieldElement

▸ **sampleFieldElement**(`secret`: Uint8Array, `_hashKey`: *string*, `__fakeExpand?`: (`hashKey`: *string*) => Uint8Array): Uint8Array

Samples a field element from a given seed using HKDF
If the result of HKDF does not lead to a field element,
the key identifier is padded until the key derivation
leads to a valid field element

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `secret` | Uint8Array | the seed |
| `_hashKey` | *string* | identifier used to derive the field element |
| `__fakeExpand?` | (`hashKey`: *string*) => Uint8Array | used for testing |

**Returns:** Uint8Array

a field element

Defined in: [crypto/por/keyDerivation.ts:48](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/por/keyDerivation.ts#L48)
