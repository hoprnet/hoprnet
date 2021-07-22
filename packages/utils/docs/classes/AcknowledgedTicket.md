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

#### Defined in

[types/acknowledgedTicket.ts:5](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L5)

## Properties

### preImage

• `Readonly` **preImage**: [`Hash`](Hash.md)

___

### response

• `Readonly` **response**: [`Response`](Response.md)

___

### signer

• `Readonly` **signer**: [`PublicKey`](PublicKey.md)

___

### ticket

• `Readonly` **ticket**: [`Ticket`](Ticket.md)

## Accessors

### SIZE

• `Static` `get` **SIZE**(): `number`

#### Returns

`number`

#### Defined in

[types/acknowledgedTicket.ts:39](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L39)

## Methods

### serialize

▸ **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

#### Defined in

[types/acknowledgedTicket.ts:16](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L16)

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

[types/acknowledgedTicket.ts:25](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L25)

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

[types/acknowledgedTicket.ts:29](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L29)
