[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [commands/sendMessage](../modules/commands_sendmessage.md) / SendMessage

# Class: SendMessage

[commands/sendMessage](../modules/commands_sendmessage.md).SendMessage

## Hierarchy

- [_SendMessageBase_](commands_sendmessage.sendmessagebase.md)

  ↳ **SendMessage**

## Table of contents

### Constructors

- [constructor](commands_sendmessage.sendmessage.md#constructor)

### Properties

- [hidden](commands_sendmessage.sendmessage.md#hidden)
- [node](commands_sendmessage.sendmessage.md#node)

### Methods

- [\_assertUsage](commands_sendmessage.sendmessage.md#_assertusage)
- [\_autocompleteByFiltering](commands_sendmessage.sendmessage.md#_autocompletebyfiltering)
- [autocomplete](commands_sendmessage.sendmessage.md#autocomplete)
- [execute](commands_sendmessage.sendmessage.md#execute)
- [help](commands_sendmessage.sendmessage.md#help)
- [name](commands_sendmessage.sendmessage.md#name)
- [sendMessage](commands_sendmessage.sendmessage.md#sendmessage)
- [usage](commands_sendmessage.sendmessage.md#usage)

## Constructors

### constructor

\+ **new SendMessage**(`node`: _Hopr_): [_SendMessage_](commands_sendmessage.sendmessage.md)

#### Parameters

| Name   | Type   |
| :----- | :----- |
| `node` | _Hopr_ |

**Returns:** [_SendMessage_](commands_sendmessage.sendmessage.md)

Inherited from: [SendMessageBase](commands_sendmessage.sendmessagebase.md)

Defined in: [commands/sendMessage.ts:8](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/sendMessage.ts#L8)

## Properties

### hidden

• **hidden**: _boolean_= false

Inherited from: [SendMessageBase](commands_sendmessage.sendmessagebase.md).[hidden](commands_sendmessage.sendmessagebase.md#hidden)

Defined in: [commands/abstractCommand.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L15)

---

### node

• **node**: _Hopr_

Inherited from: [SendMessageBase](commands_sendmessage.sendmessagebase.md).[node](commands_sendmessage.sendmessagebase.md#node)

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

Inherited from: [SendMessageBase](commands_sendmessage.sendmessagebase.md)

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

Inherited from: [SendMessageBase](commands_sendmessage.sendmessagebase.md)

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

Inherited from: [SendMessageBase](commands_sendmessage.sendmessagebase.md)

Defined in: [commands/sendMessage.ts:41](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/sendMessage.ts#L41)

---

### execute

▸ **execute**(`query`: _string_, `state`: [_GlobalState_](../modules/commands_abstractcommand.md#globalstate)): _Promise_<[_CommandResponse_](../modules/commands_abstractcommand.md#commandresponse)\>

#### Parameters

| Name    | Type                                                                |
| :------ | :------------------------------------------------------------------ |
| `query` | _string_                                                            |
| `state` | [_GlobalState_](../modules/commands_abstractcommand.md#globalstate) |

**Returns:** _Promise_<[_CommandResponse_](../modules/commands_abstractcommand.md#commandresponse)\>

Overrides: [SendMessageBase](commands_sendmessage.sendmessagebase.md)

Defined in: [commands/sendMessage.ts:52](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/sendMessage.ts#L52)

---

### help

▸ **help**(): _string_

**Returns:** _string_

Inherited from: [SendMessageBase](commands_sendmessage.sendmessagebase.md)

Defined in: [commands/sendMessage.ts:17](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/sendMessage.ts#L17)

---

### name

▸ **name**(): _string_

**Returns:** _string_

Inherited from: [SendMessageBase](commands_sendmessage.sendmessagebase.md)

Defined in: [commands/sendMessage.ts:13](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/sendMessage.ts#L13)

---

### sendMessage

▸ `Protected` **sendMessage**(`state`: [_GlobalState_](../modules/commands_abstractcommand.md#globalstate), `recipient`: _PeerId_, `rawMessage`: _string_, `getIntermediateNodes?`: () => _Promise_<PeerId[]\>): _Promise_<string \| void\>

#### Parameters

| Name                    | Type                                                                |
| :---------------------- | :------------------------------------------------------------------ |
| `state`                 | [_GlobalState_](../modules/commands_abstractcommand.md#globalstate) |
| `recipient`             | _PeerId_                                                            |
| `rawMessage`            | _string_                                                            |
| `getIntermediateNodes?` | () => _Promise_<PeerId[]\>                                          |

**Returns:** _Promise_<string \| void\>

Inherited from: [SendMessageBase](commands_sendmessage.sendmessagebase.md)

Defined in: [commands/sendMessage.ts:26](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/sendMessage.ts#L26)

---

### usage

▸ `Protected` **usage**(`parameters`: _string_[]): _string_

#### Parameters

| Name         | Type       |
| :----------- | :--------- |
| `parameters` | _string_[] |

**Returns:** _string_

Inherited from: [SendMessageBase](commands_sendmessage.sendmessagebase.md)

Defined in: [commands/abstractCommand.ts:49](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L49)
