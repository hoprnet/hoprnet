[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / [types/snapshot](../modules/types_snapshot.md) / Snapshot

# Class: Snapshot

[types/snapshot](../modules/types_snapshot.md).Snapshot

Represents a snapshot in the blockchain.

## Table of contents

### Constructors

- [constructor](types_snapshot.snapshot.md#constructor)

### Properties

- [blockNumber](types_snapshot.snapshot.md#blocknumber)
- [logIndex](types_snapshot.snapshot.md#logindex)
- [transactionIndex](types_snapshot.snapshot.md#transactionindex)

### Accessors

- [SIZE](types_snapshot.snapshot.md#size)

### Methods

- [serialize](types_snapshot.snapshot.md#serialize)
- [deserialize](types_snapshot.snapshot.md#deserialize)

## Constructors

### constructor

\+ **new Snapshot**(`blockNumber`: *BN*, `transactionIndex`: *BN*, `logIndex`: *BN*): [*Snapshot*](types_snapshot.snapshot.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `blockNumber` | *BN* |
| `transactionIndex` | *BN* |
| `logIndex` | *BN* |

**Returns:** [*Snapshot*](types_snapshot.snapshot.md)

Defined in: [types/snapshot.ts:8](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/snapshot.ts#L8)

## Properties

### blockNumber

• `Readonly` **blockNumber**: *BN*

___

### logIndex

• `Readonly` **logIndex**: *BN*

___

### transactionIndex

• `Readonly` **transactionIndex**: *BN*

## Accessors

### SIZE

• `Static` get **SIZE**(): *number*

**Returns:** *number*

Defined in: [types/snapshot.ts:28](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/snapshot.ts#L28)

## Methods

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Defined in: [types/snapshot.ts:20](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/snapshot.ts#L20)

___

### deserialize

▸ `Static` **deserialize**(`arr`: *Uint8Array*): [*Snapshot*](types_snapshot.snapshot.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*Snapshot*](types_snapshot.snapshot.md)

Defined in: [types/snapshot.ts:11](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/snapshot.ts#L11)
