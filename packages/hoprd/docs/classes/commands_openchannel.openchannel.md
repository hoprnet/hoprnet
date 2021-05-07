[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [commands/openChannel](../modules/commands_openchannel.md) / OpenChannel

# Class: OpenChannel

[commands/openChannel](../modules/commands_openchannel.md).OpenChannel

## Hierarchy

- [_AbstractCommand_](commands_abstractcommand.abstractcommand.md)

  ↳ **OpenChannel**

## Table of contents

### Constructors

- [constructor](commands_openchannel.openchannel.md#constructor)

### Properties

- [hidden](commands_openchannel.openchannel.md#hidden)
- [node](commands_openchannel.openchannel.md#node)

### Methods

- [\_assertUsage](commands_openchannel.openchannel.md#_assertusage)
- [\_autocompleteByFiltering](commands_openchannel.openchannel.md#_autocompletebyfiltering)
- [autocomplete](commands_openchannel.openchannel.md#autocomplete)
- [execute](commands_openchannel.openchannel.md#execute)
- [help](commands_openchannel.openchannel.md#help)
- [name](commands_openchannel.openchannel.md#name)
- [open](commands_openchannel.openchannel.md#open)
- [usage](commands_openchannel.openchannel.md#usage)
- [validateAmountToFund](commands_openchannel.openchannel.md#validateamounttofund)

## Constructors

### constructor

\+ **new OpenChannel**(`node`: _Hopr_): [_OpenChannel_](commands_openchannel.openchannel.md)

#### Parameters

| Name   | Type   |
| :----- | :----- |
| `node` | _Hopr_ |

**Returns:** [_OpenChannel_](commands_openchannel.openchannel.md)

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/openChannel.ts:9](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/openChannel.ts#L9)

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

▸ **execute**(`query`: _string_, `state`: [_GlobalState_](../modules/commands_abstractcommand.md#globalstate)): _Promise_<string\>

Encapsulates the functionality that is executed once the user decides to open a payment channel
with another party.

#### Parameters

| Name    | Type                                                                | Description                      |
| :------ | :------------------------------------------------------------------ | :------------------------------- |
| `query` | _string_                                                            | peerId string to send message to |
| `state` | [_GlobalState_](../modules/commands_abstractcommand.md#globalstate) | -                                |

**Returns:** _Promise_<string\>

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/openChannel.ts:56](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/openChannel.ts#L56)

---

### help

▸ **help**(): _string_

**Returns:** _string_

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/openChannel.ts:18](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/openChannel.ts#L18)

---

### name

▸ **name**(): _string_

**Returns:** _string_

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/openChannel.ts:14](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/openChannel.ts#L14)

---

### open

▸ **open**(`state`: [_GlobalState_](../modules/commands_abstractcommand.md#globalstate), `counterpartyStr`: _string_, `amountToFundStr`: _string_): _Promise_<string\>

#### Parameters

| Name              | Type                                                                |
| :---------------- | :------------------------------------------------------------------ |
| `state`           | [_GlobalState_](../modules/commands_abstractcommand.md#globalstate) |
| `counterpartyStr` | _string_                                                            |
| `amountToFundStr` | _string_                                                            |

**Returns:** _Promise_<string\>

Defined in: [commands/openChannel.ts:32](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/openChannel.ts#L32)

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

---

### validateAmountToFund

▸ `Protected` **validateAmountToFund**(`amountToFund`: _BN_): _Promise_<void\>

#### Parameters

| Name           | Type |
| :------------- | :--- |
| `amountToFund` | _BN_ |

**Returns:** _Promise_<void\>

Defined in: [commands/openChannel.ts:22](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/openChannel.ts#L22)
