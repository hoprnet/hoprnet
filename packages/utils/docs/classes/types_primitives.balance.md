[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / [types/primitives](../modules/types_primitives.md) / Balance

# Class: Balance

[types/primitives](../modules/types_primitives.md).Balance

## Table of contents

### Constructors

- [constructor](types_primitives.balance.md#constructor)

### Accessors

- [DECIMALS](types_primitives.balance.md#decimals)
- [SIZE](types_primitives.balance.md#size)
- [SYMBOL](types_primitives.balance.md#symbol)

### Methods

- [serialize](types_primitives.balance.md#serialize)
- [toBN](types_primitives.balance.md#tobn)
- [toFormattedString](types_primitives.balance.md#toformattedstring)
- [deserialize](types_primitives.balance.md#deserialize)

## Constructors

### constructor

\+ **new Balance**(`bn`: *BN*): [*Balance*](types_primitives.balance.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `bn` | *BN* |

**Returns:** [*Balance*](types_primitives.balance.md)

Defined in: [types/primitives.ts:192](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L192)

## Accessors

### DECIMALS

• `Static` get **DECIMALS**(): *number*

**Returns:** *number*

Defined in: [types/primitives.ts:199](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L199)

___

### SIZE

• `Static` get **SIZE**(): *number*

**Returns:** *number*

Defined in: [types/primitives.ts:219](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L219)

___

### SYMBOL

• `Static` get **SYMBOL**(): *string*

**Returns:** *string*

Defined in: [types/primitives.ts:195](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L195)

## Methods

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Defined in: [types/primitives.ts:211](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L211)

___

### toBN

▸ **toBN**(): *BN*

**Returns:** *BN*

Defined in: [types/primitives.ts:203](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L203)

___

### toFormattedString

▸ **toFormattedString**(): *string*

**Returns:** *string*

Defined in: [types/primitives.ts:215](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L215)

___

### deserialize

▸ `Static` **deserialize**(`arr`: *Uint8Array*): [*Balance*](types_primitives.balance.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*Balance*](types_primitives.balance.md)

Defined in: [types/primitives.ts:207](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L207)
