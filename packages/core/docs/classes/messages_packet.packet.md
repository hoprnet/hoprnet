[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / [messages/packet](../modules/messages_packet.md) / Packet

# Class: Packet

[messages/packet](../modules/messages_packet.md).Packet

## Table of contents

### Constructors

- [constructor](messages_packet.packet.md#constructor)

### Properties

- [ackChallenge](messages_packet.packet.md#ackchallenge)
- [isReadyToForward](messages_packet.packet.md#isreadytoforward)
- [isReceiver](messages_packet.packet.md#isreceiver)
- [nextChallenge](messages_packet.packet.md#nextchallenge)
- [nextHop](messages_packet.packet.md#nexthop)
- [ownKey](messages_packet.packet.md#ownkey)
- [ownShare](messages_packet.packet.md#ownshare)
- [packetTag](messages_packet.packet.md#packettag)
- [plaintext](messages_packet.packet.md#plaintext)
- [previousHop](messages_packet.packet.md#previoushop)

### Accessors

- [SIZE](messages_packet.packet.md#size)

### Methods

- [checkPacketTag](messages_packet.packet.md#checkpackettag)
- [createAcknowledgement](messages_packet.packet.md#createacknowledgement)
- [forwardTransform](messages_packet.packet.md#forwardtransform)
- [serialize](messages_packet.packet.md#serialize)
- [setFinal](messages_packet.packet.md#setfinal)
- [setForward](messages_packet.packet.md#setforward)
- [setReadyToForward](messages_packet.packet.md#setreadytoforward)
- [storeUnacknowledgedTicket](messages_packet.packet.md#storeunacknowledgedticket)
- [validateUnacknowledgedTicket](messages_packet.packet.md#validateunacknowledgedticket)
- [create](messages_packet.packet.md#create)
- [deserialize](messages_packet.packet.md#deserialize)

## Constructors

### constructor

\+ **new Packet**(`packet`: *Uint8Array*, `challenge`: [*Challenge*](messages_challenge.challenge.md), `ticket`: *Ticket*): [*Packet*](messages_packet.packet.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `packet` | *Uint8Array* |
| `challenge` | [*Challenge*](messages_challenge.challenge.md) |
| `ticket` | *Ticket* |

**Returns:** [*Packet*](messages_packet.packet.md)

Defined in: [packages/core/src/messages/packet.ts:166](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L166)

## Properties

### ackChallenge

• **ackChallenge**: *Uint8Array*

Defined in: [packages/core/src/messages/packet.ts:166](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L166)

___

### isReadyToForward

• **isReadyToForward**: *boolean*

Defined in: [packages/core/src/messages/packet.ts:156](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L156)

___

### isReceiver

• **isReceiver**: *boolean*

Defined in: [packages/core/src/messages/packet.ts:155](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L155)

___

### nextChallenge

• **nextChallenge**: *Uint8Array*

Defined in: [packages/core/src/messages/packet.ts:165](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L165)

___

### nextHop

• **nextHop**: *Uint8Array*

Defined in: [packages/core/src/messages/packet.ts:162](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L162)

___

### ownKey

• **ownKey**: *Uint8Array*

Defined in: [packages/core/src/messages/packet.ts:164](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L164)

___

### ownShare

• **ownShare**: *Uint8Array*

Defined in: [packages/core/src/messages/packet.ts:163](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L163)

___

### packetTag

• **packetTag**: *Uint8Array*

Defined in: [packages/core/src/messages/packet.ts:160](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L160)

___

### plaintext

• **plaintext**: *Uint8Array*

Defined in: [packages/core/src/messages/packet.ts:158](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L158)

___

### previousHop

• **previousHop**: *Uint8Array*

Defined in: [packages/core/src/messages/packet.ts:161](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L161)

## Accessors

### SIZE

• `Static` get **SIZE**(): *number*

**Returns:** *number*

Defined in: [packages/core/src/messages/packet.ts:249](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L249)

## Methods

### checkPacketTag

▸ **checkPacketTag**(`db`: *HoprDB*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `db` | *HoprDB* |

**Returns:** *Promise*<void\>

Defined in: [packages/core/src/messages/packet.ts:308](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L308)

___

### createAcknowledgement

▸ **createAcknowledgement**(`privKey`: *PeerId*): [*Acknowledgement*](messages_acknowledgement.acknowledgement.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `privKey` | *PeerId* |

**Returns:** [*Acknowledgement*](messages_acknowledgement.acknowledgement.md)

Defined in: [packages/core/src/messages/packet.ts:343](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L343)

___

### forwardTransform

▸ **forwardTransform**(`privKey`: *PeerId*, `chain`: *default*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `privKey` | *PeerId* |
| `chain` | *default* |

**Returns:** *Promise*<void\>

Defined in: [packages/core/src/messages/packet.ts:351](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L351)

___

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Defined in: [packages/core/src/messages/packet.ts:245](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L245)

___

### setFinal

▸ `Private` **setFinal**(`plaintext`: *Uint8Array*, `packetTag`: *Uint8Array*, `ownKey`: *Uint8Array*): [*Packet*](messages_packet.packet.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `plaintext` | *Uint8Array* |
| `packetTag` | *Uint8Array* |
| `ownKey` | *Uint8Array* |

**Returns:** [*Packet*](messages_packet.packet.md)

Defined in: [packages/core/src/messages/packet.ts:177](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L177)

___

### setForward

▸ `Private` **setForward**(`ownKey`: *Uint8Array*, `ownShare`: *Uint8Array*, `nextHop`: *Uint8Array*, `previousHop`: *Uint8Array*, `nextChallenge`: *Uint8Array*, `ackChallenge`: *Uint8Array*, `packetTag`: *Uint8Array*): [*Packet*](messages_packet.packet.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `ownKey` | *Uint8Array* |
| `ownShare` | *Uint8Array* |
| `nextHop` | *Uint8Array* |
| `previousHop` | *Uint8Array* |
| `nextChallenge` | *Uint8Array* |
| `ackChallenge` | *Uint8Array* |
| `packetTag` | *Uint8Array* |

**Returns:** [*Packet*](messages_packet.packet.md)

Defined in: [packages/core/src/messages/packet.ts:187](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L187)

___

### setReadyToForward

▸ `Private` **setReadyToForward**(`ackChallenge`: *Uint8Array*): [*Packet*](messages_packet.packet.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `ackChallenge` | *Uint8Array* |

**Returns:** [*Packet*](messages_packet.packet.md)

Defined in: [packages/core/src/messages/packet.ts:170](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L170)

___

### storeUnacknowledgedTicket

▸ **storeUnacknowledgedTicket**(`db`: *HoprDB*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `db` | *HoprDB* |

**Returns:** *Promise*<void\>

Defined in: [packages/core/src/messages/packet.ts:316](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L316)

___

### validateUnacknowledgedTicket

▸ **validateUnacknowledgedTicket**(`db`: *HoprDB*, `chain`: *default*, `privKey`: *PeerId*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `db` | *HoprDB* |
| `chain` | *default* |
| `privKey` | *PeerId* |

**Returns:** *Promise*<void\>

Defined in: [packages/core/src/messages/packet.ts:332](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L332)

___

### create

▸ `Static` **create**(`msg`: *Uint8Array*, `path`: *PeerId*[], `privKey`: *PeerId*, `chain`: *default*, `ticketOpts`: { `value`: *Balance* ; `winProb`: *number*  }): *Promise*<[*Packet*](messages_packet.packet.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `msg` | *Uint8Array* |
| `path` | *PeerId*[] |
| `privKey` | *PeerId* |
| `chain` | *default* |
| `ticketOpts` | *object* |
| `ticketOpts.value` | *Balance* |
| `ticketOpts.winProb` | *number* |

**Returns:** *Promise*<[*Packet*](messages_packet.packet.md)\>

Defined in: [packages/core/src/messages/packet.ts:210](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L210)

___

### deserialize

▸ `Static` **deserialize**(`preArray`: *Uint8Array*, `privKey`: *PeerId*, `pubKeySender`: *PeerId*): [*Packet*](messages_packet.packet.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `preArray` | *Uint8Array* |
| `privKey` | *PeerId* |
| `pubKeySender` | *PeerId* |

**Returns:** [*Packet*](messages_packet.packet.md)

Defined in: [packages/core/src/messages/packet.ts:253](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L253)
