[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / [crypto/prg](../modules/crypto_prg.md) / PRG

# Class: PRG

[crypto/prg](../modules/crypto_prg.md).PRG

## Table of contents

### Constructors

- [constructor](crypto_prg.prg.md#constructor)

### Properties

- [iv](crypto_prg.prg.md#iv)
- [key](crypto_prg.prg.md#key)

### Methods

- [digest](crypto_prg.prg.md#digest)
- [createPRG](crypto_prg.prg.md#createprg)

## Constructors

### constructor

\+ `Private` **new PRG**(`key`: _Uint8Array_, `iv`: _Uint8Array_): [_PRG_](crypto_prg.prg.md)

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `key` | _Uint8Array_ |
| `iv`  | _Uint8Array_ |

**Returns:** [_PRG_](crypto_prg.prg.md)

Defined in: [crypto/prg.ts:18](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prg.ts#L18)

## Properties

### iv

• `Private` `Readonly` **iv**: _Uint8Array_

Defined in: [crypto/prg.ts:18](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prg.ts#L18)

---

### key

• `Private` `Readonly` **key**: _Uint8Array_

Defined in: [crypto/prg.ts:17](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prg.ts#L17)

## Methods

### digest

▸ **digest**(`start`: _number_, `end`: _number_): _Uint8Array_

#### Parameters

| Name    | Type     |
| :------ | :------- |
| `start` | _number_ |
| `end`   | _number_ |

**Returns:** _Uint8Array_

Defined in: [crypto/prg.ts:35](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prg.ts#L35)

---

### createPRG

▸ `Static` **createPRG**(`params`: [_PRGParameters_](../modules/crypto_prg.md#prgparameters)): [_PRG_](crypto_prg.prg.md)

#### Parameters

| Name     | Type                                                      |
| :------- | :-------------------------------------------------------- |
| `params` | [_PRGParameters_](../modules/crypto_prg.md#prgparameters) |

**Returns:** [_PRG_](crypto_prg.prg.md)

Defined in: [crypto/prg.ts:25](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prg.ts#L25)
