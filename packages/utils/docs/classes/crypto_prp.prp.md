[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / [crypto/prp](../modules/crypto_prp.md) / PRP

# Class: PRP

[crypto/prp](../modules/crypto_prp.md).PRP

## Table of contents

### Constructors

- [constructor](crypto_prp.prp.md#constructor)

### Properties

- [iv1](crypto_prp.prp.md#iv1)
- [iv2](crypto_prp.prp.md#iv2)
- [iv3](crypto_prp.prp.md#iv3)
- [iv4](crypto_prp.prp.md#iv4)
- [k1](crypto_prp.prp.md#k1)
- [k2](crypto_prp.prp.md#k2)
- [k3](crypto_prp.prp.md#k3)
- [k4](crypto_prp.prp.md#k4)

### Methods

- [inverse](crypto_prp.prp.md#inverse)
- [permutate](crypto_prp.prp.md#permutate)
- [createPRP](crypto_prp.prp.md#createprp)

## Constructors

### constructor

\+ `Private` **new PRP**(`iv`: _Uint8Array_, `key`: _Uint8Array_): [_PRP_](crypto_prp.prp.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `iv`  | _Uint8Array_ |
| `key` | _Uint8Array_ |

**Returns:** [_PRP_](crypto_prp.prp.md)

Defined in: [crypto/prp.ts:30](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prp.ts#L30)

## Properties

### iv1

• `Private` `Readonly` **iv1**: _Uint8Array_

Defined in: [crypto/prp.ts:27](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prp.ts#L27)

---

### iv2

• `Private` `Readonly` **iv2**: _Uint8Array_

Defined in: [crypto/prp.ts:28](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prp.ts#L28)

---

### iv3

• `Private` `Readonly` **iv3**: _Uint8Array_

Defined in: [crypto/prp.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prp.ts#L29)

---

### iv4

• `Private` `Readonly` **iv4**: _Uint8Array_

Defined in: [crypto/prp.ts:30](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prp.ts#L30)

---

### k1

• `Private` `Readonly` **k1**: _Uint8Array_

Defined in: [crypto/prp.ts:22](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prp.ts#L22)

---

### k2

• `Private` `Readonly` **k2**: _Uint8Array_

Defined in: [crypto/prp.ts:23](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prp.ts#L23)

---

### k3

• `Private` `Readonly` **k3**: _Uint8Array_

Defined in: [crypto/prp.ts:24](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prp.ts#L24)

---

### k4

• `Private` `Readonly` **k4**: _Uint8Array_

Defined in: [crypto/prp.ts:25](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prp.ts#L25)

## Methods

### inverse

▸ **inverse**(`ciphertext`: _Uint8Array_): _Uint8Array_

#### Parameters

| Name         | Type         |
| :----------- | :----------- |
| `ciphertext` | _Uint8Array_ |

**Returns:** _Uint8Array_

Defined in: [crypto/prp.ts:79](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prp.ts#L79)

---

### permutate

▸ **permutate**(`plaintext`: _Uint8Array_): _Uint8Array_

#### Parameters

| Name        | Type         |
| :---------- | :----------- |
| `plaintext` | _Uint8Array_ |

**Returns:** _Uint8Array_

Defined in: [crypto/prp.ts:64](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prp.ts#L64)

---

### createPRP

▸ `Static` **createPRP**(`params`: [_PRPParameters_](../modules/crypto_prp.md#prpparameters)): [_PRP_](crypto_prp.prp.md)

#### Parameters

| Name     | Type                                                      |
| :------- | :-------------------------------------------------------- |
| `params` | [_PRPParameters_](../modules/crypto_prp.md#prpparameters) |

**Returns:** [_PRP_](crypto_prp.prp.md)

Defined in: [crypto/prp.ts:44](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prp.ts#L44)
