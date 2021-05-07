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

\+ **new PacketForwardInteraction**(`subscribe`: *any*, `sendMessage`: *any*, `privKey`: *PeerId*, `chain`: *default*, `emitMessage`: (`msg`: *Uint8Array*) => *void*, `db`: *HoprDB*): [*PacketForwardInteraction*](interactions_packet_forward.packetforwardinteraction.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `subscribe` | *any* |
| `sendMessage` | *any* |
| `privKey` | *PeerId* |
| `chain` | *default* |
| `emitMessage` | (`msg`: *Uint8Array*) => *void* |
| `db` | *HoprDB* |

**Returns:** [*PacketForwardInteraction*](interactions_packet_forward.packetforwardinteraction.md)

Defined in: [packages/core/src/interactions/packet/forward.ts:12](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/interactions/packet/forward.ts#L12)

## Properties

### mixer

• `Private` **mixer**: [*Mixer*](mixer.mixer-1.md)

Defined in: [packages/core/src/interactions/packet/forward.ts:12](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/interactions/packet/forward.ts#L12)

## Methods

### handleMixedPacket

▸ **handleMixedPacket**(`packet`: [*Packet*](messages_packet.packet.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `packet` | [*Packet*](messages_packet.packet.md) |

**Returns:** *Promise*<void\>

Defined in: [packages/core/src/interactions/packet/forward.ts:38](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/interactions/packet/forward.ts#L38)

___

### handlePacket

▸ **handlePacket**(`msg`: *Uint8Array*, `remotePeer`: *PeerId*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `msg` | *Uint8Array* |
| `remotePeer` | *PeerId* |

**Returns:** *Promise*<void\>

Defined in: [packages/core/src/interactions/packet/forward.ts:32](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/interactions/packet/forward.ts#L32)

___

### interact

▸ **interact**(`counterparty`: *PeerId*, `packet`: [*Packet*](messages_packet.packet.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | *PeerId* |
| `packet` | [*Packet*](messages_packet.packet.md) |

**Returns:** *Promise*<void\>

Defined in: [packages/core/src/interactions/packet/forward.ts:26](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/interactions/packet/forward.ts#L26)
