[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Response

# Class: Response

## Hierarchy

- [*Hash*](hash.md)

  ↳ **Response**

## Table of contents

### Constructors

- [constructor](response.md#constructor)

### Properties

- [arr](response.md#arr)
- [SIZE](response.md#size)

### Methods

- [clone](response.md#clone)
- [eq](response.md#eq)
- [hash](response.md#hash)
- [serialize](response.md#serialize)
- [toChallenge](response.md#tochallenge)
- [toHex](response.md#tohex)
- [create](response.md#create)
- [deserialize](response.md#deserialize)
- [fromHalfKeys](response.md#fromhalfkeys)

## Constructors

### constructor

\+ **new Response**(`arr`: *Uint8Array*): [*Response*](response.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*Response*](response.md)

Inherited from: [Hash](hash.md)

Defined in: [types/primitives.ts:123](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L123)

## Properties

### arr

• `Protected` **arr**: *Uint8Array*

Inherited from: [Hash](hash.md).[arr](hash.md#arr)

Defined in: [types/primitives.ts:123](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L123)

___

### SIZE

▪ `Static` **SIZE**: *number*

Inherited from: [Hash](hash.md).[SIZE](hash.md#size)

Defined in: [types/primitives.ts:137](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L137)

## Methods

### clone

▸ **clone**(): [*Hash*](hash.md)

**Returns:** [*Hash*](hash.md)

Inherited from: [Hash](hash.md)

Defined in: [types/primitives.ts:159](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L159)

___

### eq

▸ **eq**(`b`: [*Hash*](hash.md)): *boolean*

#### Parameters

| Name | Type |
| :------ | :------ |
| `b` | [*Hash*](hash.md) |

**Returns:** *boolean*

Inherited from: [Hash](hash.md)

Defined in: [types/primitives.ts:151](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L151)

___

### hash

▸ **hash**(): [*Hash*](hash.md)

**Returns:** [*Hash*](hash.md)

Inherited from: [Hash](hash.md)

Defined in: [types/primitives.ts:163](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L163)

___

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Inherited from: [Hash](hash.md)

Defined in: [types/primitives.ts:147](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L147)

___

### toChallenge

▸ **toChallenge**(): [*Challenge*](challenge.md)

**Returns:** [*Challenge*](challenge.md)

Defined in: [types/response.ts:12](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/response.ts#L12)

___

### toHex

▸ **toHex**(): *string*

**Returns:** *string*

Inherited from: [Hash](hash.md)

Defined in: [types/primitives.ts:155](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L155)

___

### create

▸ `Static` **create**(...`inputs`: *Uint8Array*[]): [*Hash*](hash.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `...inputs` | *Uint8Array*[] |

**Returns:** [*Hash*](hash.md)

Inherited from: [Hash](hash.md)

Defined in: [types/primitives.ts:139](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L139)

___

### deserialize

▸ `Static` **deserialize**(`arr`: *Uint8Array*): [*Hash*](hash.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*Hash*](hash.md)

Inherited from: [Hash](hash.md)

Defined in: [types/primitives.ts:143](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L143)

___

### fromHalfKeys

▸ `Static` **fromHalfKeys**(`firstHalfKey`: [*HalfKey*](halfkey.md), `secondHalfKey`: [*HalfKey*](halfkey.md)): [*Response*](response.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `firstHalfKey` | [*HalfKey*](halfkey.md) |
| `secondHalfKey` | [*HalfKey*](halfkey.md) |

**Returns:** [*Response*](response.md)

Defined in: [types/response.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/response.ts#L8)
