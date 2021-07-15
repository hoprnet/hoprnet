[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / EthereumChallenge

# Class: EthereumChallenge

## Table of contents

### Constructors

- [constructor](ethereumchallenge.md#constructor)

### Properties

- [SIZE](ethereumchallenge.md#size)

### Methods

- [eq](ethereumchallenge.md#eq)
- [serialize](ethereumchallenge.md#serialize)
- [toHex](ethereumchallenge.md#tohex)
- [deserialize](ethereumchallenge.md#deserialize)

## Constructors

### constructor

• **new EthereumChallenge**(`arr`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Defined in

[types/ethereumChallenge.ts:3](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ethereumChallenge.ts#L3)

## Properties

### SIZE

▪ `Static` **SIZE**: `number` = `20`

#### Defined in

[types/ethereumChallenge.ts:30](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ethereumChallenge.ts#L30)

## Methods

### eq

▸ **eq**(`ethCallenge`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `ethCallenge` | [`EthereumChallenge`](ethereumchallenge.md) |

#### Returns

`boolean`

#### Defined in

[types/ethereumChallenge.ts:26](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ethereumChallenge.ts#L26)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/ethereumChallenge.ts:18](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ethereumChallenge.ts#L18)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[types/ethereumChallenge.ts:22](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ethereumChallenge.ts#L22)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`EthereumChallenge`](ethereumchallenge.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`EthereumChallenge`](ethereumchallenge.md)

#### Defined in

[types/ethereumChallenge.ts:14](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ethereumChallenge.ts#L14)
