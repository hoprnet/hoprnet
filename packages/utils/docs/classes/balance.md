[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Balance

# Class: Balance

## Table of contents

### Constructors

- [constructor](balance.md#constructor)

### Accessors

- [DECIMALS](balance.md#decimals)
- [SIZE](balance.md#size)
- [SYMBOL](balance.md#symbol)

### Methods

- [serialize](balance.md#serialize)
- [toBN](balance.md#tobn)
- [toFormattedString](balance.md#toformattedstring)
- [deserialize](balance.md#deserialize)

## Constructors

### constructor

\+ **new Balance**(`bn`: *BN*): [*Balance*](balance.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `bn` | *BN* |

**Returns:** [*Balance*](balance.md)

Defined in: [types/primitives.ts:196](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L196)

## Accessors

### DECIMALS

• `Static` get **DECIMALS**(): *number*

**Returns:** *number*

Defined in: [types/primitives.ts:203](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L203)

___

### SIZE

• `Static` get **SIZE**(): *number*

**Returns:** *number*

Defined in: [types/primitives.ts:223](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L223)

___

### SYMBOL

• `Static` get **SYMBOL**(): *string*

**Returns:** *string*

Defined in: [types/primitives.ts:199](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L199)

## Methods

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Defined in: [types/primitives.ts:215](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L215)

___

### toBN

▸ **toBN**(): *BN*

**Returns:** *BN*

Defined in: [types/primitives.ts:207](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L207)

___

### toFormattedString

▸ **toFormattedString**(): *string*

**Returns:** *string*

Defined in: [types/primitives.ts:219](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L219)

___

### deserialize

▸ `Static` **deserialize**(`arr`: *Uint8Array*): [*Balance*](balance.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*Balance*](balance.md)

Defined in: [types/primitives.ts:211](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L211)
