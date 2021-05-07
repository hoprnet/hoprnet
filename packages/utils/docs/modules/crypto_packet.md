[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / crypto/packet

# Module: crypto/packet

## Table of contents

### References

- [generateKeyShares](crypto_packet.md#generatekeyshares)

### Functions

- [createPacket](crypto_packet.md#createpacket)
- [forwardTransform](crypto_packet.md#forwardtransform)
- [getHeaderLength](crypto_packet.md#getheaderlength)
- [getPacketLength](crypto_packet.md#getpacketlength)

## References

### generateKeyShares

Re-exports: [generateKeyShares](crypto_packet_keyshares.md#generatekeyshares)

## Functions

### createPacket

▸ **createPacket**(`secrets`: Uint8Array[], `alpha`: Uint8Array, `msg`: Uint8Array, `path`: PeerId[], `maxHops`: *number*, `additionalDataRelayerLength`: *number*, `additionalDataRelayer`: Uint8Array[], `additionalDataLastHop?`: Uint8Array): Uint8Array

Creates a mixnet packet

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `secrets` | Uint8Array[] | - |
| `alpha` | Uint8Array | - |
| `msg` | Uint8Array | payload to send |
| `path` | PeerId[] | nodes to use for relaying, including the final destination |
| `maxHops` | *number* | maximal number of hops to use |
| `additionalDataRelayerLength` | *number* | - |
| `additionalDataRelayer` | Uint8Array[] | additional data to put next to each node's routing information |
| `additionalDataLastHop?` | Uint8Array | additional data for the final destination |

**Returns:** Uint8Array

the packet as u8a

Defined in: [crypto/packet/index.ts:65](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/index.ts#L65)

___

### forwardTransform

▸ **forwardTransform**(`privKey`: PeerId, `packet`: Uint8Array, `additionalDataRelayerLength`: *number*, `additionalDataLastHopLength`: *number*, `maxHops`: *number*): LastNodeOutput \| RelayNodeOutput

Applies the transformation to the header to forward
it to the next node or deliver it to the user

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `privKey` | PeerId | private key of the relayer |
| `packet` | Uint8Array | incoming packet as u8a |
| `additionalDataRelayerLength` | *number* | length of the additional data next the routing information of each hop |
| `additionalDataLastHopLength` | *number* | lenght of the additional data for the last hop |
| `maxHops` | *number* | maximal amount of hops |

**Returns:** LastNodeOutput \| RelayNodeOutput

whether the packet is valid, if yes returns
the transformed packet, the public key of the next hop
and the data next to the routing information. If current
hop is the final recipient, it returns the plaintext

Defined in: [crypto/packet/index.ts:128](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/index.ts#L128)

___

### getHeaderLength

▸ **getHeaderLength**(`maxHops`: *number*, `additionalDataRelayerLength`: *number*, `additionalDataLastHopLength`: *number*): *number*

#### Parameters

| Name | Type |
| :------ | :------ |
| `maxHops` | *number* |
| `additionalDataRelayerLength` | *number* |
| `additionalDataLastHopLength` | *number* |

**Returns:** *number*

Defined in: [crypto/packet/index.ts:28](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/index.ts#L28)

___

### getPacketLength

▸ **getPacketLength**(`maxHops`: *number*, `additionalDataRelayerLength`: *number*, `additionalDataLastHopLength`: *number*): *number*

#### Parameters

| Name | Type |
| :------ | :------ |
| `maxHops` | *number* |
| `additionalDataRelayerLength` | *number* |
| `additionalDataLastHopLength` | *number* |

**Returns:** *number*

Defined in: [crypto/packet/index.ts:39](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/index.ts#L39)
