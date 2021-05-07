[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / crypto/packet/filler

# Module: crypto/packet/filler

## Table of contents

### Functions

- [generateFiller](crypto_packet_filler.md#generatefiller)

## Functions

### generateFiller

▸ **generateFiller**(`maxHops`: *number*, `routingInfoLength`: *number*, `routingInfoLastHopLength`: *number*, `secrets`: Uint8Array[]): Uint8Array

Writes the filler bitstring into the header such
that the integrity tag can be computed

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `maxHops` | *number* | amount of relayers to use |
| `routingInfoLength` | *number* | length of additional data to put next to the routing information |
| `routingInfoLastHopLength` | *number* | length of the additional data to put next to the routing information of the last hop |
| `secrets` | Uint8Array[] | shared secrets with the creator of the packet |

**Returns:** Uint8Array

Defined in: [crypto/packet/filler.ts:16](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/filler.ts#L16)
