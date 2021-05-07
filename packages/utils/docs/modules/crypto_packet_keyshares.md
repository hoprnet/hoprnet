[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / crypto/packet/keyShares

# Module: crypto/packet/keyShares

## Table of contents

### Functions

- [forwardTransform](crypto_packet_keyshares.md#forwardtransform)
- [generateKeyShares](crypto_packet_keyshares.md#generatekeyshares)

## Functions

### forwardTransform

▸ **forwardTransform**(`alpha`: Uint8Array, `privKey`: PeerId): _object_

Applies the forward transformation of the key shares to
an incoming packet.

#### Parameters

| Name      | Type       | Description                                                        |
| :-------- | :--------- | :----------------------------------------------------------------- |
| `alpha`   | Uint8Array | the group element used for the offline Diffie-Hellman key exchange |
| `privKey` | PeerId     | private key of the relayer                                         |

**Returns:** _object_

| Name     | Type       |
| :------- | :--------- |
| `alpha`  | Uint8Array |
| `secret` | Uint8Array |

Defined in: [crypto/packet/keyShares.ts:64](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/keyShares.ts#L64)

---

### generateKeyShares

▸ **generateKeyShares**(`path`: PeerId[]): _object_

Performs an offline Diffie-Hellman key exchange with
the nodes along the given path

#### Parameters

| Name   | Type     | Description                           |
| :----- | :------- | :------------------------------------ |
| `path` | PeerId[] | the path to use for the mixnet packet |

**Returns:** _object_

| Name      | Type         |
| :-------- | :----------- |
| `alpha`   | Uint8Array   |
| `secrets` | Uint8Array[] |

the first group element and the shared secrets
with the nodes along the path

Defined in: [crypto/packet/keyShares.ts:16](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/keyShares.ts#L16)
