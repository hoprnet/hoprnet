[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Address

# Class: Address

## Hierarchy

- **Address**

  ↳ [*EthereumChallenge*](ethereumchallenge.md)

## Table of contents

### Constructors

- [constructor](address.md#constructor)

### Accessors

- [SIZE](address.md#size)

### Methods

- [compare](address.md#compare)
- [eq](address.md#eq)
- [lt](address.md#lt)
- [serialize](address.md#serialize)
- [sortPair](address.md#sortpair)
- [toHex](address.md#tohex)
- [createMock](address.md#createmock)
- [deserialize](address.md#deserialize)
- [fromString](address.md#fromstring)

## Constructors

### constructor

\+ **new Address**(`arr`: *Uint8Array*): [*Address*](address.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*Address*](address.md)

Defined in: [types/primitives.ts:72](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L72)

## Accessors

### SIZE

• `Static` get **SIZE**(): *number*

**Returns:** *number*

Defined in: [types/primitives.ts:81](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L81)

## Methods

### compare

▸ **compare**(`b`: [*Address*](address.md)): *number*

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [*Address*](address.md) |

**Returns:** *number*

Defined in: [types/primitives.ts:105](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L105)

___

### eq

▸ **eq**(`b`: [*Address*](address.md)): *boolean*

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [*Address*](address.md) |

**Returns:** *boolean*

Defined in: [types/primitives.ts:101](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L101)

___

### lt

▸ **lt**(`b`: [*Address*](address.md)): *boolean*

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [*Address*](address.md) |

**Returns:** *boolean*

Defined in: [types/primitives.ts:109](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L109)

___

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Defined in: [types/primitives.ts:93](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L93)

___

### sortPair

▸ **sortPair**(`b`: [*Address*](address.md)): [[*Address*](address.md), [*Address*](address.md)]

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [*Address*](address.md) |

**Returns:** [[*Address*](address.md), [*Address*](address.md)]

Defined in: [types/primitives.ts:113](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L113)

___

### toHex

▸ **toHex**(): *string*

**Returns:** *string*

Defined in: [types/primitives.ts:97](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L97)

___

### createMock

▸ `Static` **createMock**(): [*Address*](address.md)

**Returns:** [*Address*](address.md)

Defined in: [types/primitives.ts:117](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L117)

___

### deserialize

▸ `Static` **deserialize**(`arr`: *Uint8Array*): [*Address*](address.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*Address*](address.md)

Defined in: [types/primitives.ts:89](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L89)

___

### fromString

▸ `Static` **fromString**(`str`: *string*): [*Address*](address.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `str` | *string* |

**Returns:** [*Address*](address.md)

Defined in: [types/primitives.ts:85](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L85)
