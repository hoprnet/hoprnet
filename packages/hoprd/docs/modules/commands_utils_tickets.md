[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / commands/utils/tickets

# Module: commands/utils/tickets

## Table of contents

### Functions

- [countSignedTickets](commands_utils_tickets.md#countsignedtickets)
- [toSignedTickets](commands_utils_tickets.md#tosignedtickets)

## Functions

### countSignedTickets

▸ **countSignedTickets**(`signedTickets`: Ticket[]): _object_

Derive various data from the given signed tickets.

#### Parameters

| Name            | Type     |
| :-------------- | :------- |
| `signedTickets` | Ticket[] |

**Returns:** _object_

| Name      | Type                                             |
| :-------- | :----------------------------------------------- |
| `tickets` | { `amount`: _string_ ; `challange`: _string_ }[] |
| `total`   | _string_                                         |

the total amount of tokens in the tickets & more

Defined in: [commands/utils/tickets.ts:20](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/utils/tickets.ts#L20)

---

### toSignedTickets

▸ **toSignedTickets**(`ackTickets`: AcknowledgedTicket[]): _Promise_<Ticket[]\>

Retrieves all signed tickets from the given acknowledged tickets.

#### Parameters

| Name         | Type                 |
| :----------- | :------------------- |
| `ackTickets` | AcknowledgedTicket[] |

**Returns:** _Promise_<Ticket[]\>

a promise that resolves into an array of signed tickets

Defined in: [commands/utils/tickets.ts:10](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/utils/tickets.ts#L10)
