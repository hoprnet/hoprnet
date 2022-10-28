[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Snapshot

# Class: Snapshot

Represents a snapshot in the blockchain.

## Table of contents

### Constructors

- [constructor](Snapshot.md#constructor)

### Properties

- [blockNumber](Snapshot.md#blocknumber)
- [logIndex](Snapshot.md#logindex)
- [transactionIndex](Snapshot.md#transactionindex)

### Accessors

- [SIZE](Snapshot.md#size)

### Methods

- [serialize](Snapshot.md#serialize)
- [deserialize](Snapshot.md#deserialize)

## Constructors

### constructor

• **new Snapshot**(`blockNumber`, `transactionIndex`, `logIndex`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `blockNumber` | `BN` |
| `transactionIndex` | `BN` |
| `logIndex` | `BN` |

## Properties

### blockNumber

• `Readonly` **blockNumber**: `BN`

#### Defined in

[src/types/snapshot.ts:9](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/snapshot.ts#L9)

___

### logIndex

• `Readonly` **logIndex**: `BN`

#### Defined in

[src/types/snapshot.ts:9](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/snapshot.ts#L9)

___

### transactionIndex

• `Readonly` **transactionIndex**: `BN`

#### Defined in

[src/types/snapshot.ts:9](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/snapshot.ts#L9)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

## Methods

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`Snapshot`](Snapshot.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`Snapshot`](Snapshot.md)
