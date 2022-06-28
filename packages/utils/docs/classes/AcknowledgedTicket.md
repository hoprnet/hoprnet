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

## Properties

### preImage

• `Readonly` **preImage**: [`Hash`](Hash.md)

#### Defined in

[types/acknowledgedTicket.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L8)

___

### response

• `Readonly` **response**: [`Response`](Response.md)

#### Defined in

[types/acknowledgedTicket.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L7)

___

### signer

• `Readonly` **signer**: [`PublicKey`](PublicKey.md)

#### Defined in

[types/acknowledgedTicket.ts:9](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L9)

___

### ticket

• `Readonly` **ticket**: [`Ticket`](Ticket.md)

#### Defined in

[types/acknowledgedTicket.ts:6](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L6)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

## Methods

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

___

### verify

▸ **verify**(`ticketIssuer`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `ticketIssuer` | [`PublicKey`](PublicKey.md) |

#### Returns

`boolean`

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`AcknowledgedTicket`](AcknowledgedTicket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`AcknowledgedTicket`](AcknowledgedTicket.md)
