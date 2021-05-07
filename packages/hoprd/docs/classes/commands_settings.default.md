[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [commands/settings](../modules/commands_settings.md) / default

# Class: default

[commands/settings](../modules/commands_settings.md).default

## Hierarchy

- [_AbstractCommand_](commands_abstractcommand.abstractcommand.md)

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

\+ **new default**(`node`: _Hopr_): [_default_](commands_settings.default.md)

#### Parameters

| Name   | Type   |
| :----- | :----- |
| `node` | _Hopr_ |

**Returns:** [_default_](commands_settings.default.md)

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/settings/index.ts:16](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/settings/index.ts#L16)

## Properties

### hidden

• **hidden**: _boolean_= false

Inherited from: [AbstractCommand](commands_abstractcommand.abstractcommand.md).[hidden](commands_abstractcommand.abstractcommand.md#hidden)

Defined in: [commands/abstractCommand.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L15)

---

### settings

• `Private` **settings**: _any_

Defined in: [commands/settings/index.ts:16](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/settings/index.ts#L16)

## Accessors

### settingsKeys

• `Private` get **settingsKeys**(): _string_[]

**Returns:** _string_[]

Defined in: [commands/settings/index.ts:51](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/settings/index.ts#L51)

## Methods

### \_assertUsage

▸ `Protected` **\_assertUsage**(`query`: _string_, `parameters`: _string_[], `test?`: _RegExp_): _string_[]

#### Parameters

| Name         | Type       |
| :----------- | :--------- |
| `query`      | _string_   |
| `parameters` | _string_[] |
| `test?`      | _RegExp_   |

**Returns:** _string_[]

Inherited from: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/abstractCommand.ts:54](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L54)

---

### \_autocompleteByFiltering

▸ `Protected` **\_autocompleteByFiltering**(`query`: _string_, `allResults`: _string_[], `line`: _string_): [_AutoCompleteResult_](../modules/commands_abstractcommand.md#autocompleteresult)

#### Parameters

| Name         | Type       |
| :----------- | :--------- |
| `query`      | _string_   |
| `allResults` | _string_[] |
| `line`       | _string_   |

**Returns:** [_AutoCompleteResult_](../modules/commands_abstractcommand.md#autocompleteresult)

Inherited from: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/abstractCommand.ts:33](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L33)

---

### autocomplete

▸ **autocomplete**(`_query`: _string_, `line`: _string_, `_state`: [_GlobalState_](../modules/commands_abstractcommand.md#globalstate)): _Promise_<[_AutoCompleteResult_](../modules/commands_abstractcommand.md#autocompleteresult)\>

#### Parameters

| Name     | Type                                                                |
| :------- | :------------------------------------------------------------------ |
| `_query` | _string_                                                            |
| `line`   | _string_                                                            |
| `_state` | [_GlobalState_](../modules/commands_abstractcommand.md#globalstate) |

**Returns:** _Promise_<[_AutoCompleteResult_](../modules/commands_abstractcommand.md#autocompleteresult)\>

Inherited from: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/abstractCommand.ts:26](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L26)

---

### execute

▸ **execute**(`query`: _string_, `state`: [_GlobalState_](../modules/commands_abstractcommand.md#globalstate)): _Promise_<string \| void\>

#### Parameters

| Name    | Type                                                                |
| :------ | :------------------------------------------------------------------ |
| `query` | _string_                                                            |
| `state` | [_GlobalState_](../modules/commands_abstractcommand.md#globalstate) |

**Returns:** _Promise_<string \| void\>

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/settings/index.ts:77](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/settings/index.ts#L77)

---

### getState

▸ `Private` **getState**(`setting`: _string_, `state`: [_GlobalState_](../modules/commands_abstractcommand.md#globalstate)): _string_

#### Parameters

| Name      | Type                                                                |
| :-------- | :------------------------------------------------------------------ |
| `setting` | _string_                                                            |
| `state`   | [_GlobalState_](../modules/commands_abstractcommand.md#globalstate) |

**Returns:** _string_

Defined in: [commands/settings/index.ts:69](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/settings/index.ts#L69)

---

### getStrategy

▸ `Private` **getStrategy**(): _string_

**Returns:** _string_

Defined in: [commands/settings/index.ts:39](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/settings/index.ts#L39)

---

### help

▸ **help**(): _string_

**Returns:** _string_

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/settings/index.ts:47](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/settings/index.ts#L47)

---

### listSettings

▸ `Private` **listSettings**(`state`: [_GlobalState_](../modules/commands_abstractcommand.md#globalstate)): _string_

#### Parameters

| Name    | Type                                                                |
| :------ | :------------------------------------------------------------------ |
| `state` | [_GlobalState_](../modules/commands_abstractcommand.md#globalstate) |

**Returns:** _string_

Defined in: [commands/settings/index.ts:55](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/settings/index.ts#L55)

---

### name

▸ **name**(): _string_

**Returns:** _string_

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/settings/index.ts:43](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/settings/index.ts#L43)

---

### setStrategy

▸ `Private` **setStrategy**(`query`: _string_): _Promise_<string\>

#### Parameters

| Name    | Type     |
| :------ | :------- |
| `query` | _string_ |

**Returns:** _Promise_<string\>

Defined in: [commands/settings/index.ts:30](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/settings/index.ts#L30)

---

### usage

▸ `Protected` **usage**(`parameters`: _string_[]): _string_

#### Parameters

| Name         | Type       |
| :----------- | :--------- |
| `parameters` | _string_[] |

**Returns:** _string_

Inherited from: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/abstractCommand.ts:49](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L49)
