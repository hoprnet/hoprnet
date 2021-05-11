[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Commitment

# Class: Commitment

## Hierarchy

- [*Hash*](hash.md)

  ↳ **Commitment**

## Table of contents

### Constructors

- [constructor](commitment.md#constructor)

### Properties

- [arr](commitment.md#arr)
- [SIZE](commitment.md#size)

### Methods

- [clone](commitment.md#clone)
- [eq](commitment.md#eq)
- [hash](commitment.md#hash)
- [serialize](commitment.md#serialize)
- [toHex](commitment.md#tohex)
- [create](commitment.md#create)
- [deserialize](commitment.md#deserialize)

## Constructors

### constructor

\+ **new Commitment**(`arr`: *Uint8Array*): [*Commitment*](commitment.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*Commitment*](commitment.md)

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
