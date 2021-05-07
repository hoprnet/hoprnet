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

- [WAIT\_TIME](mixer.mixer-1.md#wait_time)
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

\+ **new Mixer**(`onMessage`: (`m`: [*Packet*](messages_packet.packet.md)) => *void*, `clock?`: () => *number*): [*Mixer*](mixer.mixer-1.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `onMessage` | (`m`: [*Packet*](messages_packet.packet.md)) => *void* |
| `clock` | () => *number* |

**Returns:** [*Mixer*](mixer.mixer-1.md)

Defined in: [packages/core/src/mixer.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/mixer.ts#L29)

## Properties

### WAIT\_TIME

• **WAIT\_TIME**: *number*

Defined in: [packages/core/src/mixer.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/mixer.ts#L29)

___

### next

• `Private` **next**: *Timeout*

Defined in: [packages/core/src/mixer.ts:27](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/mixer.ts#L27)

___

### queue

• `Private` **queue**: *Heap*<HeapElement\>

Defined in: [packages/core/src/mixer.ts:26](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/mixer.ts#L26)

## Methods

### addTimeout

▸ `Private` **addTimeout**(): *void*

**Returns:** *void*

Defined in: [packages/core/src/mixer.ts:40](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/mixer.ts#L40)

___

### getPriority

▸ `Private` **getPriority**(): *number*

**Returns:** *number*

Defined in: [packages/core/src/mixer.ts:59](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/mixer.ts#L59)

___

### intervalUntilNextMessage

▸ `Private` **intervalUntilNextMessage**(): *number*

**Returns:** *number*

Defined in: [packages/core/src/mixer.ts:55](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/mixer.ts#L55)

___

### push

▸ **push**(`p`: [*Packet*](messages_packet.packet.md)): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `p` | [*Packet*](messages_packet.packet.md) |

**Returns:** *void*

Defined in: [packages/core/src/mixer.ts:35](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/mixer.ts#L35)

___

### tick

▸ `Private` **tick**(): *void*

**Returns:** *void*

Defined in: [packages/core/src/mixer.ts:46](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/mixer.ts#L46)
