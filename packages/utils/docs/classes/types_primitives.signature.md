[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / [types/primitives](../modules/types_primitives.md) / Signature

# Class: Signature

[types/primitives](../modules/types_primitives.md).Signature

## Table of contents

### Constructors

- [constructor](types_primitives.signature.md#constructor)

### Properties

- [recovery](types_primitives.signature.md#recovery)
- [signature](types_primitives.signature.md#signature)
- [SIZE](types_primitives.signature.md#size)

### Methods

- [serialize](types_primitives.signature.md#serialize)
- [verify](types_primitives.signature.md#verify)
- [create](types_primitives.signature.md#create)
- [deserialize](types_primitives.signature.md#deserialize)

## Constructors

### constructor

\+ **new Signature**(`signature`: *Uint8Array*, `recovery`: *number*): [*Signature*](types_primitives.signature.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signature` | *Uint8Array* |
| `recovery` | *number* |

**Returns:** [*Signature*](types_primitives.signature.md)

Defined in: [types/primitives.ts:161](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L161)

## Properties

### recovery

• `Readonly` **recovery**: *number*

___

### signature

• `Readonly` **signature**: *Uint8Array*

___

### SIZE

▪ `Static` **SIZE**: *number*

Defined in: [types/primitives.ts:189](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L189)

## Methods

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Defined in: [types/primitives.ts:178](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L178)

___

### verify

▸ **verify**(`msg`: *Uint8Array*, `pubKey`: [*PublicKey*](types_primitives.publickey.md)): *boolean*

#### Parameters

| Name | Type |
| :------ | :------ |
| `msg` | *Uint8Array* |
| `pubKey` | [*PublicKey*](types_primitives.publickey.md) |

**Returns:** *boolean*

Defined in: [types/primitives.ts:185](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L185)

___

### create

▸ `Static` **create**(`msg`: *Uint8Array*, `privKey`: *Uint8Array*): [*Signature*](types_primitives.signature.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `msg` | *Uint8Array* |
| `privKey` | *Uint8Array* |

**Returns:** [*Signature*](types_primitives.signature.md)

Defined in: [types/primitives.ts:173](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L173)

___

### deserialize

▸ `Static` **deserialize**(`arr`: *Uint8Array*): [*Signature*](types_primitives.signature.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | *Uint8Array* |

**Returns:** [*Signature*](types_primitives.signature.md)

Defined in: [types/primitives.ts:168](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/primitives.ts#L168)
