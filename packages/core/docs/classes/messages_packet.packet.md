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

\+ **new Packet**(`packet`: _Uint8Array_, `challenge`: [_Challenge_](messages_challenge.challenge.md), `ticket`: _Ticket_): [_Packet_](messages_packet.packet.md)

#### Parameters

| Name        | Type                                           |
| :---------- | :--------------------------------------------- |
| `packet`    | _Uint8Array_                                   |
| `challenge` | [_Challenge_](messages_challenge.challenge.md) |
| `ticket`    | _Ticket_                                       |

**Returns:** [_Packet_](messages_packet.packet.md)

Defined in: [packages/core/src/messages/packet.ts:166](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L166)

## Properties

### ackChallenge

• **ackChallenge**: _Uint8Array_

Defined in: [packages/core/src/messages/packet.ts:166](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L166)

---

### isReadyToForward

• **isReadyToForward**: _boolean_

Defined in: [packages/core/src/messages/packet.ts:156](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L156)

---

### isReceiver

• **isReceiver**: _boolean_

Defined in: [packages/core/src/messages/packet.ts:155](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L155)

---

### nextChallenge

• **nextChallenge**: _Uint8Array_

Defined in: [packages/core/src/messages/packet.ts:165](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L165)

---

### nextHop

• **nextHop**: _Uint8Array_

Defined in: [packages/core/src/messages/packet.ts:162](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L162)

---

### ownKey

• **ownKey**: _Uint8Array_

Defined in: [packages/core/src/messages/packet.ts:164](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L164)

---

### ownShare

• **ownShare**: _Uint8Array_

Defined in: [packages/core/src/messages/packet.ts:163](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L163)

---

### packetTag

• **packetTag**: _Uint8Array_

Defined in: [packages/core/src/messages/packet.ts:160](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L160)

---

### plaintext

• **plaintext**: _Uint8Array_

Defined in: [packages/core/src/messages/packet.ts:158](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L158)

---

### previousHop

• **previousHop**: _Uint8Array_

Defined in: [packages/core/src/messages/packet.ts:161](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L161)

## Accessors

### SIZE

• `Static` get **SIZE**(): _number_

**Returns:** _number_

Defined in: [packages/core/src/messages/packet.ts:249](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L249)

## Methods

### checkPacketTag

▸ **checkPacketTag**(`db`: _HoprDB_): _Promise_<void\>

#### Parameters

| Name | Type     |
| :--- | :------- |
| `db` | _HoprDB_ |

**Returns:** _Promise_<void\>

Defined in: [packages/core/src/messages/packet.ts:308](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L308)

---

### createAcknowledgement

▸ **createAcknowledgement**(`privKey`: _PeerId_): [_Acknowledgement_](messages_acknowledgement.acknowledgement.md)

#### Parameters

| Name      | Type     |
| :-------- | :------- |
| `privKey` | _PeerId_ |

**Returns:** [_Acknowledgement_](messages_acknowledgement.acknowledgement.md)

Defined in: [packages/core/src/messages/packet.ts:343](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L343)

---

### forwardTransform

▸ **forwardTransform**(`privKey`: _PeerId_, `chain`: _default_): _Promise_<void\>

#### Parameters

| Name      | Type      |
| :-------- | :-------- |
| `privKey` | _PeerId_  |
| `chain`   | _default_ |

**Returns:** _Promise_<void\>

Defined in: [packages/core/src/messages/packet.ts:351](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L351)

---

### serialize

▸ **serialize**(): _Uint8Array_

**Returns:** _Uint8Array_

Defined in: [packages/core/src/messages/packet.ts:245](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L245)

---

### setFinal

▸ `Private` **setFinal**(`plaintext`: _Uint8Array_, `packetTag`: _Uint8Array_, `ownKey`: _Uint8Array_): [_Packet_](messages_packet.packet.md)

#### Parameters

