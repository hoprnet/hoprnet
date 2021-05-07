[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [commands/abstractCommand](../modules/commands_abstractcommand.md) / AbstractCommand

# Class: AbstractCommand

[commands/abstractCommand](../modules/commands_abstractcommand.md).AbstractCommand

## Hierarchy

- **AbstractCommand**

  ↳ [_default_](commands_addresses.default.md)

  ↳ [_Alias_](commands_alias.alias.md)

  ↳ [_default_](commands_closechannel.default.md)

  ↳ [_CoverTraffic_](commands_cover_traffic.covertraffic.md)

  ↳ [_default_](commands_fundchannel.default.md)

  ↳ [_Info_](commands_info.info.md)

  ↳ [_default_](commands_listcommands.default.md)

  ↳ [_default_](commands_listconnected.default.md)

  ↳ [_default_](commands_listconnectors.default.md)

  ↳ [_default_](commands_listopenchannels.default.md)

  ↳ [_OpenChannel_](commands_openchannel.openchannel.md)

  ↳ [_default_](commands_ping.default.md)

  ↳ [_default_](commands_printaddress.default.md)

  ↳ [_default_](commands_printbalance.default.md)

  ↳ [_default_](commands_redeemtickets.default.md)

  ↳ [_SendMessageBase_](commands_sendmessage.sendmessagebase.md)

  ↳ [_default_](commands_settings.default.md)

  ↳ [_default_](commands_stopnode.default.md)

  ↳ [_default_](commands_tickets.default.md)

  ↳ [_default_](commands_version.default.md)

  ↳ [_default_](commands_withdraw.default.md)

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

\+ **new AbstractCommand**(): [_AbstractCommand_](commands_abstractcommand.abstractcommand.md)

**Returns:** [_AbstractCommand_](commands_abstractcommand.abstractcommand.md)

## Properties

### hidden

• **hidden**: _boolean_= false

Defined in: [commands/abstractCommand.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L15)

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

Defined in: [commands/abstractCommand.ts:26](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L26)

---

### execute

▸ `Abstract` **execute**(`query`: _string_, `state`: [_GlobalState_](../modules/commands_abstractcommand.md#globalstate)): [_CommandResponse_](../modules/commands_abstractcommand.md#commandresponse) \| _Promise_<[_CommandResponse_](../modules/commands_abstractcommand.md#commandresponse)\>

#### Parameters

| Name    | Type                                                                |
| :------ | :------------------------------------------------------------------ |
| `query` | _string_                                                            |
| `state` | [_GlobalState_](../modules/commands_abstractcommand.md#globalstate) |

**Returns:** [_CommandResponse_](../modules/commands_abstractcommand.md#commandresponse) \| _Promise_<[_CommandResponse_](../modules/commands_abstractcommand.md#commandresponse)\>

Defined in: [commands/abstractCommand.ts:24](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L24)

---

### help

▸ `Abstract` **help**(): _string_

**Returns:** _string_

Defined in: [commands/abstractCommand.ts:21](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L21)

---

### name

▸ `Abstract` **name**(): _string_

**Returns:** _string_

Defined in: [commands/abstractCommand.ts:18](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L18)

---

### usage

▸ `Protected` **usage**(`parameters`: _string_[]): _string_

#### Parameters

| Name         | Type       |
| :----------- | :--------- |
| `parameters` | _string_[] |

**Returns:** _string_

Defined in: [commands/abstractCommand.ts:49](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L49)
