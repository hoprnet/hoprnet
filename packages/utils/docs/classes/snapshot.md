[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Snapshot

# Class: Snapshot

Represents a snapshot in the blockchain.

## Table of contents

### Constructors

- [constructor](snapshot.md#constructor)

### Properties

- [blockNumber](snapshot.md#blocknumber)
- [logIndex](snapshot.md#logindex)
- [transactionIndex](snapshot.md#transactionindex)

### Accessors

- [SIZE](snapshot.md#size)

### Methods

- [serialize](snapshot.md#serialize)
- [deserialize](snapshot.md#deserialize)

## Constructors

### constructor

\+ **new Snapshot**(`blockNumber`: _BN_, `transactionIndex`: _BN_, `logIndex`: _BN_): [_Snapshot_](snapshot.md)

#### Parameters

| Name               | Type |
| :----------------- | :--- |
| `blockNumber`      | _BN_ |
| `transactionIndex` | _BN_ |
| `logIndex`         | _BN_ |

**Returns:** [_Snapshot_](snapshot.md)

Defined in: [types/snapshot.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/snapshot.ts#L8)

## Properties

### blockNumber

• `Readonly` **blockNumber**: _BN_

---

### logIndex

• `Readonly` **logIndex**: _BN_

---

### transactionIndex

• `Readonly` **transactionIndex**: _BN_

## Accessors

### SIZE

• `Static` get **SIZE**(): _number_

**Returns:** _number_

Defined in: [types/snapshot.ts:28](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/snapshot.ts#L28)

## Methods

### serialize

▸ **serialize**(): _Uint8Array_

**Returns:** _Uint8Array_

Defined in: [types/snapshot.ts:20](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/snapshot.ts#L20)

---

### deserialize

▸ `Static` **deserialize**(`arr`: _Uint8Array_): [_Snapshot_](snapshot.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `arr` | _Uint8Array_ |

**Returns:** [_Snapshot_](snapshot.md)

Defined in: [types/snapshot.ts:11](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/snapshot.ts#L11)
