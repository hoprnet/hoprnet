[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / commands/utils/tickets

# Module: commands/utils/tickets

## Table of contents

### Functions

- [countSignedTickets](commands_utils_tickets.md#countsignedtickets)
- [toSignedTickets](commands_utils_tickets.md#tosignedtickets)

## Functions

### countSignedTickets

▸ **countSignedTickets**(`signedTickets`: Ticket[]): *object*

Derive various data from the given signed tickets.

#### Parameters

| Name | Type |
| :------ | :------ |
| `signedTickets` | Ticket[] |

**Returns:** *object*

| Name | Type |
| :------ | :------ |
| `tickets` | { `amount`: *string* ; `challange`: *string*  }[] |
| `total` | *string* |

the total amount of tokens in the tickets & more

Defined in: [commands/utils/tickets.ts:20](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/utils/tickets.ts#L20)

___

### toSignedTickets

▸ **toSignedTickets**(`ackTickets`: AcknowledgedTicket[]): *Promise*<Ticket[]\>

Retrieves all signed tickets from the given acknowledged tickets.

#### Parameters

| Name | Type |
| :------ | :------ |
| `ackTickets` | AcknowledgedTicket[] |

**Returns:** *Promise*<Ticket[]\>

a promise that resolves into an array of signed tickets

Defined in: [commands/utils/tickets.ts:10](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/utils/tickets.ts#L10)
