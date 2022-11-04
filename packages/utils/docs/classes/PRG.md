[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / PRG

# Class: PRG

## Table of contents

### Constructors

- [constructor](PRG.md#constructor)

### Properties

- [iv](PRG.md#iv)
- [key](PRG.md#key)

### Methods

- [digest](PRG.md#digest)
- [createPRG](PRG.md#createprg)

## Constructors

### constructor

• `Private` **new PRG**(`key`, `iv`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | `Uint8Array` |
| `iv` | `Uint8Array` |

## Properties

### iv

• `Private` `Readonly` **iv**: `Uint8Array`

#### Defined in

[src/crypto/prg.ts:18](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prg.ts#L18)

___

### key

• `Private` `Readonly` **key**: `Uint8Array`

#### Defined in

[src/crypto/prg.ts:17](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prg.ts#L17)

## Methods

### digest

▸ **digest**(`start`, `end`): `Uint8Array`

#### Parameters

| Name | Type |
| :------ | :------ |
| `start` | `number` |
| `end` | `number` |

#### Returns

`Uint8Array`

___

### createPRG

▸ `Static` **createPRG**(`params`): [`PRG`](PRG.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `params` | [`PRGParameters`](../modules.md#prgparameters) |

#### Returns

[`PRG`](PRG.md)
