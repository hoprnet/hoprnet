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

\+ **new UINT256**(`bn`: _BN_): [_UINT256_](types_solidity.uint256.md)

#### Parameters

| Name | Type |
| :--- | :--- |
| `bn` | _BN_ |

**Returns:** [_UINT256_](types_solidity.uint256.md)

Defined in: [types/solidity.ts:4](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/solidity.ts#L4)

## Accessors

### SIZE

• `Static` get **SIZE**(): _number_

**Returns:** _number_

Defined in: [types/solidity.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/solidity.ts#L29)

## Methods

### serialize

▸ **serialize**(): _Uint8Array_

**Returns:** _Uint8Array_

Defined in: [types/solidity.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/solidity.ts#L15)

---

### toBN

▸ **toBN**(): _BN_

**Returns:** _BN_

Defined in: [types/solidity.ts:7](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/solidity.ts#L7)

---

### deserialize

▸ `Static` **deserialize**(`arr`: _Uint8Array_): [_UINT256_](types_solidity.uint256.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `arr` | _Uint8Array_ |

**Returns:** [_UINT256_](types_solidity.uint256.md)

Defined in: [types/solidity.ts:11](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/solidity.ts#L11)

---

### fromProbability

▸ `Static` **fromProbability**(`n`: _number_): [_UINT256_](types_solidity.uint256.md)

#### Parameters

| Name | Type     |
| :--- | :------- |
| `n`  | _number_ |

**Returns:** [_UINT256_](types_solidity.uint256.md)

Defined in: [types/solidity.ts:23](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/solidity.ts#L23)

---

### fromString

▸ `Static` **fromString**(`str`: _string_): [_UINT256_](types_solidity.uint256.md)

#### Parameters

| Name  | Type     |
| :---- | :------- |
| `str` | _string_ |

**Returns:** [_UINT256_](types_solidity.uint256.md)

Defined in: [types/solidity.ts:19](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/solidity.ts#L19)
