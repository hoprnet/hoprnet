[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / crypto/packet/keyDerivation

# Module: crypto/packet/keyDerivation

## Table of contents

### Functions

- [deriveBlinding](crypto_packet_keyderivation.md#deriveblinding)
- [derivePRGParameters](crypto_packet_keyderivation.md#deriveprgparameters)
- [derivePRPParameters](crypto_packet_keyderivation.md#deriveprpparameters)
- [derivePacketTag](crypto_packet_keyderivation.md#derivepackettag)

## Functions

### deriveBlinding

▸ **deriveBlinding**(`secret`: Uint8Array): Uint8Array

Derive the blinding that is applied to the group element
before forwarding the packet

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `secret` | Uint8Array | shared secret with the creator of the packet |

**Returns:** Uint8Array

the blinding

Defined in: [crypto/packet/keyDerivation.ts:20](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/keyDerivation.ts#L20)

___

### derivePRGParameters

▸ **derivePRGParameters**(`secret`: Uint8Array): [*PRGParameters*](crypto_prg.md#prgparameters)

Derive the seed for the pseudo-randomness generator
by using the secret shared derived from the mixnet packet

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `secret` | Uint8Array | shared secret with the creator of the packet |

**Returns:** [*PRGParameters*](crypto_prg.md#prgparameters)

the PRG seed

Defined in: [crypto/packet/keyDerivation.ts:40](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/keyDerivation.ts#L40)

___

### derivePRPParameters

▸ **derivePRPParameters**(`secret`: Uint8Array): [*PRPParameters*](crypto_prp.md#prpparameters)

Derive the seed for the pseudo-random permutation
by using the secret shared with the creator of the packet

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `secret` | Uint8Array | shared secret with the creator of the packet |

**Returns:** [*PRPParameters*](crypto_prp.md#prpparameters)

Defined in: [crypto/packet/keyDerivation.ts:59](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/keyDerivation.ts#L59)

___

### derivePacketTag

▸ **derivePacketTag**(`secret`: Uint8Array): Uint8Array

#### Parameters

| Name | Type |
| :------ | :------ |
| `secret` | Uint8Array |

**Returns:** Uint8Array

Defined in: [crypto/packet/keyDerivation.ts:72](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/keyDerivation.ts#L72)
