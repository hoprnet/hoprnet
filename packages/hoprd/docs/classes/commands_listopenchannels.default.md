[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [commands/listOpenChannels](../modules/commands_listopenchannels.md) / default

# Class: default

[commands/listOpenChannels](../modules/commands_listopenchannels.md).default

## Hierarchy

- [*AbstractCommand*](commands_abstractcommand.abstractcommand.md)

  ↳ **default**

## Table of contents

### Constructors

- [constructor](commands_listopenchannels.default.md#constructor)

### Properties

- [hidden](commands_listopenchannels.default.md#hidden)
- [node](commands_listopenchannels.default.md#node)

### Methods

- [\_assertUsage](commands_listopenchannels.default.md#_assertusage)
- [\_autocompleteByFiltering](commands_listopenchannels.default.md#_autocompletebyfiltering)
- [autocomplete](commands_listopenchannels.default.md#autocomplete)
- [execute](commands_listopenchannels.default.md#execute)
- [generateOutput](commands_listopenchannels.default.md#generateoutput)
- [help](commands_listopenchannels.default.md#help)
- [name](commands_listopenchannels.default.md#name)
- [usage](commands_listopenchannels.default.md#usage)

## Constructors

### constructor

\+ **new default**(`node`: *Hopr*): [*default*](commands_listopenchannels.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `node` | *Hopr* |

**Returns:** [*default*](commands_listopenchannels.default.md)

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/listOpenChannels.ts:8](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/listOpenChannels.ts#L8)

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

Lists all channels that we have with other nodes. Triggered from the CLI.

**Returns:** *Promise*<string \| void\>

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/listOpenChannels.ts:68](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/listOpenChannels.ts#L68)

___

### generateOutput

▸ `Private` **generateOutput**(`__namedParameters`: { `id`: *string* ; `myBalance`: *string* ; `peerId?`: *string* ; `status?`: *string* ; `totalBalance`: *string*  }): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `__namedParameters` | *object* |
| `__namedParameters.id` | *string* |
| `__namedParameters.myBalance` | *string* |
| `__namedParameters.peerId?` | *string* |
| `__namedParameters.status?` | *string* |
| `__namedParameters.totalBalance` | *string* |

**Returns:** *string*

Defined in: [commands/listOpenChannels.ts:21](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/listOpenChannels.ts#L21)

___

### help

▸ **help**(): *string*

**Returns:** *string*

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/listOpenChannels.ts:17](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/listOpenChannels.ts#L17)

___

### name

▸ **name**(): *string*

**Returns:** *string*

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/listOpenChannels.ts:13](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/listOpenChannels.ts#L13)

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
