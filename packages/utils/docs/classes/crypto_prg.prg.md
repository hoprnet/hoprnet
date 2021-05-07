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

\+ `Private` **new PRG**(`key`: *Uint8Array*, `iv`: *Uint8Array*): [*PRG*](crypto_prg.prg.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |
| `iv` | *Uint8Array* |

**Returns:** [*PRG*](crypto_prg.prg.md)

Defined in: [crypto/prg.ts:18](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prg.ts#L18)

## Properties

### iv

• `Private` `Readonly` **iv**: *Uint8Array*

Defined in: [crypto/prg.ts:18](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prg.ts#L18)

___

### key

• `Private` `Readonly` **key**: *Uint8Array*

Defined in: [crypto/prg.ts:17](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prg.ts#L17)

## Methods

### digest

▸ **digest**(`start`: *number*, `end`: *number*): *Uint8Array*

#### Parameters

| Name | Type |
| :------ | :------ |
| `start` | *number* |
| `end` | *number* |

**Returns:** *Uint8Array*

Defined in: [crypto/prg.ts:35](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prg.ts#L35)

___

### createPRG

▸ `Static` **createPRG**(`params`: [*PRGParameters*](../modules/crypto_prg.md#prgparameters)): [*PRG*](crypto_prg.prg.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `params` | [*PRGParameters*](../modules/crypto_prg.md#prgparameters) |

**Returns:** [*PRG*](crypto_prg.prg.md)

Defined in: [crypto/prg.ts:25](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prg.ts#L25)
