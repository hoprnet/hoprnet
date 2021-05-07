[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [commands/info](../modules/commands_info.md) / Info

# Class: Info

[commands/info](../modules/commands_info.md).Info

## Hierarchy

- [*AbstractCommand*](commands_abstractcommand.abstractcommand.md)

  ↳ **Info**

## Table of contents

### Constructors

- [constructor](commands_info.info.md#constructor)

### Properties

- [hidden](commands_info.info.md#hidden)
- [node](commands_info.info.md#node)

### Methods

- [\_assertUsage](commands_info.info.md#_assertusage)
- [\_autocompleteByFiltering](commands_info.info.md#_autocompletebyfiltering)
- [autocomplete](commands_info.info.md#autocomplete)
- [execute](commands_info.info.md#execute)
- [help](commands_info.info.md#help)
- [name](commands_info.info.md#name)
- [usage](commands_info.info.md#usage)

## Constructors

### constructor

\+ **new Info**(`node`: *Hopr*): [*Info*](commands_info.info.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `node` | *Hopr* |

**Returns:** [*Info*](commands_info.info.md)

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/info.ts:4](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/info.ts#L4)

## Properties

### hidden

• **hidden**: *boolean*= false

Inherited from: [AbstractCommand](commands_abstractcommand.abstractcommand.md).[hidden](commands_abstractcommand.abstractcommand.md#hidden)

Defined in: [commands/abstractCommand.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L15)

___

### node

• **node**: *Hopr*

## Methods

### \_assertUsage

▸ `Protected` **_assertUsage**(`query`: *string*, `parameters`: *string*[], `test?`: *RegExp*): *string*[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `query` | *string* |
| `parameters` | *string*[] |
| `test?` | *RegExp* |

**Returns:** *string*[]

Inherited from: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/abstractCommand.ts:54](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L54)

___

### \_autocompleteByFiltering

▸ `Protected` **_autocompleteByFiltering**(`query`: *string*, `allResults`: *string*[], `line`: *string*): [*AutoCompleteResult*](../modules/commands_abstractcommand.md#autocompleteresult)

#### Parameters

| Name | Type |
| :------ | :------ |
| `query` | *string* |
| `allResults` | *string*[] |
| `line` | *string* |

**Returns:** [*AutoCompleteResult*](../modules/commands_abstractcommand.md#autocompleteresult)

Inherited from: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/abstractCommand.ts:33](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L33)

___

### autocomplete

▸ **autocomplete**(`_query`: *string*, `line`: *string*, `_state`: [*GlobalState*](../modules/commands_abstractcommand.md#globalstate)): *Promise*<[*AutoCompleteResult*](../modules/commands_abstractcommand.md#autocompleteresult)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `_query` | *string* |
| `line` | *string* |
| `_state` | [*GlobalState*](../modules/commands_abstractcommand.md#globalstate) |

**Returns:** *Promise*<[*AutoCompleteResult*](../modules/commands_abstractcommand.md#autocompleteresult)\>

Inherited from: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/abstractCommand.ts:26](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L26)

___

### execute

▸ **execute**(): *Promise*<string\>

**Returns:** *Promise*<string\>

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/info.ts:17](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/info.ts#L17)

___

### help

▸ **help**(): *string*

**Returns:** *string*

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/info.ts:13](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/info.ts#L13)

___

### name

▸ **name**(): *string*

**Returns:** *string*

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/info.ts:9](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/info.ts#L9)

___

### usage

▸ `Protected` **usage**(`parameters`: *string*[]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `parameters` | *string*[] |

**Returns:** *string*

Inherited from: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/abstractCommand.ts:49](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L49)
