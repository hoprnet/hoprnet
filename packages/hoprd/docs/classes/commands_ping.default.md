[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [commands/ping](../modules/commands_ping.md) / default

# Class: default

[commands/ping](../modules/commands_ping.md).default

## Hierarchy

- [*AbstractCommand*](commands_abstractcommand.abstractcommand.md)

  ↳ **default**

## Table of contents

### Constructors

- [constructor](commands_ping.default.md#constructor)

### Properties

- [hidden](commands_ping.default.md#hidden)
- [node](commands_ping.default.md#node)

### Methods

- [\_assertUsage](commands_ping.default.md#_assertusage)
- [\_autocompleteByFiltering](commands_ping.default.md#_autocompletebyfiltering)
- [autocomplete](commands_ping.default.md#autocomplete)
- [execute](commands_ping.default.md#execute)
- [help](commands_ping.default.md#help)
- [name](commands_ping.default.md#name)
- [usage](commands_ping.default.md#usage)

## Constructors

### constructor

\+ **new default**(`node`: *Hopr*): [*default*](commands_ping.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `node` | *Hopr* |

**Returns:** [*default*](commands_ping.default.md)

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/ping.ts:7](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/ping.ts#L7)

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

▸ **autocomplete**(`query?`: *string*, `line?`: *string*, `state`: [*GlobalState*](../modules/commands_abstractcommand.md#globalstate)): *Promise*<[*AutoCompleteResult*](../modules/commands_abstractcommand.md#autocompleteresult)\>

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `query` | *string* | '' |
| `line` | *string* | '' |
| `state` | [*GlobalState*](../modules/commands_abstractcommand.md#globalstate) | - |

**Returns:** *Promise*<[*AutoCompleteResult*](../modules/commands_abstractcommand.md#autocompleteresult)\>

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/ping.ts:57](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/ping.ts#L57)

___

### execute

▸ **execute**(`query`: *string*, `state`: [*GlobalState*](../modules/commands_abstractcommand.md#globalstate)): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `query` | *string* |
| `state` | [*GlobalState*](../modules/commands_abstractcommand.md#globalstate) |

**Returns:** *Promise*<string\>

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/ping.ts:20](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/ping.ts#L20)

___

### help

▸ **help**(): *string*

**Returns:** *string*

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/ping.ts:16](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/ping.ts#L16)

___

### name

▸ **name**(): *string*

**Returns:** *string*

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/ping.ts:12](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/ping.ts#L12)

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
