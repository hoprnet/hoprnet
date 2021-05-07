[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / interactions/packet/acknowledgement

# Module: interactions/packet/acknowledgement

## Table of contents

### Functions

- [sendAcknowledgement](interactions_packet_acknowledgement.md#sendacknowledgement)
- [subscribeToAcknowledgements](interactions_packet_acknowledgement.md#subscribetoacknowledgements)

## Functions

### sendAcknowledgement

▸ **sendAcknowledgement**(`packet`: [_Packet_](../classes/messages_packet.packet.md), `destination`: PeerId, `sendMessage`: _any_, `privKey`: PeerId): _void_

#### Parameters

| Name          | Type                                             |
| :------------ | :----------------------------------------------- |
| `packet`      | [_Packet_](../classes/messages_packet.packet.md) |
| `destination` | PeerId                                           |
| `sendMessage` | _any_                                            |
| `privKey`     | PeerId                                           |

**Returns:** _void_

Defined in: [packages/core/src/interactions/packet/acknowledgement.ts:45](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/interactions/packet/acknowledgement.ts#L45)

---

### subscribeToAcknowledgements

▸ **subscribeToAcknowledgements**(`subscribe`: _any_, `db`: HoprDB, `chain`: HoprCoreEthereum, `pubKey`: PeerId, `onMessage`: (`ackMessage`: [_Acknowledgement_](../classes/messages_acknowledgement.acknowledgement.md)) => _void_): _void_

#### Parameters

| Name        | Type                                                                                                  |
| :---------- | :---------------------------------------------------------------------------------------------------- |
| `subscribe` | _any_                                                                                                 |
| `db`        | HoprDB                                                                                                |
| `chain`     | HoprCoreEthereum                                                                                      |
| `pubKey`    | PeerId                                                                                                |
| `onMessage` | (`ackMessage`: [_Acknowledgement_](../classes/messages_acknowledgement.acknowledgement.md)) => _void_ |

**Returns:** _void_

Defined in: [packages/core/src/interactions/packet/acknowledgement.ts:12](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/interactions/packet/acknowledgement.ts#L12)
