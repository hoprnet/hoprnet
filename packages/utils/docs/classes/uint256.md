[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / UINT256

# Class: UINT256

## Table of contents

### Constructors

- [constructor](uint256.md#constructor)

### Accessors

- [SIZE](uint256.md#size)

### Methods

- [serialize](uint256.md#serialize)
- [toBN](uint256.md#tobn)
- [deserialize](uint256.md#deserialize)
- [fromProbability](uint256.md#fromprobability)
- [fromString](uint256.md#fromstring)

## Constructors

### constructor

\+ **new UINT256**(`bn`: *BN*): [*UINT256*](uint256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `bn` | *BN* |

**Returns:** [*UINT256*](uint256.md)

Defined in: [types/solidity.ts:4](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L4)

## Accessors

### SIZE

• `Static` get **SIZE**(): *number*

**Returns:** *number*

Defined in: [types/solidity.ts:29](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L29)

## Methods

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Defined in: [types/solidity.ts:15](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L15)

___

### toBN

▸ **toBN**(): *BN*

**Returns:** *BN*

Defined in: [types/solidity.ts:7](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L7)

___

### deserialize

▸ `Static` **deserialize**(`arr`: *Uint8Array*): [*UINT256*](uint256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*UINT256*](uint256.md)

Defined in: [types/solidity.ts:11](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L11)

___

### fromProbability

▸ `Static` **fromProbability**(`n`: *number*): [*UINT256*](uint256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `n` | *number* |

**Returns:** [*UINT256*](uint256.md)

Defined in: [types/solidity.ts:23](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L23)

___

### fromString

▸ `Static` **fromString**(`str`: *string*): [*UINT256*](uint256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `str` | *string* |

**Returns:** [*UINT256*](uint256.md)

Defined in: [types/solidity.ts:19](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L19)
