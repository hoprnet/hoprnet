[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / interactions/packet/acknowledgement

# Module: interactions/packet/acknowledgement

## Table of contents

### Functions

- [sendAcknowledgement](interactions_packet_acknowledgement.md#sendacknowledgement)
- [subscribeToAcknowledgements](interactions_packet_acknowledgement.md#subscribetoacknowledgements)

## Functions

### sendAcknowledgement

▸ **sendAcknowledgement**(`packet`: [*Packet*](../classes/messages_packet.packet.md), `destination`: PeerId, `sendMessage`: *any*, `privKey`: PeerId): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `packet` | [*Packet*](../classes/messages_packet.packet.md) |
| `destination` | PeerId |
| `sendMessage` | *any* |
| `privKey` | PeerId |

**Returns:** *void*

Defined in: [packages/core/src/interactions/packet/acknowledgement.ts:45](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/interactions/packet/acknowledgement.ts#L45)

___

### subscribeToAcknowledgements

▸ **subscribeToAcknowledgements**(`subscribe`: *any*, `db`: HoprDB, `chain`: HoprCoreEthereum, `pubKey`: PeerId, `onMessage`: (`ackMessage`: [*Acknowledgement*](../classes/messages_acknowledgement.acknowledgement.md)) => *void*): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `subscribe` | *any* |
| `db` | HoprDB |
| `chain` | HoprCoreEthereum |
| `pubKey` | PeerId |
| `onMessage` | (`ackMessage`: [*Acknowledgement*](../classes/messages_acknowledgement.acknowledgement.md)) => *void* |

**Returns:** *void*

Defined in: [packages/core/src/interactions/packet/acknowledgement.ts:12](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/interactions/packet/acknowledgement.ts#L12)
