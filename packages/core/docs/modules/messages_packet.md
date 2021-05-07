[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / messages/packet

# Module: messages/packet

## Table of contents

### Classes

- [Packet](../classes/messages_packet.packet.md)

### Variables

- [MAX_HOPS](messages_packet.md#max_hops)

### Functions

- [validateCreatedTicket](messages_packet.md#validatecreatedticket)
- [validateUnacknowledgedTicket](messages_packet.md#validateunacknowledgedticket)

## Variables

### MAX_HOPS

• `Const` **MAX_HOPS**: `3`= 3

Defined in: [packages/core/src/messages/packet.ts:32](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L32)

## Functions

### validateCreatedTicket

▸ **validateCreatedTicket**(`myBalance`: BN, `ticket`: Ticket): _void_

Validate newly created tickets

#### Parameters

| Name        | Type   |
| :---------- | :----- |
| `myBalance` | BN     |
| `ticket`    | Ticket |

**Returns:** _void_

Defined in: [packages/core/src/messages/packet.ts:42](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L42)

---

### validateUnacknowledgedTicket

▸ **validateUnacknowledgedTicket**(`id`: PeerId, `nodeTicketAmount`: _string_, `nodeTicketWinProb`: _number_, `senderPeerId`: PeerId, `ticket`: Ticket, `channel`: Channel, `getTickets`: () => _Promise_<Ticket[]\>): _Promise_<void\>

Validate unacknowledged tickets as we receive them

#### Parameters

| Name                | Type                       |
| :------------------ | :------------------------- |
| `id`                | PeerId                     |
| `nodeTicketAmount`  | _string_                   |
| `nodeTicketWinProb` | _number_                   |
| `senderPeerId`      | PeerId                     |
| `ticket`            | Ticket                     |
| `channel`           | Channel                    |
| `getTickets`        | () => _Promise_<Ticket[]\> |

**Returns:** _Promise_<void\>

Defined in: [packages/core/src/messages/packet.ts:53](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/packet.ts#L53)