| Name        | Type         |
| :---------- | :----------- |
| `plaintext` | _Uint8Array_ |
| `packetTag` | _Uint8Array_ |
| `ownKey`    | _Uint8Array_ |

**Returns:** [_Packet_](messages_packet.packet.md)

Defined in: [packages/core/src/messages/packet.ts:177](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L177)

---

### setForward

▸ `Private` **setForward**(`ownKey`: _Uint8Array_, `ownShare`: _Uint8Array_, `nextHop`: _Uint8Array_, `previousHop`: _Uint8Array_, `nextChallenge`: _Uint8Array_, `ackChallenge`: _Uint8Array_, `packetTag`: _Uint8Array_): [_Packet_](messages_packet.packet.md)

#### Parameters

| Name            | Type         |
| :-------------- | :----------- |
| `ownKey`        | _Uint8Array_ |
| `ownShare`      | _Uint8Array_ |
| `nextHop`       | _Uint8Array_ |
| `previousHop`   | _Uint8Array_ |
| `nextChallenge` | _Uint8Array_ |
| `ackChallenge`  | _Uint8Array_ |
| `packetTag`     | _Uint8Array_ |

**Returns:** [_Packet_](messages_packet.packet.md)

Defined in: [packages/core/src/messages/packet.ts:187](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L187)

---

### setReadyToForward

▸ `Private` **setReadyToForward**(`ackChallenge`: _Uint8Array_): [_Packet_](messages_packet.packet.md)

#### Parameters

| Name           | Type         |
| :------------- | :----------- |
| `ackChallenge` | _Uint8Array_ |

**Returns:** [_Packet_](messages_packet.packet.md)

Defined in: [packages/core/src/messages/packet.ts:170](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L170)

---

### storeUnacknowledgedTicket

▸ **storeUnacknowledgedTicket**(`db`: _HoprDB_): _Promise_<void\>

#### Parameters

| Name | Type     |
| :--- | :------- |
| `db` | _HoprDB_ |

**Returns:** _Promise_<void\>

Defined in: [packages/core/src/messages/packet.ts:316](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L316)

---

### validateUnacknowledgedTicket

▸ **validateUnacknowledgedTicket**(`db`: _HoprDB_, `chain`: _default_, `privKey`: _PeerId_): _Promise_<void\>

#### Parameters

| Name      | Type      |
| :-------- | :-------- |
| `db`      | _HoprDB_  |
| `chain`   | _default_ |
| `privKey` | _PeerId_  |

**Returns:** _Promise_<void\>

Defined in: [packages/core/src/messages/packet.ts:332](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L332)

---

### create

▸ `Static` **create**(`msg`: _Uint8Array_, `path`: _PeerId_[], `privKey`: _PeerId_, `chain`: _default_, `ticketOpts`: { `value`: _Balance_ ; `winProb`: _number_ }): _Promise_<[_Packet_](messages_packet.packet.md)\>

#### Parameters

| Name                 | Type         |
| :------------------- | :----------- |
| `msg`                | _Uint8Array_ |
| `path`               | _PeerId_[]   |
| `privKey`            | _PeerId_     |
| `chain`              | _default_    |
| `ticketOpts`         | _object_     |
| `ticketOpts.value`   | _Balance_    |
| `ticketOpts.winProb` | _number_     |

**Returns:** _Promise_<[_Packet_](messages_packet.packet.md)\>

Defined in: [packages/core/src/messages/packet.ts:210](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L210)

---

### deserialize

▸ `Static` **deserialize**(`preArray`: _Uint8Array_, `privKey`: _PeerId_, `pubKeySender`: _PeerId_): [_Packet_](messages_packet.packet.md)

#### Parameters

| Name           | Type         |
| :------------- | :----------- |
| `preArray`     | _Uint8Array_ |
| `privKey`      | _PeerId_     |
| `pubKeySender` | _PeerId_     |

**Returns:** [_Packet_](messages_packet.packet.md)

Defined in: [packages/core/src/messages/packet.ts:253](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L253)
