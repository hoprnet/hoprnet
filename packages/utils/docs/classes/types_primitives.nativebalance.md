[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / [types/primitives](../modules/types_primitives.md) / NativeBalance

# Class: NativeBalance

[types/primitives](../modules/types_primitives.md).NativeBalance

## Table of contents

### Constructors

- [constructor](types_primitives.nativebalance.md#constructor)

### Accessors

- [DECIMALS](types_primitives.nativebalance.md#decimals)
- [SIZE](types_primitives.nativebalance.md#size)
- [SYMBOL](types_primitives.nativebalance.md#symbol)

### Methods

- [serialize](types_primitives.nativebalance.md#serialize)
- [toBN](types_primitives.nativebalance.md#tobn)
- [toFormattedString](types_primitives.nativebalance.md#toformattedstring)
- [deserialize](types_primitives.nativebalance.md#deserialize)

## Constructors

### constructor

\+ **new NativeBalance**(`bn`: *BN*): [*NativeBalance*](types_primitives.nativebalance.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `bn` | *BN* |

**Returns:** [*NativeBalance*](types_primitives.nativebalance.md)

Defined in: [types/primitives.ts:225](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L225)

## Accessors

### DECIMALS

• `Static` get **DECIMALS**(): *number*

**Returns:** *number*

Defined in: [types/primitives.ts:232](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L232)

___

### SIZE

• `Static` get **SIZE**(): *number*

**Returns:** *number*

Defined in: [types/primitives.ts:252](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L252)

___

### SYMBOL

• `Static` get **SYMBOL**(): *string*

**Returns:** *string*

Defined in: [types/primitives.ts:228](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L228)

## Methods

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Defined in: [types/primitives.ts:244](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L244)

___

### toBN

▸ **toBN**(): *BN*

**Returns:** *BN*

Defined in: [types/primitives.ts:240](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L240)

___

### toFormattedString

▸ **toFormattedString**(): *string*

**Returns:** *string*

Defined in: [types/primitives.ts:248](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L248)

___

### deserialize

▸ `Static` **deserialize**(`arr`: *Uint8Array*): [*NativeBalance*](types_primitives.nativebalance.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*NativeBalance*](types_primitives.nativebalance.md)

Defined in: [types/primitives.ts:236](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L236)
