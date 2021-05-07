[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [commands/abstractCommand](../modules/commands_abstractcommand.md) / AbstractCommand

# Class: AbstractCommand

[commands/abstractCommand](../modules/commands_abstractcommand.md).AbstractCommand

## Hierarchy

- **AbstractCommand**

  ↳ [*default*](commands_addresses.default.md)

  ↳ [*Alias*](commands_alias.alias.md)

  ↳ [*default*](commands_closechannel.default.md)

  ↳ [*CoverTraffic*](commands_cover_traffic.covertraffic.md)

  ↳ [*default*](commands_fundchannel.default.md)

  ↳ [*Info*](commands_info.info.md)

  ↳ [*default*](commands_listcommands.default.md)

  ↳ [*default*](commands_listconnected.default.md)

  ↳ [*default*](commands_listconnectors.default.md)

  ↳ [*default*](commands_listopenchannels.default.md)

  ↳ [*OpenChannel*](commands_openchannel.openchannel.md)

  ↳ [*default*](commands_ping.default.md)

  ↳ [*default*](commands_printaddress.default.md)

  ↳ [*default*](commands_printbalance.default.md)

  ↳ [*default*](commands_redeemtickets.default.md)

  ↳ [*SendMessageBase*](commands_sendmessage.sendmessagebase.md)

  ↳ [*default*](commands_settings.default.md)

  ↳ [*default*](commands_stopnode.default.md)

  ↳ [*default*](commands_tickets.default.md)

  ↳ [*default*](commands_version.default.md)

  ↳ [*default*](commands_withdraw.default.md)

## Table of contents

### Constructors

- [constructor](commands_abstractcommand.abstractcommand.md#constructor)

### Properties

- [hidden](commands_abstractcommand.abstractcommand.md#hidden)

### Methods

- [\_assertUsage](commands_abstractcommand.abstractcommand.md#_assertusage)
- [\_autocompleteByFiltering](commands_abstractcommand.abstractcommand.md#_autocompletebyfiltering)
- [autocomplete](commands_abstractcommand.abstractcommand.md#autocomplete)
- [execute](commands_abstractcommand.abstractcommand.md#execute)
- [help](commands_abstractcommand.abstractcommand.md#help)
- [name](commands_abstractcommand.abstractcommand.md#name)
- [usage](commands_abstractcommand.abstractcommand.md#usage)

## Constructors

### constructor

\+ **new AbstractCommand**(): [*AbstractCommand*](commands_abstractcommand.abstractcommand.md)

**Returns:** [*AbstractCommand*](commands_abstractcommand.abstractcommand.md)

## Properties

### hidden

• **hidden**: *boolean*= false

Defined in: [commands/abstractCommand.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L15)

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

Defined in: [commands/abstractCommand.ts:26](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L26)

___

### execute

▸ `Abstract` **execute**(`query`: *string*, `state`: [*GlobalState*](../modules/commands_abstractcommand.md#globalstate)): [*CommandResponse*](../modules/commands_abstractcommand.md#commandresponse) \| *Promise*<[*CommandResponse*](../modules/commands_abstractcommand.md#commandresponse)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `query` | *string* |
| `state` | [*GlobalState*](../modules/commands_abstractcommand.md#globalstate) |

**Returns:** [*CommandResponse*](../modules/commands_abstractcommand.md#commandresponse) \| *Promise*<[*CommandResponse*](../modules/commands_abstractcommand.md#commandresponse)\>

Defined in: [commands/abstractCommand.ts:24](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L24)

___

### help

▸ `Abstract` **help**(): *string*

**Returns:** *string*

Defined in: [commands/abstractCommand.ts:21](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L21)

___

### name

▸ `Abstract` **name**(): *string*

**Returns:** *string*

Defined in: [commands/abstractCommand.ts:18](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L18)

___

### usage

▸ `Protected` **usage**(`parameters`: *string*[]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `parameters` | *string*[] |

**Returns:** *string*

Defined in: [commands/abstractCommand.ts:49](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L49)
