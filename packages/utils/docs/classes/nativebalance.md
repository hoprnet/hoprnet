[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / NativeBalance

# Class: NativeBalance

## Table of contents

### Constructors

- [constructor](nativebalance.md#constructor)

### Accessors

- [DECIMALS](nativebalance.md#decimals)
- [SIZE](nativebalance.md#size)
- [SYMBOL](nativebalance.md#symbol)

### Methods

- [serialize](nativebalance.md#serialize)
- [toBN](nativebalance.md#tobn)
- [toFormattedString](nativebalance.md#toformattedstring)
- [deserialize](nativebalance.md#deserialize)

## Constructors

### constructor

\+ **new NativeBalance**(`bn`: _BN_): [_NativeBalance_](nativebalance.md)

#### Parameters

| Name | Type |
| :--- | :--- |
| `bn` | _BN_ |

**Returns:** [_NativeBalance_](nativebalance.md)

Defined in: [types/primitives.ts:225](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L225)

## Accessors

### DECIMALS

• `Static` get **DECIMALS**(): _number_

**Returns:** _number_

Defined in: [types/primitives.ts:232](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L232)

---

### SIZE

• `Static` get **SIZE**(): _number_

**Returns:** _number_

Defined in: [types/primitives.ts:252](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L252)

---

### SYMBOL

• `Static` get **SYMBOL**(): _string_

**Returns:** _string_

Defined in: [types/primitives.ts:228](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L228)

## Methods

### serialize

▸ **serialize**(): _Uint8Array_

**Returns:** _Uint8Array_

Defined in: [types/primitives.ts:244](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L244)

---

### toBN

▸ **toBN**(): _BN_

**Returns:** _BN_

Defined in: [types/primitives.ts:240](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L240)

---

### toFormattedString

▸ **toFormattedString**(): _string_

**Returns:** _string_

Defined in: [types/primitives.ts:248](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L248)

---

### deserialize

▸ `Static` **deserialize**(`arr`: _Uint8Array_): [_NativeBalance_](nativebalance.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `arr` | _Uint8Array_ |

**Returns:** [_NativeBalance_](nativebalance.md)

Defined in: [types/primitives.ts:236](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L236)
