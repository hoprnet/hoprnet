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

\+ `Private` **new PRP**(`iv`: *Uint8Array*, `key`: *Uint8Array*): [*PRP*](crypto_prp.prp.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `iv` | *Uint8Array* |
| `key` | *Uint8Array* |

**Returns:** [*PRP*](crypto_prp.prp.md)

Defined in: [crypto/prp.ts:30](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prp.ts#L30)

## Properties

### iv1

• `Private` `Readonly` **iv1**: *Uint8Array*

Defined in: [crypto/prp.ts:27](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prp.ts#L27)

___

### iv2

• `Private` `Readonly` **iv2**: *Uint8Array*

Defined in: [crypto/prp.ts:28](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prp.ts#L28)

___

### iv3

• `Private` `Readonly` **iv3**: *Uint8Array*

Defined in: [crypto/prp.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prp.ts#L29)

___

### iv4

• `Private` `Readonly` **iv4**: *Uint8Array*

Defined in: [crypto/prp.ts:30](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prp.ts#L30)

___

### k1

• `Private` `Readonly` **k1**: *Uint8Array*

Defined in: [crypto/prp.ts:22](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prp.ts#L22)

___

### k2

• `Private` `Readonly` **k2**: *Uint8Array*

Defined in: [crypto/prp.ts:23](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prp.ts#L23)

___

### k3

• `Private` `Readonly` **k3**: *Uint8Array*

Defined in: [crypto/prp.ts:24](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prp.ts#L24)

___

### k4

• `Private` `Readonly` **k4**: *Uint8Array*

Defined in: [crypto/prp.ts:25](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prp.ts#L25)

## Methods

### inverse

▸ **inverse**(`ciphertext`: *Uint8Array*): *Uint8Array*

#### Parameters

| Name | Type |
| :------ | :------ |
| `ciphertext` | *Uint8Array* |

**Returns:** *Uint8Array*

Defined in: [crypto/prp.ts:79](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prp.ts#L79)

___

### permutate

▸ **permutate**(`plaintext`: *Uint8Array*): *Uint8Array*

#### Parameters

| Name | Type |
| :------ | :------ |
| `plaintext` | *Uint8Array* |

**Returns:** *Uint8Array*

Defined in: [crypto/prp.ts:64](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prp.ts#L64)

___

### createPRP

▸ `Static` **createPRP**(`params`: [*PRPParameters*](../modules/crypto_prp.md#prpparameters)): [*PRP*](crypto_prp.prp.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `params` | [*PRPParameters*](../modules/crypto_prp.md#prpparameters) |

**Returns:** [*PRP*](crypto_prp.prp.md)

Defined in: [crypto/prp.ts:44](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/prp.ts#L44)
