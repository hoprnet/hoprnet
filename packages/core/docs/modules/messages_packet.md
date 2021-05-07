[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / messages/packet

# Module: messages/packet

## Table of contents

### Classes

- [Packet](../classes/messages_packet.packet.md)

### Variables

- [MAX\_HOPS](messages_packet.md#max_hops)

### Functions

- [validateCreatedTicket](messages_packet.md#validatecreatedticket)
- [validateUnacknowledgedTicket](messages_packet.md#validateunacknowledgedticket)

## Variables

### MAX\_HOPS

• `Const` **MAX\_HOPS**: ``3``= 3

Defined in: [packages/core/src/messages/packet.ts:32](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L32)

## Functions

### validateCreatedTicket

▸ **validateCreatedTicket**(`myBalance`: BN, `ticket`: Ticket): *void*

Validate newly created tickets

#### Parameters

| Name | Type |
| :------ | :------ |
| `myBalance` | BN |
| `ticket` | Ticket |

**Returns:** *void*

Defined in: [packages/core/src/messages/packet.ts:42](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L42)

___

### validateUnacknowledgedTicket

▸ **validateUnacknowledgedTicket**(`id`: PeerId, `nodeTicketAmount`: *string*, `nodeTicketWinProb`: *number*, `senderPeerId`: PeerId, `ticket`: Ticket, `channel`: Channel, `getTickets`: () => *Promise*<Ticket[]\>): *Promise*<void\>

Validate unacknowledged tickets as we receive them

#### Parameters

| Name | Type |
| :------ | :------ |
| `id` | PeerId |
| `nodeTicketAmount` | *string* |
| `nodeTicketWinProb` | *number* |
| `senderPeerId` | PeerId |
| `ticket` | Ticket |
| `channel` | Channel |
| `getTickets` | () => *Promise*<Ticket[]\> |

**Returns:** *Promise*<void\>

Defined in: [packages/core/src/messages/packet.ts:53](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L53)
