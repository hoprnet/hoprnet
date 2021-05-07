[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [commands/withdraw](../modules/commands_withdraw.md) / default

# Class: default

[commands/withdraw](../modules/commands_withdraw.md).default

## Hierarchy

- [*AbstractCommand*](commands_abstractcommand.abstractcommand.md)

  ↳ **default**

## Table of contents

### Constructors

- [constructor](commands_withdraw.default.md#constructor)

### Properties

- [arguments](commands_withdraw.default.md#arguments)
- [hidden](commands_withdraw.default.md#hidden)
- [node](commands_withdraw.default.md#node)

### Methods

- [\_assertUsage](commands_withdraw.default.md#_assertusage)
- [\_autocompleteByFiltering](commands_withdraw.default.md#_autocompletebyfiltering)
- [autocomplete](commands_withdraw.default.md#autocomplete)
- [checkArgs](commands_withdraw.default.md#checkargs)
- [execute](commands_withdraw.default.md#execute)
- [help](commands_withdraw.default.md#help)
- [name](commands_withdraw.default.md#name)
- [usage](commands_withdraw.default.md#usage)

## Constructors

### constructor

\+ **new default**(`node`: *Hopr*): [*default*](commands_withdraw.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `node` | *Hopr* |

**Returns:** [*default*](commands_withdraw.default.md)

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/withdraw.ts:8](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/withdraw.ts#L8)

## Properties

### arguments

• `Private` **arguments**: *string*[]

Defined in: [commands/withdraw.ts:8](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/withdraw.ts#L8)

___

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

▸ **autocomplete**(`query?`: *string*): *Promise*<[*AutoCompleteResult*](../modules/commands_abstractcommand.md#autocompleteresult)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `query?` | *string* |

**Returns:** *Promise*<[*AutoCompleteResult*](../modules/commands_abstractcommand.md#autocompleteresult)\>

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/withdraw.ts:65](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/withdraw.ts#L65)

___

### checkArgs

▸ `Private` **checkArgs**(`query`: *string*): *Promise*<{ `amount`: *string* ; `currency`: ``"NATIVE"`` \| ``"HOPR"`` ; `recipient`: *string* ; `weiAmount`: *string*  }\>

Will throw if any of the arguments are incorrect.

#### Parameters

| Name | Type |
| :------ | :------ |
| `query` | *string* |

**Returns:** *Promise*<{ `amount`: *string* ; `currency`: ``"NATIVE"`` \| ``"HOPR"`` ; `recipient`: *string* ; `weiAmount`: *string*  }\>

Defined in: [commands/withdraw.ts:17](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/withdraw.ts#L17)

___

### execute

▸ **execute**(`query?`: *string*): *Promise*<string\>

Withdraws native or hopr balance.

**`notice`** triggered by the CLI

#### Parameters

| Name | Type |
| :------ | :------ |
| `query?` | *string* |

**Returns:** *Promise*<string\>

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/withdraw.ts:73](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/withdraw.ts#L73)

___

### help

▸ **help**(): *string*

**Returns:** *string*

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/withdraw.ts:61](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/withdraw.ts#L61)

___

### name

▸ **name**(): *string*

**Returns:** *string*

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/withdraw.ts:57](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/withdraw.ts#L57)

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
