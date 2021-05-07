[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [commands/addresses](../modules/commands_addresses.md) / default

# Class: default

[commands/addresses](../modules/commands_addresses.md).default

## Hierarchy

- [*AbstractCommand*](commands_abstractcommand.abstractcommand.md)

  ↳ **default**

## Table of contents

### Constructors

- [constructor](commands_addresses.default.md#constructor)

### Properties

- [hidden](commands_addresses.default.md#hidden)
- [node](commands_addresses.default.md#node)

### Methods

- [\_assertUsage](commands_addresses.default.md#_assertusage)
- [\_autocompleteByFiltering](commands_addresses.default.md#_autocompletebyfiltering)
- [autocomplete](commands_addresses.default.md#autocomplete)
- [execute](commands_addresses.default.md#execute)
- [help](commands_addresses.default.md#help)
- [name](commands_addresses.default.md#name)
- [usage](commands_addresses.default.md#usage)

## Constructors

### constructor

\+ **new default**(`node`: *Hopr*): [*default*](commands_addresses.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `node` | *Hopr* |

**Returns:** [*default*](commands_addresses.default.md)

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/addresses.ts:7](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/addresses.ts#L7)

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

▸ **execute**(`query`: *string*, `state`: [*GlobalState*](../modules/commands_abstractcommand.md#globalstate)): *Promise*<string \| void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `query` | *string* |
| `state` | [*GlobalState*](../modules/commands_abstractcommand.md#globalstate) |

**Returns:** *Promise*<string \| void\>

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/addresses.ts:21](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/addresses.ts#L21)

___

### help

▸ **help**(): *string*

**Returns:** *string*

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/addresses.ts:17](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/addresses.ts#L17)

___

### name

▸ **name**(): *string*

**Returns:** *string*

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/addresses.ts:13](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/addresses.ts#L13)

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
