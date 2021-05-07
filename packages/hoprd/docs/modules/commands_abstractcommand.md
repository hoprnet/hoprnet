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

Ƭ **AutoCompleteResult**: [*string*[], *string*]

Defined in: [commands/abstractCommand.ts:4](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L4)

___

### CommandResponse

Ƭ **CommandResponse**: *string* \| *void*

Defined in: [commands/abstractCommand.ts:6](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L6)

___

### GlobalState

Ƭ **GlobalState**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `aliases` | *Map*<string, PeerId\> |
| `includeRecipient` | *boolean* |

Defined in: [commands/abstractCommand.ts:8](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L8)

## Functions

### emptyAutoCompleteResult

▸ `Const` **emptyAutoCompleteResult**(`line`: *string*): [*AutoCompleteResult*](commands_abstractcommand.md#autocompleteresult)

#### Parameters

| Name | Type |
| :------ | :------ |
| `line` | *string* |

**Returns:** [*AutoCompleteResult*](commands_abstractcommand.md#autocompleteresult)

Defined in: [commands/abstractCommand.ts:5](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L5)
