[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [commands/openChannel](../modules/commands_openchannel.md) / OpenChannel

# Class: OpenChannel

[commands/openChannel](../modules/commands_openchannel.md).OpenChannel

## Hierarchy

- [*AbstractCommand*](commands_abstractcommand.abstractcommand.md)

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

\+ **new OpenChannel**(`node`: *Hopr*): [*OpenChannel*](commands_openchannel.openchannel.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `node` | *Hopr* |

**Returns:** [*OpenChannel*](commands_openchannel.openchannel.md)

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/openChannel.ts:9](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/openChannel.ts#L9)

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

▸ **execute**(`query`: *string*, `state`: [*GlobalState*](../modules/commands_abstractcommand.md#globalstate)): *Promise*<string\>

Encapsulates the functionality that is executed once the user decides to open a payment channel
with another party.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `query` | *string* | peerId string to send message to |
| `state` | [*GlobalState*](../modules/commands_abstractcommand.md#globalstate) | - |

**Returns:** *Promise*<string\>

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/openChannel.ts:56](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/openChannel.ts#L56)

___

### help

▸ **help**(): *string*

**Returns:** *string*

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/openChannel.ts:18](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/openChannel.ts#L18)

___

### name

▸ **name**(): *string*

**Returns:** *string*

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/openChannel.ts:14](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/openChannel.ts#L14)

___

### open

▸ **open**(`state`: [*GlobalState*](../modules/commands_abstractcommand.md#globalstate), `counterpartyStr`: *string*, `amountToFundStr`: *string*): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `state` | [*GlobalState*](../modules/commands_abstractcommand.md#globalstate) |
| `counterpartyStr` | *string* |
| `amountToFundStr` | *string* |

**Returns:** *Promise*<string\>

Defined in: [commands/openChannel.ts:32](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/openChannel.ts#L32)

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

___

### validateAmountToFund

▸ `Protected` **validateAmountToFund**(`amountToFund`: *BN*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `amountToFund` | *BN* |

**Returns:** *Promise*<void\>

Defined in: [commands/openChannel.ts:22](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/openChannel.ts#L22)
