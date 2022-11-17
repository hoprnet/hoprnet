[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / FIFO

# Interface: FIFO<T\>

## Type parameters

| Name |
| :------ |
| `T` |

## Table of contents

### Constructors

- [constructor](FIFO.md#constructor)

### Methods

- [last](FIFO.md#last)
- [peek](FIFO.md#peek)
- [push](FIFO.md#push)
- [replace](FIFO.md#replace)
- [shift](FIFO.md#shift)
- [size](FIFO.md#size)
- [toArray](FIFO.md#toarray)

## Constructors

### constructor

• **constructor**: `Object`

## Methods

### last

▸ **last**(): `T`

#### Returns

`T`

#### Defined in

[src/collection/fifo.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/collection/fifo.ts#L7)

___

### peek

▸ **peek**(): `T`

#### Returns

`T`

#### Defined in

[src/collection/fifo.ts:10](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/collection/fifo.ts#L10)

___

### push

▸ **push**(`item`): `number`

#### Parameters

| Name | Type |
| :------ | :------ |
| `item` | `T` |

#### Returns

`number`

#### Defined in

[src/collection/fifo.ts:12](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/collection/fifo.ts#L12)

___

### replace

▸ **replace**(`find`, `modify`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `find` | (`item`: `T`) => `boolean` |
| `modify` | (`oldItem`: `T`) => `T` |

#### Returns

`boolean`

#### Defined in

[src/collection/fifo.ts:11](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/collection/fifo.ts#L11)

___

### shift

▸ **shift**(): `T`

#### Returns

`T`

#### Defined in

[src/collection/fifo.ts:9](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/collection/fifo.ts#L9)

___

### size

▸ **size**(): `number`

#### Returns

`number`

#### Defined in

[src/collection/fifo.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/collection/fifo.ts#L8)

___

### toArray

▸ **toArray**(): `T`[]

#### Returns

`T`[]

#### Defined in

[src/collection/fifo.ts:13](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/collection/fifo.ts#L13)
