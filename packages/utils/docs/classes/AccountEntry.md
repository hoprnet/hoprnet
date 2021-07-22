[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / AccountEntry

# Class: AccountEntry

## Table of contents

### Constructors

- [constructor](AccountEntry.md#constructor)

### Properties

- [address](AccountEntry.md#address)
- [multiAddr](AccountEntry.md#multiaddr)
- [updatedBlock](AccountEntry.md#updatedblock)

### Accessors

- [SIZE](AccountEntry.md#size)

### Methods

- [containsRouting](AccountEntry.md#containsrouting)
- [getPeerId](AccountEntry.md#getpeerid)
- [getPublicKey](AccountEntry.md#getpublickey)
- [hasAnnounced](AccountEntry.md#hasannounced)
- [serialize](AccountEntry.md#serialize)
- [deserialize](AccountEntry.md#deserialize)

## Constructors

### constructor

• **new AccountEntry**(`address`, `multiAddr`, `updatedBlock`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | [`Address`](Address.md) |
| `multiAddr` | `Multiaddr` |
| `updatedBlock` | `BN` |

#### Defined in

[types/accountEntry.ts:11](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L11)

## Properties

### address

• `Readonly` **address**: [`Address`](Address.md)

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

▸ **getPublicKey**(): [`PublicKey`](PublicKey.md)

#### Returns

[`PublicKey`](PublicKey.md)

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

▸ `Static` **deserialize**(`arr`): [`AccountEntry`](AccountEntry.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`AccountEntry`](AccountEntry.md)

#### Defined in

[types/accountEntry.ts:39](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L39)
