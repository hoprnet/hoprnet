[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [commands](../modules/commands.md) / Commands

# Class: Commands

[commands](../modules/commands.md).Commands

## Table of contents

### Constructors

- [constructor](commands.commands-1.md#constructor)

### Properties

- [commandMap](commands.commands-1.md#commandmap)
- [commands](commands.commands-1.md#commands)
- [node](commands.commands-1.md#node)
- [state](commands.commands-1.md#state)

### Methods

- [allCommands](commands.commands-1.md#allcommands)
- [autocomplete](commands.commands-1.md#autocomplete)
- [execute](commands.commands-1.md#execute)
- [find](commands.commands-1.md#find)
- [setState](commands.commands-1.md#setstate)

## Constructors

### constructor

\+ **new Commands**(`node`: _Hopr_): [_Commands_](commands.commands-1.md)

#### Parameters

| Name   | Type   |
| :----- | :----- |
| `node` | _Hopr_ |

**Returns:** [_Commands_](commands.commands-1.md)

Defined in: [commands/index.ts:30](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/index.ts#L30)

## Properties

### commandMap

• `Private` **commandMap**: _Map_<string, [_AbstractCommand_](commands_abstractcommand.abstractcommand.md)\>

Defined in: [commands/index.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/index.ts#L29)

---

### commands

• `Readonly` **commands**: [_AbstractCommand_](commands_abstractcommand.abstractcommand.md)[]

Defined in: [commands/index.ts:28](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/index.ts#L28)

---

### node

• **node**: _Hopr_

---

### state

• `Private` **state**: [_GlobalState_](../modules/commands_abstractcommand.md#globalstate)

Defined in: [commands/index.ts:30](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/index.ts#L30)

## Methods

### allCommands

▸ **allCommands**(): _string_[]

**Returns:** _string_[]

Defined in: [commands/index.ts:75](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/index.ts#L75)

---

### autocomplete

▸ **autocomplete**(`message`: _string_): _Promise_<[_AutoCompleteResult_](../modules/commands_abstractcommand.md#autocompleteresult)\>

#### Parameters

| Name      | Type     |
| :-------- | :------- |
| `message` | _string_ |

**Returns:** _Promise_<[_AutoCompleteResult_](../modules/commands_abstractcommand.md#autocompleteresult)\>

Defined in: [commands/index.ts:101](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/index.ts#L101)

---

### execute

▸ **execute**(`message`: _string_): _Promise_<[_CommandResponse_](../modules/commands_abstractcommand.md#commandresponse)\>

#### Parameters

| Name      | Type     |
| :-------- | :------- |
| `message` | _string_ |

**Returns:** _Promise_<[_CommandResponse_](../modules/commands_abstractcommand.md#commandresponse)\>

Defined in: [commands/index.ts:83](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/index.ts#L83)

---

### find

▸ **find**(`command`: _string_): [_AbstractCommand_](commands_abstractcommand.abstractcommand.md)

#### Parameters

| Name      | Type     |
| :-------- | :------- |
| `command` | _string_ |

**Returns:** [_AbstractCommand_](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/index.ts:79](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/index.ts#L79)

---

### setState

▸ **setState**(`settings`: _any_): _void_

#### Parameters

| Name       | Type  |
| :--------- | :---- |
| `settings` | _any_ |

**Returns:** _void_

Defined in: [commands/index.ts:71](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/index.ts#L71)
