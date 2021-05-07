[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [commands/addresses](../modules/commands_addresses.md) / default

# Class: default

[commands/addresses](../modules/commands_addresses.md).default

## Hierarchy

- [_AbstractCommand_](commands_abstractcommand.abstractcommand.md)

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

\+ **new default**(`node`: _Hopr_): [_default_](commands_addresses.default.md)

#### Parameters

| Name   | Type   |
| :----- | :----- |
| `node` | _Hopr_ |

**Returns:** [_default_](commands_addresses.default.md)

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/addresses.ts:7](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/addresses.ts#L7)

## Properties

### hidden

• **hidden**: _boolean_= false

Inherited from: [AbstractCommand](commands_abstractcommand.abstractcommand.md).[hidden](commands_abstractcommand.abstractcommand.md#hidden)

Defined in: [commands/abstractCommand.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L15)

---

### node

• **node**: _Hopr_

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

Defined in: [commands/addresses.ts:21](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/addresses.ts#L21)

---

### help

▸ **help**(): _string_

**Returns:** _string_

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/addresses.ts:17](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/addresses.ts#L17)

---

### name

▸ **name**(): _string_

**Returns:** _string_

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/addresses.ts:13](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/addresses.ts#L13)

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
