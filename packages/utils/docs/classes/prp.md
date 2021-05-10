[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / PRP

# Class: PRP

## Table of contents

### Constructors

- [constructor](prp.md#constructor)

### Properties

- [iv1](prp.md#iv1)
- [iv2](prp.md#iv2)
- [iv3](prp.md#iv3)
- [iv4](prp.md#iv4)
- [k1](prp.md#k1)
- [k2](prp.md#k2)
- [k3](prp.md#k3)
- [k4](prp.md#k4)

### Methods

- [inverse](prp.md#inverse)
- [permutate](prp.md#permutate)
- [createPRP](prp.md#createprp)

## Constructors

### constructor

\+ `Private` **new PRP**(`iv`: _Uint8Array_, `key`: _Uint8Array_): [_PRP_](prp.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `iv`  | _Uint8Array_ |
| `key` | _Uint8Array_ |

**Returns:** [_PRP_](prp.md)

Defined in: [crypto/prp.ts:30](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L30)

## Properties

### iv1

• `Private` `Readonly` **iv1**: _Uint8Array_

Defined in: [crypto/prp.ts:27](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L27)

---

### iv2

• `Private` `Readonly` **iv2**: _Uint8Array_

Defined in: [crypto/prp.ts:28](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L28)

---

### iv3

• `Private` `Readonly` **iv3**: _Uint8Array_

Defined in: [crypto/prp.ts:29](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L29)

---

### iv4

• `Private` `Readonly` **iv4**: _Uint8Array_

Defined in: [crypto/prp.ts:30](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L30)

---

### k1

• `Private` `Readonly` **k1**: _Uint8Array_

Defined in: [crypto/prp.ts:22](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L22)

---

### k2

• `Private` `Readonly` **k2**: _Uint8Array_

Defined in: [crypto/prp.ts:23](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L23)

---

### k3

• `Private` `Readonly` **k3**: _Uint8Array_

Defined in: [crypto/prp.ts:24](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L24)

---

### k4

• `Private` `Readonly` **k4**: _Uint8Array_

Defined in: [crypto/prp.ts:25](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L25)

## Methods

### inverse

▸ **inverse**(`ciphertext`: _Uint8Array_): _Uint8Array_

#### Parameters

| Name         | Type         |
| :----------- | :----------- |
| `ciphertext` | _Uint8Array_ |

**Returns:** _Uint8Array_

Defined in: [crypto/prp.ts:79](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L79)

---

### permutate

▸ **permutate**(`plaintext`: _Uint8Array_): _Uint8Array_

#### Parameters

| Name        | Type         |
| :---------- | :----------- |
| `plaintext` | _Uint8Array_ |

**Returns:** _Uint8Array_

Defined in: [crypto/prp.ts:64](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L64)

---

### createPRP

▸ `Static` **createPRP**(`params`: [_PRPParameters_](../modules.md#prpparameters)): [_PRP_](prp.md)

#### Parameters

| Name     | Type                                           |
| :------- | :--------------------------------------------- |
| `params` | [_PRPParameters_](../modules.md#prpparameters) |

**Returns:** [_PRP_](prp.md)

Defined in: [crypto/prp.ts:44](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L44)
