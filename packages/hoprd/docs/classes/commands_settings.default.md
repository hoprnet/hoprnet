[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [commands/settings](../modules/commands_settings.md) / default

# Class: default

[commands/settings](../modules/commands_settings.md).default

## Hierarchy

- [*AbstractCommand*](commands_abstractcommand.abstractcommand.md)

  ↳ **default**

## Table of contents

### Constructors

- [constructor](commands_settings.default.md#constructor)

### Properties

- [hidden](commands_settings.default.md#hidden)
- [settings](commands_settings.default.md#settings)

### Accessors

- [settingsKeys](commands_settings.default.md#settingskeys)

### Methods

- [\_assertUsage](commands_settings.default.md#_assertusage)
- [\_autocompleteByFiltering](commands_settings.default.md#_autocompletebyfiltering)
- [autocomplete](commands_settings.default.md#autocomplete)
- [execute](commands_settings.default.md#execute)
- [getState](commands_settings.default.md#getstate)
- [getStrategy](commands_settings.default.md#getstrategy)
- [help](commands_settings.default.md#help)
- [listSettings](commands_settings.default.md#listsettings)
- [name](commands_settings.default.md#name)
- [setStrategy](commands_settings.default.md#setstrategy)
- [usage](commands_settings.default.md#usage)

## Constructors

### constructor

\+ **new default**(`node`: *Hopr*): [*default*](commands_settings.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `node` | *Hopr* |

**Returns:** [*default*](commands_settings.default.md)

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/settings/index.ts:16](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/settings/index.ts#L16)

## Properties

### hidden

• **hidden**: *boolean*= false

Inherited from: [AbstractCommand](commands_abstractcommand.abstractcommand.md).[hidden](commands_abstractcommand.abstractcommand.md#hidden)

Defined in: [commands/abstractCommand.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L15)

___

### settings

• `Private` **settings**: *any*

Defined in: [commands/settings/index.ts:16](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/settings/index.ts#L16)

## Accessors

### settingsKeys

• `Private` get **settingsKeys**(): *string*[]

**Returns:** *string*[]

Defined in: [commands/settings/index.ts:51](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/settings/index.ts#L51)

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

Defined in: [commands/settings/index.ts:77](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/settings/index.ts#L77)

___

### getState

▸ `Private` **getState**(`setting`: *string*, `state`: [*GlobalState*](../modules/commands_abstractcommand.md#globalstate)): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `setting` | *string* |
| `state` | [*GlobalState*](../modules/commands_abstractcommand.md#globalstate) |

**Returns:** *string*

Defined in: [commands/settings/index.ts:69](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/settings/index.ts#L69)

___

### getStrategy

▸ `Private` **getStrategy**(): *string*

**Returns:** *string*

Defined in: [commands/settings/index.ts:39](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/settings/index.ts#L39)

___

### help

▸ **help**(): *string*

**Returns:** *string*

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/settings/index.ts:47](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/settings/index.ts#L47)

___

### listSettings

▸ `Private` **listSettings**(`state`: [*GlobalState*](../modules/commands_abstractcommand.md#globalstate)): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `state` | [*GlobalState*](../modules/commands_abstractcommand.md#globalstate) |

**Returns:** *string*

Defined in: [commands/settings/index.ts:55](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/settings/index.ts#L55)

___

### name

▸ **name**(): *string*

**Returns:** *string*

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/settings/index.ts:43](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/settings/index.ts#L43)

___

### setStrategy

▸ `Private` **setStrategy**(`query`: *string*): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `query` | *string* |

**Returns:** *Promise*<string\>

Defined in: [commands/settings/index.ts:30](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/settings/index.ts#L30)

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
