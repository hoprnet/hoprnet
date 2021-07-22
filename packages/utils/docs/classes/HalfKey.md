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

#### Defined in

[types/halfKey.ts:6](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKey.ts#L6)

## Properties

### SIZE

▪ `Static` **SIZE**: `number` = `32`

#### Defined in

[types/halfKey.ts:41](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKey.ts#L41)

## Methods

### clone

▸ **clone**(): [`HalfKey`](HalfKey.md)

#### Returns

[`HalfKey`](HalfKey.md)

#### Defined in

[types/halfKey.ts:36](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKey.ts#L36)

___

### eq

▸ **eq**(`halfKey`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `halfKey` | [`HalfKey`](HalfKey.md) |

#### Returns

`boolean`

#### Defined in

[types/halfKey.ts:28](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKey.ts#L28)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/halfKey.ts:20](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKey.ts#L20)

___

### toChallenge

▸ **toChallenge**(): [`HalfKeyChallenge`](HalfKeyChallenge.md)

#### Returns

[`HalfKeyChallenge`](HalfKeyChallenge.md)

#### Defined in

[types/halfKey.ts:16](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKey.ts#L16)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[types/halfKey.ts:24](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKey.ts#L24)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`HalfKey`](HalfKey.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`HalfKey`](HalfKey.md)

#### Defined in

[types/halfKey.ts:32](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/halfKey.ts#L32)
