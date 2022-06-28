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

___

### peek

▸ **peek**(): `T`

#### Returns

`T`

___

### push

▸ **push**(`item`): `number`

#### Parameters

| Name | Type |
| :------ | :------ |
| `item` | `T` |

#### Returns

`number`

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

___

### shift

▸ **shift**(): `T`

#### Returns

`T`

___

### size

▸ **size**(): `number`

#### Returns

`number`

___

### toArray

▸ **toArray**(): `T`[]

#### Returns

`T`[]
