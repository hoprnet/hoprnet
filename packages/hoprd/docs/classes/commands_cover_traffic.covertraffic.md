[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [commands/cover-traffic](../modules/commands_cover_traffic.md) / CoverTraffic

# Class: CoverTraffic

[commands/cover-traffic](../modules/commands_cover_traffic.md).CoverTraffic

## Hierarchy

- [*AbstractCommand*](commands_abstractcommand.abstractcommand.md)

  ↳ **CoverTraffic**

## Table of contents

### Constructors

- [constructor](commands_cover_traffic.covertraffic.md#constructor)

### Properties

- [hidden](commands_cover_traffic.covertraffic.md#hidden)
- [identifier](commands_cover_traffic.covertraffic.md#identifier)
- [messagesReceived](commands_cover_traffic.covertraffic.md#messagesreceived)
- [messagesSent](commands_cover_traffic.covertraffic.md#messagessent)
- [node](commands_cover_traffic.covertraffic.md#node)
- [registered](commands_cover_traffic.covertraffic.md#registered)
- [seq](commands_cover_traffic.covertraffic.md#seq)
- [timeout](commands_cover_traffic.covertraffic.md#timeout)
- [totalLatency](commands_cover_traffic.covertraffic.md#totallatency)

### Methods

- [\_assertUsage](commands_cover_traffic.covertraffic.md#_assertusage)
- [\_autocompleteByFiltering](commands_cover_traffic.covertraffic.md#_autocompletebyfiltering)
- [autocomplete](commands_cover_traffic.covertraffic.md#autocomplete)
- [execute](commands_cover_traffic.covertraffic.md#execute)
- [handleMessage](commands_cover_traffic.covertraffic.md#handlemessage)
- [help](commands_cover_traffic.covertraffic.md#help)
- [name](commands_cover_traffic.covertraffic.md#name)
- [stats](commands_cover_traffic.covertraffic.md#stats)
- [tick](commands_cover_traffic.covertraffic.md#tick)
- [usage](commands_cover_traffic.covertraffic.md#usage)

## Constructors

### constructor

\+ **new CoverTraffic**(`node`: *Hopr*): [*CoverTraffic*](commands_cover_traffic.covertraffic.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `node` | *Hopr* |

**Returns:** [*CoverTraffic*](commands_cover_traffic.covertraffic.md)

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/cover-traffic.ts:32](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L32)

## Properties

### hidden

• **hidden**: *boolean*= false

Inherited from: [AbstractCommand](commands_abstractcommand.abstractcommand.md).[hidden](commands_abstractcommand.abstractcommand.md#hidden)

Defined in: [commands/abstractCommand.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L15)

___

### identifier

• `Private` **identifier**: *string*

Defined in: [commands/cover-traffic.ts:32](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L32)

___

### messagesReceived

• `Private` **messagesReceived**: *number*

Defined in: [commands/cover-traffic.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L29)

___

### messagesSent

• `Private` **messagesSent**: *number*

Defined in: [commands/cover-traffic.ts:28](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L28)

___

### node

• **node**: *Hopr*

___

### registered

• `Private` **registered**: *boolean*

Defined in: [commands/cover-traffic.ts:26](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L26)

___

### seq

• `Private` **seq**: *number*= 0

Defined in: [commands/cover-traffic.ts:24](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L24)

___

### timeout

• `Private` **timeout**: *Timeout*

Defined in: [commands/cover-traffic.ts:25](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L25)

___

### totalLatency

• `Private` **totalLatency**: *number*

Defined in: [commands/cover-traffic.ts:30](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L30)

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

▸ **execute**(`query`: *string*): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `query` | *string* |

**Returns:** *Promise*<string\>

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/cover-traffic.ts:82](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L82)

___

### handleMessage

▸ `Private` **handleMessage**(`msg`: *Uint8Array*): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `msg` | *Uint8Array* |

**Returns:** *void*

Defined in: [commands/cover-traffic.ts:63](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L63)

___

### help

▸ **help**(): *string*

**Returns:** *string*

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/cover-traffic.ts:45](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L45)

___

### name

▸ **name**(): *string*

**Returns:** *string*

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/cover-traffic.ts:41](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L41)

___

### stats

▸ `Private` **stats**(): *string*

**Returns:** *string*

Defined in: [commands/cover-traffic.ts:73](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L73)

___

### tick

▸ `Private` **tick**(): *Promise*<void\>

**Returns:** *Promise*<void\>

Defined in: [commands/cover-traffic.ts:49](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L49)

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
