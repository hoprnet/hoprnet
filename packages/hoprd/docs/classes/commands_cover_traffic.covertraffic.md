[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [commands/cover-traffic](../modules/commands_cover_traffic.md) / CoverTraffic

# Class: CoverTraffic

[commands/cover-traffic](../modules/commands_cover_traffic.md).CoverTraffic

## Hierarchy

- [_AbstractCommand_](commands_abstractcommand.abstractcommand.md)

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

\+ **new CoverTraffic**(`node`: _Hopr_): [_CoverTraffic_](commands_cover_traffic.covertraffic.md)

#### Parameters

| Name   | Type   |
| :----- | :----- |
| `node` | _Hopr_ |

**Returns:** [_CoverTraffic_](commands_cover_traffic.covertraffic.md)

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/cover-traffic.ts:32](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L32)

## Properties

### hidden

• **hidden**: _boolean_= false

Inherited from: [AbstractCommand](commands_abstractcommand.abstractcommand.md).[hidden](commands_abstractcommand.abstractcommand.md#hidden)

Defined in: [commands/abstractCommand.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L15)

---

### identifier

• `Private` **identifier**: _string_

Defined in: [commands/cover-traffic.ts:32](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L32)

---

### messagesReceived

• `Private` **messagesReceived**: _number_

Defined in: [commands/cover-traffic.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L29)

---

### messagesSent

• `Private` **messagesSent**: _number_

Defined in: [commands/cover-traffic.ts:28](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L28)

---

### node

• **node**: _Hopr_

---

### registered

• `Private` **registered**: _boolean_

Defined in: [commands/cover-traffic.ts:26](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L26)

---

### seq

• `Private` **seq**: _number_= 0

Defined in: [commands/cover-traffic.ts:24](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L24)

---

### timeout

• `Private` **timeout**: _Timeout_

Defined in: [commands/cover-traffic.ts:25](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L25)

---

### totalLatency

• `Private` **totalLatency**: _number_

Defined in: [commands/cover-traffic.ts:30](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L30)

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

▸ **execute**(`query`: _string_): _Promise_<string\>

#### Parameters

| Name    | Type     |
| :------ | :------- |
| `query` | _string_ |

**Returns:** _Promise_<string\>

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/cover-traffic.ts:82](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L82)

---

### handleMessage

▸ `Private` **handleMessage**(`msg`: _Uint8Array_): _void_

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `msg` | _Uint8Array_ |

**Returns:** _void_

Defined in: [commands/cover-traffic.ts:63](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L63)

---

### help

▸ **help**(): _string_

**Returns:** _string_

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/cover-traffic.ts:45](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L45)

---

### name

▸ **name**(): _string_

**Returns:** _string_

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/cover-traffic.ts:41](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L41)

---

### stats

▸ `Private` **stats**(): _string_

**Returns:** _string_

Defined in: [commands/cover-traffic.ts:73](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L73)

---

### tick

▸ `Private` **tick**(): _Promise_<void\>

**Returns:** _Promise_<void\>

Defined in: [commands/cover-traffic.ts:49](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/cover-traffic.ts#L49)

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
