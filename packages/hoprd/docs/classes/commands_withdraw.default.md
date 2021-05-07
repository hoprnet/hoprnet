[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [commands/withdraw](../modules/commands_withdraw.md) / default

# Class: default

[commands/withdraw](../modules/commands_withdraw.md).default

## Hierarchy

- [_AbstractCommand_](commands_abstractcommand.abstractcommand.md)

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

\+ **new default**(`node`: _Hopr_): [_default_](commands_withdraw.default.md)

#### Parameters

| Name   | Type   |
| :----- | :----- |
| `node` | _Hopr_ |

**Returns:** [_default_](commands_withdraw.default.md)

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/withdraw.ts:8](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/withdraw.ts#L8)

## Properties

### arguments

• `Private` **arguments**: _string_[]

Defined in: [commands/withdraw.ts:8](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/withdraw.ts#L8)

---

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

▸ **autocomplete**(`query?`: _string_): _Promise_<[_AutoCompleteResult_](../modules/commands_abstractcommand.md#autocompleteresult)\>

#### Parameters

| Name     | Type     |
| :------- | :------- |
| `query?` | _string_ |

**Returns:** _Promise_<[_AutoCompleteResult_](../modules/commands_abstractcommand.md#autocompleteresult)\>

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/withdraw.ts:65](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/withdraw.ts#L65)

---

### checkArgs

▸ `Private` **checkArgs**(`query`: _string_): _Promise_<{ `amount`: _string_ ; `currency`: `"NATIVE"` \| `"HOPR"` ; `recipient`: _string_ ; `weiAmount`: _string_ }\>

Will throw if any of the arguments are incorrect.

#### Parameters

| Name    | Type     |
| :------ | :------- |
| `query` | _string_ |

**Returns:** _Promise_<{ `amount`: _string_ ; `currency`: `"NATIVE"` \| `"HOPR"` ; `recipient`: _string_ ; `weiAmount`: _string_ }\>

Defined in: [commands/withdraw.ts:17](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/withdraw.ts#L17)

---

### execute

▸ **execute**(`query?`: _string_): _Promise_<string\>

Withdraws native or hopr balance.

**`notice`** triggered by the CLI

#### Parameters

| Name     | Type     |
| :------- | :------- |
| `query?` | _string_ |

**Returns:** _Promise_<string\>

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/withdraw.ts:73](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/withdraw.ts#L73)

---

### help

▸ **help**(): _string_

**Returns:** _string_

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/withdraw.ts:61](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/withdraw.ts#L61)

---

### name

▸ **name**(): _string_

**Returns:** _string_

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/withdraw.ts:57](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/withdraw.ts#L57)

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
