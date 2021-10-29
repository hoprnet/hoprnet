[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / UINT256

# Class: UINT256

## Table of contents

### Constructors

- [constructor](UINT256.md#constructor)

### Accessors

- [DUMMY\_INVERSE\_PROBABILITY](UINT256.md#dummy_inverse_probability)
- [SIZE](UINT256.md#size)

### Methods

- [eq](UINT256.md#eq)
- [serialize](UINT256.md#serialize)
- [toBN](UINT256.md#tobn)
- [toHex](UINT256.md#tohex)
- [deserialize](UINT256.md#deserialize)
- [fromInverseProbability](UINT256.md#frominverseprobability)
- [fromString](UINT256.md#fromstring)

## Constructors

### constructor

• **new UINT256**(`bn`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `bn` | `BN` |

#### Defined in

[types/solidity.ts:4](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L4)

## Accessors

### DUMMY\_INVERSE\_PROBABILITY

• `Static` `get` **DUMMY_INVERSE_PROBABILITY**(): [`UINT256`](UINT256.md)

#### Returns

[`UINT256`](UINT256.md)

#### Defined in

[types/solidity.ts:40](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L40)

___

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/solidity.ts:44](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L44)

## Methods

### eq

▸ **eq**(`b`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [`UINT256`](UINT256.md) |

#### Returns

`boolean`

#### Defined in

[types/solidity.ts:22](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L22)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/solidity.ts:14](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L14)

___

### toBN

▸ **toBN**(): `BN`

#### Returns

`BN`

#### Defined in

[types/solidity.ts:6](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L6)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[types/solidity.ts:18](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L18)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`UINT256`](UINT256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`UINT256`](UINT256.md)

#### Defined in

[types/solidity.ts:10](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L10)

___

### fromInverseProbability

▸ `Static` **fromInverseProbability**(`inverseProb`): [`UINT256`](UINT256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `inverseProb` | `BN` |

#### Returns

[`UINT256`](UINT256.md)

#### Defined in

[types/solidity.ts:30](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L30)

___

### fromString

▸ `Static` **fromString**(`str`): [`UINT256`](UINT256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `str` | `string` |

#### Returns

[`UINT256`](UINT256.md)

#### Defined in

[types/solidity.ts:26](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L26)
