[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / EthereumChallenge

# Class: EthereumChallenge

## Table of contents

### Constructors

- [constructor](EthereumChallenge.md#constructor)

### Properties

- [arr](EthereumChallenge.md#arr)
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

#### Defined in

[src/types/ethereumChallenge.ts:4](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ethereumChallenge.ts#L4)

## Properties

### arr

• `Private` `Readonly` **arr**: `Uint8Array`

#### Defined in

[src/types/ethereumChallenge.ts:4](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ethereumChallenge.ts#L4)

___

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

#### Defined in

[src/types/ethereumChallenge.ts:26](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ethereumChallenge.ts#L26)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[src/types/ethereumChallenge.ts:18](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ethereumChallenge.ts#L18)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[src/types/ethereumChallenge.ts:22](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ethereumChallenge.ts#L22)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`EthereumChallenge`](EthereumChallenge.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`EthereumChallenge`](EthereumChallenge.md)

#### Defined in

[src/types/ethereumChallenge.ts:14](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/ethereumChallenge.ts#L14)
