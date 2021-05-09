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

\+ **new Balance**(`bn`: _BN_): [_Balance_](balance.md)

#### Parameters

| Name | Type |
| :--- | :--- |
| `bn` | _BN_ |

**Returns:** [_Balance_](balance.md)

Defined in: [types/primitives.ts:192](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L192)

## Accessors

### DECIMALS

• `Static` get **DECIMALS**(): _number_

**Returns:** _number_

Defined in: [types/primitives.ts:199](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L199)

---

### SIZE

• `Static` get **SIZE**(): _number_

**Returns:** _number_

Defined in: [types/primitives.ts:219](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L219)

---

### SYMBOL

• `Static` get **SYMBOL**(): _string_

**Returns:** _string_

Defined in: [types/primitives.ts:195](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L195)

## Methods

### serialize

▸ **serialize**(): _Uint8Array_

**Returns:** _Uint8Array_

Defined in: [types/primitives.ts:211](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L211)

---

### toBN

▸ **toBN**(): _BN_

**Returns:** _BN_

Defined in: [types/primitives.ts:203](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L203)

---

### toFormattedString

▸ **toFormattedString**(): _string_

**Returns:** _string_

Defined in: [types/primitives.ts:215](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L215)

---

### deserialize

▸ `Static` **deserialize**(`arr`: _Uint8Array_): [_Balance_](balance.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `arr` | _Uint8Array_ |

**Returns:** [_Balance_](balance.md)

Defined in: [types/primitives.ts:207](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L207)
