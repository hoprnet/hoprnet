[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / AccountEntry

# Class: AccountEntry

## Table of contents

### Constructors

- [constructor](accountentry.md#constructor)

### Properties

- [address](accountentry.md#address)
- [multiAddr](accountentry.md#multiaddr)
- [updatedBlock](accountentry.md#updatedblock)

### Accessors

- [SIZE](accountentry.md#size)

### Methods

- [containsRouting](accountentry.md#containsrouting)
- [getPeerId](accountentry.md#getpeerid)
- [getPublicKey](accountentry.md#getpublickey)
- [hasAnnounced](accountentry.md#hasannounced)
- [serialize](accountentry.md#serialize)
- [deserialize](accountentry.md#deserialize)

## Constructors

### constructor

• **new AccountEntry**(`address`, `multiAddr`, `updatedBlock`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | [`Address`](address.md) |
| `multiAddr` | `Multiaddr` |
| `updatedBlock` | `BN` |

#### Defined in

[types/accountEntry.ts:10](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L10)

## Properties

### address

• `Readonly` **address**: [`Address`](address.md)

___

### multiAddr

• `Readonly` **multiAddr**: `Multiaddr`

___

### updatedBlock

• `Readonly` **updatedBlock**: `BN`

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/accountEntry.ts:35](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L35)

## Methods

### containsRouting

▸ **containsRouting**(): `boolean`

#### Returns

`boolean`

#### Defined in

[types/accountEntry.ts:71](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L71)

___

### getPeerId

▸ **getPeerId**(): `PeerId`

#### Returns

`PeerId`

#### Defined in

[types/accountEntry.ts:63](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L63)

___

### getPublicKey

▸ **getPublicKey**(): [`PublicKey`](publickey.md)

#### Returns

[`PublicKey`](publickey.md)

#### Defined in

[types/accountEntry.ts:67](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L67)

___

### hasAnnounced

▸ **hasAnnounced**(): `boolean`

#### Returns

`boolean`

#### Defined in

[types/accountEntry.ts:76](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L76)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/accountEntry.ts:50](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L50)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`AccountEntry`](accountentry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`AccountEntry`](accountentry.md)

#### Defined in

[types/accountEntry.ts:39](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L39)
