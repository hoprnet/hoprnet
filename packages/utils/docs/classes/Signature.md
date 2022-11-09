[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / Signature

# Class: Signature

Class used to represent an ECDSA signature.

The methods serialize()/deserialize() are used to convert the signature
to/from 64-byte compressed representation as given by EIP-2098 (https://eips.ethereum.org/EIPS/eip-2098).
This compressed signature format is supported by OpenZeppelin.

Internally this class still maintains representation using `(r,s)` tuple and `v` parity component separate
as this makes interop with the underlying ECDSA library simpler.

## Table of contents

### Constructors

- [constructor](Signature.md#constructor)

### Properties

- [recovery](Signature.md#recovery)
- [signature](Signature.md#signature)
- [SIZE](Signature.md#size)

### Methods

- [serialize](Signature.md#serialize)
- [toHex](Signature.md#tohex)
- [verify](Signature.md#verify)
- [create](Signature.md#create)
- [deserialize](Signature.md#deserialize)

## Constructors

### constructor

• **new Signature**(`signature`, `recovery`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signature` | `Uint8Array` |
| `recovery` | `number` |

## Properties

### recovery

• `Readonly` **recovery**: `number`

#### Defined in

[types/primitives.ts:119](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L119)

___

### signature

• `Readonly` **signature**: `Uint8Array`

#### Defined in

[types/primitives.ts:119](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L119)

___

### SIZE

▪ `Static` **SIZE**: `number` = `SIGNATURE_LENGTH`

#### Defined in

[types/primitives.ts:162](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L162)

## Methods

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

___

### verify

▸ **verify**(`msg`, `pubKey`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `msg` | `Uint8Array` |
| `pubKey` | [`PublicKey`](PublicKey.md) |

#### Returns

`boolean`

___

### create

▸ `Static` **create**(`msg`, `privKey`): [`Signature`](Signature.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `msg` | `Uint8Array` |
| `privKey` | `Uint8Array` |

#### Returns

[`Signature`](Signature.md)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`Signature`](Signature.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`Signature`](Signature.md)
