[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / libp2p/privKeyToPeerId

# Module: libp2p/privKeyToPeerId

## Table of contents

### Functions

- [privKeyToPeerId](libp2p_privkeytopeerid.md#privkeytopeerid)

## Functions

### privKeyToPeerId

▸ **privKeyToPeerId**(`privKey`: Uint8Array \| _string_): PeerId

Converts a plain compressed ECDSA private key over the curve `secp256k1`
to a peerId in order to use it with libp2p.
It equips the generated peerId with private key and public key.

#### Parameters

| Name      | Type                   | Description           |
| :-------- | :--------------------- | :-------------------- |
| `privKey` | Uint8Array \| _string_ | the plain private key |

**Returns:** PeerId

Defined in: [libp2p/privKeyToPeerId.ts:18](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/privKeyToPeerId.ts#L18)
