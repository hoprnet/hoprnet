[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [commands/listConnectors](../modules/commands_listconnectors.md) / default

# Class: default

[commands/listConnectors](../modules/commands_listconnectors.md).default

## Hierarchy

- [*AbstractCommand*](commands_abstractcommand.abstractcommand.md)

  ↳ **default**

## Table of contents

### Constructors

- [constructor](commands_listconnectors.default.md#constructor)

### Properties

- [hidden](commands_listconnectors.default.md#hidden)

### Methods

- [\_assertUsage](commands_listconnectors.default.md#_assertusage)
- [\_autocompleteByFiltering](commands_listconnectors.default.md#_autocompletebyfiltering)
- [autocomplete](commands_listconnectors.default.md#autocomplete)
- [execute](commands_listconnectors.default.md#execute)
- [help](commands_listconnectors.default.md#help)
- [name](commands_listconnectors.default.md#name)
- [usage](commands_listconnectors.default.md#usage)

## Constructors

### constructor

\+ **new default**(): [*default*](commands_listconnectors.default.md)

**Returns:** [*default*](commands_listconnectors.default.md)

Inherited from: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

## Properties

### hidden

• **hidden**: *boolean*= false

Inherited from: [AbstractCommand](commands_abstractcommand.abstractcommand.md).[hidden](commands_abstractcommand.abstractcommand.md#hidden)

Defined in: [commands/abstractCommand.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L15)

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

Check which connectors are present right now.

**`notice`** triggered by the CLI

**Returns:** *Promise*<string\>

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/listConnectors.ts:17](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/listConnectors.ts#L17)

___

### help

▸ **help**(): *string*

**Returns:** *string*

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/listConnectors.ts:10](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/listConnectors.ts#L10)

___

### name

▸ **name**(): *string*

**Returns:** *string*

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/listConnectors.ts:6](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/listConnectors.ts#L6)

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
