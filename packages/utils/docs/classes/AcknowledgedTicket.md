[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / AcknowledgedTicket

# Class: AcknowledgedTicket

## Table of contents

### Constructors

- [constructor](AcknowledgedTicket.md#constructor)

### Properties

- [preImage](AcknowledgedTicket.md#preimage)
- [response](AcknowledgedTicket.md#response)
- [signer](AcknowledgedTicket.md#signer)
- [ticket](AcknowledgedTicket.md#ticket)

### Accessors

- [SIZE](AcknowledgedTicket.md#size)

### Methods

- [serialize](AcknowledgedTicket.md#serialize)
- [setPreImage](AcknowledgedTicket.md#setpreimage)
- [verify](AcknowledgedTicket.md#verify)
- [deserialize](AcknowledgedTicket.md#deserialize)

## Constructors

### constructor

• **new AcknowledgedTicket**(`ticket`, `response`, `preImage`, `signer`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `ticket` | [`Ticket`](Ticket.md) |
| `response` | [`Response`](Response.md) |
| `preImage` | [`Hash`](Hash.md) |
| `signer` | [`PublicKey`](PublicKey.md) |

#### Defined in

[utils/src/types/acknowledgedTicket.ts:5](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L5)

## Properties

### preImage

• **preImage**: [`Hash`](Hash.md)

#### Defined in

[utils/src/types/acknowledgedTicket.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L8)

___

### response

• `Readonly` **response**: [`Response`](Response.md)

#### Defined in

[utils/src/types/acknowledgedTicket.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L7)

___

### signer

• `Readonly` **signer**: [`PublicKey`](PublicKey.md)

#### Defined in

[utils/src/types/acknowledgedTicket.ts:9](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L9)

___

### ticket

• `Readonly` **ticket**: [`Ticket`](Ticket.md)

#### Defined in

[utils/src/types/acknowledgedTicket.ts:6](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L6)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[utils/src/types/acknowledgedTicket.ts:45](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L45)

## Methods

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[utils/src/types/acknowledgedTicket.ts:20](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L20)

___

### setPreImage

▸ **setPreImage**(`preImg`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `preImg` | [`Hash`](Hash.md) |

#### Returns

`void`

#### Defined in

[utils/src/types/acknowledgedTicket.ts:16](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L16)

___

### verify

▸ **verify**(`ticketIssuer`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `ticketIssuer` | [`PublicKey`](PublicKey.md) |

#### Returns

`boolean`

#### Defined in

[utils/src/types/acknowledgedTicket.ts:29](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L29)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`AcknowledgedTicket`](AcknowledgedTicket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`AcknowledgedTicket`](AcknowledgedTicket.md)

#### Defined in

[utils/src/types/acknowledgedTicket.ts:35](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L35)
