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

\+ **new Commands**(`node`: *Hopr*): [*Commands*](commands.commands-1.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `node` | *Hopr* |

**Returns:** [*Commands*](commands.commands-1.md)

Defined in: [commands/index.ts:30](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/index.ts#L30)

## Properties

### commandMap

• `Private` **commandMap**: *Map*<string, [*AbstractCommand*](commands_abstractcommand.abstractcommand.md)\>

Defined in: [commands/index.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/index.ts#L29)

___

### commands

• `Readonly` **commands**: [*AbstractCommand*](commands_abstractcommand.abstractcommand.md)[]

Defined in: [commands/index.ts:28](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/index.ts#L28)

___

### node

• **node**: *Hopr*

___

### state

• `Private` **state**: [*GlobalState*](../modules/commands_abstractcommand.md#globalstate)

Defined in: [commands/index.ts:30](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/index.ts#L30)

## Methods

### allCommands

▸ **allCommands**(): *string*[]

**Returns:** *string*[]

Defined in: [commands/index.ts:75](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/index.ts#L75)

___

### autocomplete

▸ **autocomplete**(`message`: *string*): *Promise*<[*AutoCompleteResult*](../modules/commands_abstractcommand.md#autocompleteresult)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `message` | *string* |

**Returns:** *Promise*<[*AutoCompleteResult*](../modules/commands_abstractcommand.md#autocompleteresult)\>

Defined in: [commands/index.ts:101](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/index.ts#L101)

___

### execute

▸ **execute**(`message`: *string*): *Promise*<[*CommandResponse*](../modules/commands_abstractcommand.md#commandresponse)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `message` | *string* |

**Returns:** *Promise*<[*CommandResponse*](../modules/commands_abstractcommand.md#commandresponse)\>

Defined in: [commands/index.ts:83](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/index.ts#L83)

___

### find

▸ **find**(`command`: *string*): [*AbstractCommand*](commands_abstractcommand.abstractcommand.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `command` | *string* |

**Returns:** [*AbstractCommand*](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/index.ts:79](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/index.ts#L79)

___

### setState

▸ **setState**(`settings`: *any*): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `settings` | *any* |

**Returns:** *void*

Defined in: [commands/index.ts:71](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/index.ts#L71)
