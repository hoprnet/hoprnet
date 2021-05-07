[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / [interactions/packet/forward](../modules/interactions_packet_forward.md) / PacketForwardInteraction

# Class: PacketForwardInteraction

[interactions/packet/forward](../modules/interactions_packet_forward.md).PacketForwardInteraction

## Table of contents

### Constructors

- [constructor](interactions_packet_forward.packetforwardinteraction.md#constructor)

### Properties

- [mixer](interactions_packet_forward.packetforwardinteraction.md#mixer)

### Methods

- [handleMixedPacket](interactions_packet_forward.packetforwardinteraction.md#handlemixedpacket)
- [handlePacket](interactions_packet_forward.packetforwardinteraction.md#handlepacket)
- [interact](interactions_packet_forward.packetforwardinteraction.md#interact)

## Constructors

### constructor

\+ **new PacketForwardInteraction**(`subscribe`: _any_, `sendMessage`: _any_, `privKey`: _PeerId_, `chain`: _default_, `emitMessage`: (`msg`: _Uint8Array_) => _void_, `db`: _HoprDB_): [_PacketForwardInteraction_](interactions_packet_forward.packetforwardinteraction.md)

#### Parameters

| Name          | Type                            |
| :------------ | :------------------------------ |
| `subscribe`   | _any_                           |
| `sendMessage` | _any_                           |
| `privKey`     | _PeerId_                        |
| `chain`       | _default_                       |
| `emitMessage` | (`msg`: _Uint8Array_) => _void_ |
| `db`          | _HoprDB_                        |

**Returns:** [_PacketForwardInteraction_](interactions_packet_forward.packetforwardinteraction.md)

Defined in: [packages/core/src/interactions/packet/forward.ts:12](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/interactions/packet/forward.ts#L12)

## Properties

### mixer

• `Private` **mixer**: [_Mixer_](mixer.mixer-1.md)

Defined in: [packages/core/src/interactions/packet/forward.ts:12](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/interactions/packet/forward.ts#L12)

## Methods

### handleMixedPacket

▸ **handleMixedPacket**(`packet`: [_Packet_](messages_packet.packet.md)): _Promise_<void\>

#### Parameters

| Name     | Type                                  |
| :------- | :------------------------------------ |
| `packet` | [_Packet_](messages_packet.packet.md) |

**Returns:** _Promise_<void\>

Defined in: [packages/core/src/interactions/packet/forward.ts:38](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/interactions/packet/forward.ts#L38)

---

### handlePacket

▸ **handlePacket**(`msg`: _Uint8Array_, `remotePeer`: _PeerId_): _Promise_<void\>

#### Parameters

| Name         | Type         |
| :----------- | :----------- |
| `msg`        | _Uint8Array_ |
| `remotePeer` | _PeerId_     |

**Returns:** _Promise_<void\>

Defined in: [packages/core/src/interactions/packet/forward.ts:32](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/interactions/packet/forward.ts#L32)

---

### interact

▸ **interact**(`counterparty`: _PeerId_, `packet`: [_Packet_](messages_packet.packet.md)): _Promise_<void\>

#### Parameters

| Name           | Type                                  |
| :------------- | :------------------------------------ |
| `counterparty` | _PeerId_                              |
| `packet`       | [_Packet_](messages_packet.packet.md) |

**Returns:** _Promise_<void\>

Defined in: [packages/core/src/interactions/packet/forward.ts:26](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/interactions/packet/forward.ts#L26)
