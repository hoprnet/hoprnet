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

\+ **new UINT256**(`bn`: _BN_): [_UINT256_](uint256.md)

#### Parameters

| Name | Type |
| :--- | :--- |
| `bn` | _BN_ |

**Returns:** [_UINT256_](uint256.md)

Defined in: [types/solidity.ts:4](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L4)

## Accessors

### SIZE

• `Static` get **SIZE**(): _number_

**Returns:** _number_

Defined in: [types/solidity.ts:29](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L29)

## Methods

### serialize

▸ **serialize**(): _Uint8Array_

**Returns:** _Uint8Array_

Defined in: [types/solidity.ts:15](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L15)

---

### toBN

▸ **toBN**(): _BN_

**Returns:** _BN_

Defined in: [types/solidity.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L7)

---

### deserialize

▸ `Static` **deserialize**(`arr`: _Uint8Array_): [_UINT256_](uint256.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `arr` | _Uint8Array_ |

**Returns:** [_UINT256_](uint256.md)

Defined in: [types/solidity.ts:11](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L11)

---

### fromProbability

▸ `Static` **fromProbability**(`n`: _number_): [_UINT256_](uint256.md)

#### Parameters

| Name | Type     |
| :--- | :------- |
| `n`  | _number_ |

**Returns:** [_UINT256_](uint256.md)

Defined in: [types/solidity.ts:23](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L23)

---

### fromString

▸ `Static` **fromString**(`str`: _string_): [_UINT256_](uint256.md)

#### Parameters

| Name  | Type     |
| :---- | :------- |
| `str` | _string_ |

**Returns:** [_UINT256_](uint256.md)

Defined in: [types/solidity.ts:19](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/solidity.ts#L19)
