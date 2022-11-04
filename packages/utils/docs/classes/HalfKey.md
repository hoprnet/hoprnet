[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / HalfKey

# Class: HalfKey

## Table of contents

### Constructors

- [constructor](HalfKey.md#constructor)

### Properties

- [SIZE](HalfKey.md#size)

### Methods

- [clone](HalfKey.md#clone)
- [eq](HalfKey.md#eq)
- [serialize](HalfKey.md#serialize)
- [toChallenge](HalfKey.md#tochallenge)
- [toHex](HalfKey.md#tohex)
- [deserialize](HalfKey.md#deserialize)

## Constructors

### constructor

• **new HalfKey**(`arr`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

## Properties

### SIZE

▪ `Static` **SIZE**: `number` = `32`

#### Defined in

[src/types/halfKey.ts:41](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKey.ts#L41)

## Methods

### clone

▸ **clone**(): [`HalfKey`](HalfKey.md)

#### Returns

[`HalfKey`](HalfKey.md)

___

### eq

▸ **eq**(`halfKey`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `halfKey` | [`HalfKey`](HalfKey.md) |

#### Returns

`boolean`

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

___

### toChallenge

▸ **toChallenge**(): [`HalfKeyChallenge`](HalfKeyChallenge.md)

#### Returns

[`HalfKeyChallenge`](HalfKeyChallenge.md)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`HalfKey`](HalfKey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`HalfKey`](HalfKey.md)
