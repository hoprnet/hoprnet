[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [commands/ping](../modules/commands_ping.md) / default

# Class: default

[commands/ping](../modules/commands_ping.md).default

## Hierarchy

- [_AbstractCommand_](commands_abstractcommand.abstractcommand.md)

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

\+ **new default**(`node`: _Hopr_): [_default_](commands_ping.default.md)

#### Parameters

| Name   | Type   |
| :----- | :----- |
| `node` | _Hopr_ |

**Returns:** [_default_](commands_ping.default.md)

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/ping.ts:7](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/ping.ts#L7)

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

▸ **autocomplete**(`query?`: _string_, `line?`: _string_, `state`: [_GlobalState_](../modules/commands_abstractcommand.md#globalstate)): _Promise_<[_AutoCompleteResult_](../modules/commands_abstractcommand.md#autocompleteresult)\>

#### Parameters

| Name    | Type                                                                | Default value |
| :------ | :------------------------------------------------------------------ | :------------ |
| `query` | _string_                                                            | ''            |
| `line`  | _string_                                                            | ''            |
| `state` | [_GlobalState_](../modules/commands_abstractcommand.md#globalstate) | -             |

**Returns:** _Promise_<[_AutoCompleteResult_](../modules/commands_abstractcommand.md#autocompleteresult)\>

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/ping.ts:57](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/ping.ts#L57)

---

### execute

▸ **execute**(`query`: _string_, `state`: [_GlobalState_](../modules/commands_abstractcommand.md#globalstate)): _Promise_<string\>

#### Parameters

| Name    | Type                                                                |
| :------ | :------------------------------------------------------------------ |
| `query` | _string_                                                            |
| `state` | [_GlobalState_](../modules/commands_abstractcommand.md#globalstate) |

**Returns:** _Promise_<string\>

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/ping.ts:20](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/ping.ts#L20)

---

### help

▸ **help**(): _string_

**Returns:** _string_

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/ping.ts:16](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/ping.ts#L16)

---

### name

▸ **name**(): _string_

**Returns:** _string_

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/ping.ts:12](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/ping.ts#L12)

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
