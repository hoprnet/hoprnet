[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / UINT256

# Class: UINT256

## Table of contents

### Constructors

- [constructor](uint256.md#constructor)

### Accessors

- [DUMMY\_INVERSE\_PROBABILITY](uint256.md#dummy_inverse_probability)
- [SIZE](uint256.md#size)

### Methods

- [serialize](uint256.md#serialize)
- [toBN](uint256.md#tobn)
- [toHex](uint256.md#tohex)
- [deserialize](uint256.md#deserialize)
- [fromInverseProbability](uint256.md#frominverseprobability)
- [fromString](uint256.md#fromstring)

## Constructors

### constructor

• **new UINT256**(`bn`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `bn` | `BN` |

#### Defined in

[types/solidity.ts:3](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L3)

## Accessors

### DUMMY\_INVERSE\_PROBABILITY

• `Static` `get` **DUMMY_INVERSE_PROBABILITY**(): [`UINT256`](uint256.md)

#### Returns

[`UINT256`](uint256.md)

#### Defined in

[types/solidity.ts:34](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L34)

___

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/solidity.ts:38](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L38)

## Methods

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

▸ `Static` **deserialize**(`arr`): [`UINT256`](uint256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`UINT256`](uint256.md)

#### Defined in

[types/solidity.ts:10](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L10)

___

### fromInverseProbability

▸ `Static` **fromInverseProbability**(`inverseProb`): [`UINT256`](uint256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `inverseProb` | `BN` |

#### Returns

[`UINT256`](uint256.md)

#### Defined in

[types/solidity.ts:26](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L26)

___

### fromString

▸ `Static` **fromString**(`str`): [`UINT256`](uint256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `str` | `string` |

#### Returns

[`UINT256`](uint256.md)

#### Defined in

[types/solidity.ts:22](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L22)
