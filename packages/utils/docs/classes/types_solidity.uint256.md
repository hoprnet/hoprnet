[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / [types/solidity](../modules/types_solidity.md) / UINT256

# Class: UINT256

[types/solidity](../modules/types_solidity.md).UINT256

## Table of contents

### Constructors

- [constructor](types_solidity.uint256.md#constructor)

### Accessors

- [SIZE](types_solidity.uint256.md#size)

### Methods

- [serialize](types_solidity.uint256.md#serialize)
- [toBN](types_solidity.uint256.md#tobn)
- [deserialize](types_solidity.uint256.md#deserialize)
- [fromProbability](types_solidity.uint256.md#fromprobability)
- [fromString](types_solidity.uint256.md#fromstring)

## Constructors

### constructor

\+ **new UINT256**(`bn`: *BN*): [*UINT256*](types_solidity.uint256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `bn` | *BN* |

**Returns:** [*UINT256*](types_solidity.uint256.md)

Defined in: [types/solidity.ts:4](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/solidity.ts#L4)

## Accessors

### SIZE

• `Static` get **SIZE**(): *number*

**Returns:** *number*

Defined in: [types/solidity.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/solidity.ts#L29)

## Methods

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Defined in: [types/solidity.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/solidity.ts#L15)

___

### toBN

▸ **toBN**(): *BN*

**Returns:** *BN*

Defined in: [types/solidity.ts:7](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/solidity.ts#L7)

___

### deserialize

▸ `Static` **deserialize**(`arr`: *Uint8Array*): [*UINT256*](types_solidity.uint256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*UINT256*](types_solidity.uint256.md)

Defined in: [types/solidity.ts:11](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/solidity.ts#L11)

___

### fromProbability

▸ `Static` **fromProbability**(`n`: *number*): [*UINT256*](types_solidity.uint256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `n` | *number* |

**Returns:** [*UINT256*](types_solidity.uint256.md)

Defined in: [types/solidity.ts:23](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/solidity.ts#L23)

___

### fromString

▸ `Static` **fromString**(`str`: *string*): [*UINT256*](types_solidity.uint256.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `str` | *string* |

**Returns:** [*UINT256*](types_solidity.uint256.md)

Defined in: [types/solidity.ts:19](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/solidity.ts#L19)
