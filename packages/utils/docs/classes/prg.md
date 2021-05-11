[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / PRG

# Class: PRG

## Table of contents

### Constructors

- [constructor](prg.md#constructor)

### Properties

- [iv](prg.md#iv)
- [key](prg.md#key)

### Methods

- [digest](prg.md#digest)
- [createPRG](prg.md#createprg)

## Constructors

### constructor

\+ `Private` **new PRG**(`key`: *Uint8Array*, `iv`: *Uint8Array*): [*PRG*](prg.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |
| `iv` | *Uint8Array* |

**Returns:** [*PRG*](prg.md)

Defined in: [crypto/prg.ts:18](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prg.ts#L18)

## Properties

### iv

• `Private` `Readonly` **iv**: *Uint8Array*

Defined in: [crypto/prg.ts:18](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prg.ts#L18)

___

### key

• `Private` `Readonly` **key**: *Uint8Array*

Defined in: [crypto/prg.ts:17](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prg.ts#L17)

## Methods

### digest

▸ **digest**(`start`: *number*, `end`: *number*): *Uint8Array*

#### Parameters

| Name | Type |
| :------ | :------ |
| `start` | *number* |
| `end` | *number* |

**Returns:** *Uint8Array*

Defined in: [crypto/prg.ts:35](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prg.ts#L35)

___

### createPRG

▸ `Static` **createPRG**(`params`: [*PRGParameters*](../modules.md#prgparameters)): [*PRG*](prg.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `params` | [*PRGParameters*](../modules.md#prgparameters) |

**Returns:** [*PRG*](prg.md)

Defined in: [crypto/prg.ts:25](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prg.ts#L25)
