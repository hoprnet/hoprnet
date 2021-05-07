[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [commands/tickets](../modules/commands_tickets.md) / default

# Class: default

[commands/tickets](../modules/commands_tickets.md).default

## Hierarchy

- [_AbstractCommand_](commands_abstractcommand.abstractcommand.md)

  ↳ **default**

## Table of contents

### Constructors

- [constructor](commands_tickets.default.md#constructor)

### Properties

- [hidden](commands_tickets.default.md#hidden)
- [node](commands_tickets.default.md#node)

### Methods

- [\_assertUsage](commands_tickets.default.md#_assertusage)
- [\_autocompleteByFiltering](commands_tickets.default.md#_autocompletebyfiltering)
- [autocomplete](commands_tickets.default.md#autocomplete)
- [execute](commands_tickets.default.md#execute)
- [help](commands_tickets.default.md#help)
- [name](commands_tickets.default.md#name)
- [usage](commands_tickets.default.md#usage)

## Constructors

### constructor

\+ **new default**(`node`: _Hopr_): [_default_](commands_tickets.default.md)

#### Parameters

| Name   | Type   |
| :----- | :----- |
| `node` | _Hopr_ |

**Returns:** [_default_](commands_tickets.default.md)

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/tickets.ts:6](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/tickets.ts#L6)

## Properties

### hidden

• **hidden**: _boolean_= false

Inherited from: [AbstractCommand](commands_abstractcommand.abstractcommand.md).[hidden](commands_abstractcommand.abstractcommand.md#hidden)

Defined in: [commands/abstractCommand.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L15)

---

### node

• **node**: _Hopr_

## Methods

### \_assertUsage

▸ `Protected` **\_assertUsage**(`query`: _string_, `parameters`: _string_[], `test?`: _RegExp_): _string_[]

#### Parameters

| Name         | Type       |
| :----------- | :--------- |
| `query`      | _string_   |
| `parameters` | _string_[] |
| `test?`      | _RegExp_   |

**Returns:** _string_[]

Inherited from: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/abstractCommand.ts:54](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L54)

---

### \_autocompleteByFiltering

▸ `Protected` **\_autocompleteByFiltering**(`query`: _string_, `allResults`: _string_[], `line`: _string_): [_AutoCompleteResult_](../modules/commands_abstractcommand.md#autocompleteresult)

#### Parameters

| Name         | Type       |
| :----------- | :--------- |
| `query`      | _string_   |
| `allResults` | _string_[] |
| `line`       | _string_   |

**Returns:** [_AutoCompleteResult_](../modules/commands_abstractcommand.md#autocompleteresult)

Inherited from: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/abstractCommand.ts:33](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L33)

---

### autocomplete

▸ **autocomplete**(`_query`: _string_, `line`: _string_, `_state`: [_GlobalState_](../modules/commands_abstractcommand.md#globalstate)): _Promise_<[_AutoCompleteResult_](../modules/commands_abstractcommand.md#autocompleteresult)\>

#### Parameters

| Name     | Type                                                                |
| :------- | :------------------------------------------------------------------ |
| `_query` | _string_                                                            |
| `line`   | _string_                                                            |
| `_state` | [_GlobalState_](../modules/commands_abstractcommand.md#globalstate) |

**Returns:** _Promise_<[_AutoCompleteResult_](../modules/commands_abstractcommand.md#autocompleteresult)\>

Inherited from: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/abstractCommand.ts:26](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L26)

---

### execute

▸ **execute**(): _Promise_<string \| void\>

**Returns:** _Promise_<string \| void\>

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/tickets.ts:19](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/tickets.ts#L19)

---

### help

▸ **help**(): _string_

**Returns:** _string_

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/tickets.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/tickets.ts#L15)

---

### name

▸ **name**(): _string_

**Returns:** _string_

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/tickets.ts:11](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/tickets.ts#L11)

---

### usage

▸ `Protected` **usage**(`parameters`: _string_[]): _string_

#### Parameters

| Name         | Type       |
| :----------- | :--------- |
| `parameters` | _string_[] |

**Returns:** _string_

Inherited from: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/abstractCommand.ts:49](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L49)
