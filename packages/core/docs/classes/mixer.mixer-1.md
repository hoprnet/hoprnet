[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / [mixer](../modules/mixer.md) / Mixer

# Class: Mixer

[mixer](../modules/mixer.md).Mixer

Mix packets.

Currently an MVP version, that simply adds a random interval to their
priority.

## Table of contents

### Constructors

- [constructor](mixer.mixer-1.md#constructor)

### Properties

- [WAIT_TIME](mixer.mixer-1.md#wait_time)
- [next](mixer.mixer-1.md#next)
- [queue](mixer.mixer-1.md#queue)

### Methods

- [addTimeout](mixer.mixer-1.md#addtimeout)
- [getPriority](mixer.mixer-1.md#getpriority)
- [intervalUntilNextMessage](mixer.mixer-1.md#intervaluntilnextmessage)
- [push](mixer.mixer-1.md#push)
- [tick](mixer.mixer-1.md#tick)

## Constructors

### constructor

\+ **new Mixer**(`onMessage`: (`m`: [_Packet_](messages_packet.packet.md)) => _void_, `clock?`: () => _number_): [_Mixer_](mixer.mixer-1.md)

#### Parameters

| Name        | Type                                                   |
| :---------- | :----------------------------------------------------- |
| `onMessage` | (`m`: [_Packet_](messages_packet.packet.md)) => _void_ |
| `clock`     | () => _number_                                         |

**Returns:** [_Mixer_](mixer.mixer-1.md)

Defined in: [packages/core/src/mixer.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/mixer.ts#L29)

## Properties

### WAIT_TIME

• **WAIT_TIME**: _number_

Defined in: [packages/core/src/mixer.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/mixer.ts#L29)

---

### next

• `Private` **next**: _Timeout_

Defined in: [packages/core/src/mixer.ts:27](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/mixer.ts#L27)

---

### queue

• `Private` **queue**: _Heap_<HeapElement\>

Defined in: [packages/core/src/mixer.ts:26](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/mixer.ts#L26)

## Methods

### addTimeout

▸ `Private` **addTimeout**(): _void_

**Returns:** _void_

Defined in: [packages/core/src/mixer.ts:40](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/mixer.ts#L40)

---

### getPriority

▸ `Private` **getPriority**(): _number_

**Returns:** _number_

Defined in: [packages/core/src/mixer.ts:59](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/mixer.ts#L59)

---

### intervalUntilNextMessage

▸ `Private` **intervalUntilNextMessage**(): _number_

**Returns:** _number_

Defined in: [packages/core/src/mixer.ts:55](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/mixer.ts#L55)

---

### push

▸ **push**(`p`: [_Packet_](messages_packet.packet.md)): _void_

#### Parameters

| Name | Type                                  |
| :--- | :------------------------------------ |
| `p`  | [_Packet_](messages_packet.packet.md) |

**Returns:** _void_

Defined in: [packages/core/src/mixer.ts:35](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/mixer.ts#L35)

---

### tick

▸ `Private` **tick**(): _void_

**Returns:** _void_

Defined in: [packages/core/src/mixer.ts:46](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/mixer.ts#L46)
