[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / AcknowledgedTicket

# Class: AcknowledgedTicket

## Table of contents

### Constructors

- [constructor](acknowledgedticket.md#constructor)

### Properties

- [preImage](acknowledgedticket.md#preimage)
- [response](acknowledgedticket.md#response)
- [signer](acknowledgedticket.md#signer)
- [ticket](acknowledgedticket.md#ticket)

### Accessors

- [SIZE](acknowledgedticket.md#size)

### Methods

- [serialize](acknowledgedticket.md#serialize)
- [verify](acknowledgedticket.md#verify)
- [deserialize](acknowledgedticket.md#deserialize)

## Constructors

### constructor

• **new AcknowledgedTicket**(`ticket`, `response`, `preImage`, `signer`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `ticket` | [`Ticket`](ticket.md) |
| `response` | [`Response`](response.md) |
| `preImage` | [`Hash`](hash.md) |
| `signer` | [`PublicKey`](publickey.md) |

#### Defined in

[types/acknowledgedTicket.ts:4](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L4)

## Properties

### preImage

• `Readonly` **preImage**: [`Hash`](hash.md)

___

### response

• `Readonly` **response**: [`Response`](response.md)

___

### signer

• `Readonly` **signer**: [`PublicKey`](publickey.md)

___

### ticket

• `Readonly` **ticket**: [`Ticket`](ticket.md)

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
| `ticketIssuer` | [`PublicKey`](publickey.md) |

#### Returns

`boolean`

#### Defined in

[types/acknowledgedTicket.ts:25](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L25)

___

### deserialize

▸ `Static` **deserialize**(`arr`): [`AcknowledgedTicket`](acknowledgedticket.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `arr` | `Uint8Array` |

#### Returns

[`AcknowledgedTicket`](acknowledgedticket.md)

#### Defined in

[types/acknowledgedTicket.ts:29](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/acknowledgedTicket.ts#L29)
