[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / libp2p/pubKeyToPeerId

# Module: libp2p/pubKeyToPeerId

## Table of contents

### Functions

- [pubKeyToPeerId](libp2p_pubkeytopeerid.md#pubkeytopeerid)

## Functions

### pubKeyToPeerId

▸ **pubKeyToPeerId**(`pubKey`: Uint8Array \| *string*): PeerId

Converts a plain compressed ECDSA public key over the curve `secp256k1`
to a peerId in order to use it with libp2p.

**`notice`** Libp2p stores the keys in format that is derived from `protobuf`.
Using `libsecp256k1` directly does not work.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `pubKey` | Uint8Array \| *string* | the plain public key |

**Returns:** PeerId

Defined in: [libp2p/pubKeyToPeerId.ts:17](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/pubKeyToPeerId.ts#L17)
