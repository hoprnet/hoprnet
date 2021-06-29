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
- [serializeEthereum](signature.md#serializeethereum)
- [toHex](signature.md#tohex)
- [verify](signature.md#verify)
- [create](signature.md#create)
- [deserialize](signature.md#deserialize)
- [deserializeEthereum](signature.md#deserializeethereum)

## Constructors

### constructor

• **new Signature**(`signature`, `recovery`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signature` | `Uint8Array` |
| `recovery` | `number` |

#### Defined in

[types/primitives.ts:180](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L180)

## Properties

### recovery

• `Readonly` **recovery**: `number`

___

### signature

• `Readonly` **signature**: `Uint8Array`

___

### SIZE

▪ `Static` **SIZE**: `number`

#### Defined in

[types/primitives.ts:247](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L247)

## Methods

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/primitives.ts:221](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L221)

___

### serializeEthereum

▸ **serializeEthereum**(): `Uint8Array`

Replaces recovery value by Ethereum-specific values 27/28

#### Returns

`Uint8Array`

serialized signature to use within Ethereum

#### Defined in

[types/primitives.ts:232](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L232)

___

### toHex

▸ **toHex**(): `string`

#### Returns

`string`

#### Defined in

[types/primitives.ts:243](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L243)

___

### verify

▸ **verify**(`msg`, `pubKey`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `msg` | `Uint8Array` |
| `pubKey` | [`PublicKey`](publickey.md) |

#### Returns

`boolean`

#### Defined in

[types/primitives.ts:239](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L239)

___

### create

▸ `Static` **create**(`msg`, `privKey`): [`Signature`](signature.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `msg` | `Uint8Array` |
| `privKey` | `Uint8Array` |

#### Returns

[`Signature`](signature.md)

#### Defined in

[types/primitives.ts:216](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L216)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`Signature`](signature.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`Signature`](signature.md)

#### Defined in

[types/primitives.ts:187](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L187)

___

### deserializeEthereum

▸ `Static` **deserializeEthereum**(`arr`): [`Signature`](signature.md)

Deserializes Ethereum-specific signature with
non-standard recovery values 27 and 28

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `arr` | `Uint8Array` | serialized Ethereum signature |

#### Returns

[`Signature`](signature.md)

deserialized Ethereum signature

#### Defined in

[types/primitives.ts:205](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/primitives.ts#L205)
