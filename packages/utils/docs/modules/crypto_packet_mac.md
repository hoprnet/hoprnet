[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / crypto/packet/mac

# Module: crypto/packet/mac

## Table of contents

### Functions

- [createMAC](crypto_packet_mac.md#createmac)

## Functions

### createMAC

▸ **createMAC**(`secret`: Uint8Array, `header`: Uint8Array): Uint8Array

Computes the authentication tag to make the integrity of
the packet header verifiable

#### Parameters

| Name     | Type       | Description                                  |
| :------- | :--------- | :------------------------------------------- |
| `secret` | Uint8Array | shared secret with the creator of the packet |
| `header` | Uint8Array | the packet header                            |

**Returns:** Uint8Array

the authentication tag

Defined in: [crypto/packet/mac.ts:14](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/mac.ts#L14)
