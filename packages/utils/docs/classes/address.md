[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Address

# Class: Address

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

\+ **new Address**(`arr`: _Uint8Array_): [_Address_](address.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `arr` | _Uint8Array_ |

**Returns:** [_Address_](address.md)

Defined in: [types/primitives.ts:72](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L72)

## Accessors

### SIZE

• `Static` get **SIZE**(): _number_

**Returns:** _number_

Defined in: [types/primitives.ts:81](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L81)

## Methods

### compare

▸ **compare**(`b`: [_Address_](address.md)): _number_

#### Parameters

| Name | Type                    |
| :--- | :---------------------- |
| `b`  | [_Address_](address.md) |

**Returns:** _number_

Defined in: [types/primitives.ts:105](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L105)

---

### eq

▸ **eq**(`b`: [_Address_](address.md)): _boolean_

#### Parameters

| Name | Type                    |
| :--- | :---------------------- |
| `b`  | [_Address_](address.md) |

**Returns:** _boolean_

Defined in: [types/primitives.ts:101](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L101)

---

### lt

▸ **lt**(`b`: [_Address_](address.md)): _boolean_

#### Parameters

| Name | Type                    |
| :--- | :---------------------- |
| `b`  | [_Address_](address.md) |

**Returns:** _boolean_

Defined in: [types/primitives.ts:109](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L109)

---

### serialize

▸ **serialize**(): _Uint8Array_

**Returns:** _Uint8Array_

Defined in: [types/primitives.ts:93](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L93)

---

### sortPair

▸ **sortPair**(`b`: [_Address_](address.md)): [[_Address_](address.md), [_Address_](address.md)]

#### Parameters

| Name | Type                    |
| :--- | :---------------------- |
| `b`  | [_Address_](address.md) |

**Returns:** [[_Address_](address.md), [_Address_](address.md)]

Defined in: [types/primitives.ts:113](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L113)

---

### toHex

▸ **toHex**(): _string_

**Returns:** _string_

Defined in: [types/primitives.ts:97](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L97)

---

### createMock

▸ `Static` **createMock**(): [_Address_](address.md)

**Returns:** [_Address_](address.md)

Defined in: [types/primitives.ts:117](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L117)

---

### deserialize

▸ `Static` **deserialize**(`arr`: _Uint8Array_): [_Address_](address.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `arr` | _Uint8Array_ |

**Returns:** [_Address_](address.md)

Defined in: [types/primitives.ts:89](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L89)

---

### fromString

▸ `Static` **fromString**(`str`: _string_): [_Address_](address.md)

#### Parameters

| Name  | Type     |
| :---- | :------- |
| `str` | _string_ |

**Returns:** [_Address_](address.md)

Defined in: [types/primitives.ts:85](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L85)
