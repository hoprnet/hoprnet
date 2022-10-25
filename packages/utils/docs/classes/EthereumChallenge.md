[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / EthereumChallenge

# Class: EthereumChallenge

## Table of contents

### Constructors

- [constructor](EthereumChallenge.md#constructor)

### Properties

- [SIZE](EthereumChallenge.md#size)

### Methods

- [eq](EthereumChallenge.md#eq)
- [serialize](EthereumChallenge.md#serialize)
- [toHex](EthereumChallenge.md#tohex)
- [deserialize](EthereumChallenge.md#deserialize)

## Constructors

### constructor

• **new EthereumChallenge**(`arr`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

## Properties

### SIZE

▪ `Static` **SIZE**: `number` = `20`

#### Defined in

[src/types/ethereumChallenge.ts:30](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ethereumChallenge.ts#L30)

## Methods

### eq

▸ **eq**(`ethCallenge`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `ethCallenge` | [`EthereumChallenge`](EthereumChallenge.md) |

#### Returns

`boolean`

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`EthereumChallenge`](EthereumChallenge.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`EthereumChallenge`](EthereumChallenge.md)
