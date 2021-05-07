[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [commands/listConnected](../modules/commands_listconnected.md) / default

# Class: default

[commands/listConnected](../modules/commands_listconnected.md).default

## Hierarchy

- [*AbstractCommand*](commands_abstractcommand.abstractcommand.md)

  ↳ **default**

## Table of contents

### Constructors

- [constructor](commands_listconnected.default.md#constructor)

### Properties

- [hidden](commands_listconnected.default.md#hidden)
- [node](commands_listconnected.default.md#node)

### Methods

- [\_assertUsage](commands_listconnected.default.md#_assertusage)
- [\_autocompleteByFiltering](commands_listconnected.default.md#_autocompletebyfiltering)
- [autocomplete](commands_listconnected.default.md#autocomplete)
- [execute](commands_listconnected.default.md#execute)
- [help](commands_listconnected.default.md#help)
- [name](commands_listconnected.default.md#name)
- [usage](commands_listconnected.default.md#usage)

## Constructors

### constructor

\+ **new default**(`node`: *Hopr*): [*default*](commands_listconnected.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `node` | *Hopr* |

**Returns:** [*default*](commands_listconnected.default.md)

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/listConnected.ts:4](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/listConnected.ts#L4)

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

▸ **execute**(): *Promise*<string \| void\>

**Returns:** *Promise*<string \| void\>

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/listConnected.ts:17](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/listConnected.ts#L17)

___

### help

▸ **help**(): *string*

**Returns:** *string*

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/listConnected.ts:13](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/listConnected.ts#L13)

___

### name

▸ **name**(): *string*

**Returns:** *string*

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/listConnected.ts:9](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/listConnected.ts#L9)

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
