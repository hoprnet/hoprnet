[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / PRP

# Class: PRP

## Table of contents

### Constructors

- [constructor](PRP.md#constructor)

### Properties

- [iv1](PRP.md#iv1)
- [iv2](PRP.md#iv2)
- [iv3](PRP.md#iv3)
- [iv4](PRP.md#iv4)
- [k1](PRP.md#k1)
- [k2](PRP.md#k2)
- [k3](PRP.md#k3)
- [k4](PRP.md#k4)

### Methods

- [inverse](PRP.md#inverse)
- [permutate](PRP.md#permutate)
- [createPRP](PRP.md#createprp)

## Constructors

### constructor

• `Private` **new PRP**(`iv`, `key`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `iv` | `Uint8Array` |
| `key` | `Uint8Array` |

#### Defined in

[utils/src/crypto/prp.ts:32](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L32)

## Properties

### iv1

• `Private` `Readonly` **iv1**: `Uint8Array`

#### Defined in

[utils/src/crypto/prp.ts:27](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L27)

___

### iv2

• `Private` `Readonly` **iv2**: `Uint8Array`

#### Defined in

[utils/src/crypto/prp.ts:28](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L28)

___

### iv3

• `Private` `Readonly` **iv3**: `Uint8Array`

#### Defined in

[utils/src/crypto/prp.ts:29](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L29)

___

### iv4

• `Private` `Readonly` **iv4**: `Uint8Array`

#### Defined in

[utils/src/crypto/prp.ts:30](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L30)

___

### k1

• `Private` `Readonly` **k1**: `Uint8Array`

#### Defined in

[utils/src/crypto/prp.ts:22](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L22)

___

### k2

• `Private` `Readonly` **k2**: `Uint8Array`

#### Defined in

[utils/src/crypto/prp.ts:23](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L23)

___

### k3

• `Private` `Readonly` **k3**: `Uint8Array`

#### Defined in

[utils/src/crypto/prp.ts:24](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L24)

___

### k4

• `Private` `Readonly` **k4**: `Uint8Array`

#### Defined in

[utils/src/crypto/prp.ts:25](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L25)

## Methods

### inverse

▸ **inverse**(`ciphertext`): `Uint8Array`

#### Parameters

| Name | Type |
| :------ | :------ |
| `ciphertext` | `Uint8Array` |

#### Returns

`Uint8Array`

#### Defined in

[utils/src/crypto/prp.ts:79](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L79)

___

### permutate

▸ **permutate**(`plaintext`): `Uint8Array`

#### Parameters

| Name | Type |
| :------ | :------ |
| `plaintext` | `Uint8Array` |

#### Returns

`Uint8Array`

#### Defined in

[utils/src/crypto/prp.ts:64](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L64)

___

### createPRP

▸ `Static` **createPRP**(`params`): [`PRP`](PRP.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `params` | [`PRPParameters`](../modules.md#prpparameters) |

#### Returns

[`PRP`](PRP.md)

#### Defined in

[utils/src/crypto/prp.ts:44](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L44)
