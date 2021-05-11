[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Signature

# Class: Signature

## Table of contents

### Constructors

- [constructor](signature.md#constructor)

### Properties

- [recovery](signature.md#recovery)
- [signature](signature.md#signature)
- [SIZE](signature.md#size)

### Methods

- [serialize](signature.md#serialize)
- [verify](signature.md#verify)
- [create](signature.md#create)
- [deserialize](signature.md#deserialize)

## Constructors

### constructor

\+ **new Signature**(`signature`: *Uint8Array*, `recovery`: *number*): [*Signature*](signature.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signature` | *Uint8Array* |
| `recovery` | *number* |

**Returns:** [*Signature*](signature.md)

Defined in: [types/primitives.ts:169](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L169)

## Properties

### recovery

• `Readonly` **recovery**: *number*

___

### signature

• `Readonly` **signature**: *Uint8Array*

___

### SIZE

▪ `Static` **SIZE**: *number*

Defined in: [types/primitives.ts:197](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L197)

## Methods

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Defined in: [types/primitives.ts:186](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L186)

___

### verify

▸ **verify**(`msg`: *Uint8Array*, `pubKey`: [*PublicKey*](publickey.md)): *boolean*

#### Parameters

| Name | Type |
| :------ | :------ |
| `msg` | *Uint8Array* |
| `pubKey` | [*PublicKey*](publickey.md) |

**Returns:** *boolean*

Defined in: [types/primitives.ts:193](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L193)

___

### create

▸ `Static` **create**(`msg`: *Uint8Array*, `privKey`: *Uint8Array*): [*Signature*](signature.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `msg` | *Uint8Array* |
| `privKey` | *Uint8Array* |

**Returns:** [*Signature*](signature.md)

Defined in: [types/primitives.ts:181](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L181)

___

### deserialize

▸ `Static` **deserialize**(`arr`: *Uint8Array*): [*Signature*](signature.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*Signature*](signature.md)

Defined in: [types/primitives.ts:176](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L176)
