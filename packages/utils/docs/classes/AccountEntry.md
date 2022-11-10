[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / AccountEntry

# Class: AccountEntry

## Table of contents

### Constructors

- [constructor](AccountEntry.md#constructor)

### Properties

- [multiAddr](AccountEntry.md#multiaddr)
- [publicKey](AccountEntry.md#publickey)
- [updatedBlock](AccountEntry.md#updatedblock)

### Accessors

- [containsRouting](AccountEntry.md#containsrouting)
- [hasAnnounced](AccountEntry.md#hasannounced)
- [SIZE](AccountEntry.md#size)

### Methods

- [getAddress](AccountEntry.md#getaddress)
- [getPeerId](AccountEntry.md#getpeerid)
- [serialize](AccountEntry.md#serialize)
- [toString](AccountEntry.md#tostring)
- [deserialize](AccountEntry.md#deserialize)

## Constructors

### constructor

• **new AccountEntry**(`publicKey`, `multiAddr`, `updatedBlock`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `publicKey` | [`PublicKey`](PublicKey.md) |
| `multiAddr` | `Multiaddr` |
| `updatedBlock` | `BN` |

#### Defined in

[src/types/accountEntry.ts:16](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L16)

## Properties

### multiAddr

• `Readonly` **multiAddr**: `Multiaddr`

#### Defined in

[src/types/accountEntry.ts:18](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L18)

___

### publicKey

• `Readonly` **publicKey**: [`PublicKey`](PublicKey.md)

#### Defined in

[src/types/accountEntry.ts:17](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L17)

___

### updatedBlock

• `Readonly` **updatedBlock**: `BN`

#### Defined in

[src/types/accountEntry.ts:19](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L19)

## Accessors

### containsRouting

• `get` **containsRouting**(): `boolean`

#### Returns

`boolean`

#### Defined in

[src/types/accountEntry.ts:81](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L81)

___

### hasAnnounced

• `get` **hasAnnounced**(): `boolean`

#### Returns

`boolean`

#### Defined in

[src/types/accountEntry.ts:90](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L90)

___

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[src/types/accountEntry.ts:22](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L22)

## Methods

### getAddress

▸ **getAddress**(): [`Address`](Address.md)

#### Returns

[`Address`](Address.md)

#### Defined in

[src/types/accountEntry.ts:77](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L77)

___

### getPeerId

▸ **getPeerId**(): `PeerId`

#### Returns

`PeerId`

#### Defined in

[src/types/accountEntry.ts:73](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L73)

___

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[src/types/accountEntry.ts:45](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L45)

___

### toString

▸ **toString**(): `string`

#### Returns

`string`

#### Defined in

[src/types/accountEntry.ts:94](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L94)

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

[src/types/accountEntry.ts:26](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/accountEntry.ts#L26)
