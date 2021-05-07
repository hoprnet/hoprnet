[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / commands/abstractCommand

# Module: commands/abstractCommand

## Table of contents

### Classes

- [AbstractCommand](../classes/commands_abstractcommand.abstractcommand.md)

### Type aliases

- [AutoCompleteResult](commands_abstractcommand.md#autocompleteresult)
- [CommandResponse](commands_abstractcommand.md#commandresponse)
- [GlobalState](commands_abstractcommand.md#globalstate)

### Functions

- [emptyAutoCompleteResult](commands_abstractcommand.md#emptyautocompleteresult)

## Type aliases

### AutoCompleteResult

Ƭ **AutoCompleteResult**: [_string_[], _string_]

Defined in: [commands/abstractCommand.ts:4](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L4)

---

### CommandResponse

Ƭ **CommandResponse**: _string_ \| _void_

Defined in: [commands/abstractCommand.ts:6](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L6)

---

### GlobalState

Ƭ **GlobalState**: _object_

#### Type declaration

| Name               | Type                   |
| :----------------- | :--------------------- |
| `aliases`          | _Map_<string, PeerId\> |
| `includeRecipient` | _boolean_              |

Defined in: [commands/abstractCommand.ts:8](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L8)

## Functions

### emptyAutoCompleteResult

▸ `Const` **emptyAutoCompleteResult**(`line`: _string_): [_AutoCompleteResult_](commands_abstractcommand.md#autocompleteresult)

#### Parameters

| Name   | Type     |
| :----- | :------- |
| `line` | _string_ |

**Returns:** [_AutoCompleteResult_](commands_abstractcommand.md#autocompleteresult)

Defined in: [commands/abstractCommand.ts:5](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L5)
