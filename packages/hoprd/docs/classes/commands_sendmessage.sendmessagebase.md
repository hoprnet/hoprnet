[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [commands/sendMessage](../modules/commands_sendmessage.md) / SendMessageBase

# Class: SendMessageBase

[commands/sendMessage](../modules/commands_sendmessage.md).SendMessageBase

## Hierarchy

- [*AbstractCommand*](commands_abstractcommand.abstractcommand.md)

  ↳ **SendMessageBase**

  ↳↳ [*SendMessage*](commands_sendmessage.sendmessage.md)

## Table of contents

### Constructors

- [constructor](commands_sendmessage.sendmessagebase.md#constructor)

### Properties

- [hidden](commands_sendmessage.sendmessagebase.md#hidden)
- [node](commands_sendmessage.sendmessagebase.md#node)

### Methods

- [\_assertUsage](commands_sendmessage.sendmessagebase.md#_assertusage)
- [\_autocompleteByFiltering](commands_sendmessage.sendmessagebase.md#_autocompletebyfiltering)
- [autocomplete](commands_sendmessage.sendmessagebase.md#autocomplete)
- [execute](commands_sendmessage.sendmessagebase.md#execute)
- [help](commands_sendmessage.sendmessagebase.md#help)
- [insertMyAddress](commands_sendmessage.sendmessagebase.md#insertmyaddress)
- [name](commands_sendmessage.sendmessagebase.md#name)
- [sendMessage](commands_sendmessage.sendmessagebase.md#sendmessage)
- [usage](commands_sendmessage.sendmessagebase.md#usage)

## Constructors

### constructor

\+ **new SendMessageBase**(`node`: *Hopr*): [*SendMessageBase*](commands_sendmessage.sendmessagebase.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `node` | *Hopr* |

**Returns:** [*SendMessageBase*](commands_sendmessage.sendmessagebase.md)

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/sendMessage.ts:8](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/sendMessage.ts#L8)

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

▸ **autocomplete**(`query?`: *string*, `line?`: *string*, `state`: [*GlobalState*](../modules/commands_abstractcommand.md#globalstate)): *Promise*<[*AutoCompleteResult*](../modules/commands_abstractcommand.md#autocompleteresult)\>

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `query` | *string* | '' |
| `line` | *string* | '' |
| `state` | [*GlobalState*](../modules/commands_abstractcommand.md#globalstate) | - |

**Returns:** *Promise*<[*AutoCompleteResult*](../modules/commands_abstractcommand.md#autocompleteresult)\>

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/sendMessage.ts:41](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/sendMessage.ts#L41)

___

### execute

▸ `Abstract` **execute**(`query`: *string*, `state`: [*GlobalState*](../modules/commands_abstractcommand.md#globalstate)): [*CommandResponse*](../modules/commands_abstractcommand.md#commandresponse) \| *Promise*<[*CommandResponse*](../modules/commands_abstractcommand.md#commandresponse)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `query` | *string* |
| `state` | [*GlobalState*](../modules/commands_abstractcommand.md#globalstate) |

**Returns:** [*CommandResponse*](../modules/commands_abstractcommand.md#commandresponse) \| *Promise*<[*CommandResponse*](../modules/commands_abstractcommand.md#commandresponse)\>

Inherited from: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/abstractCommand.ts:24](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L24)

___

### help

▸ **help**(): *string*

**Returns:** *string*

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/sendMessage.ts:17](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/sendMessage.ts#L17)

___

### insertMyAddress

▸ `Private` **insertMyAddress**(`message`: *string*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `message` | *string* |

**Returns:** *string*

Defined in: [commands/sendMessage.ts:21](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/sendMessage.ts#L21)

___

### name

▸ **name**(): *string*

**Returns:** *string*

Overrides: [AbstractCommand](commands_abstractcommand.abstractcommand.md)

Defined in: [commands/sendMessage.ts:13](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/sendMessage.ts#L13)

___

### sendMessage

▸ `Protected` **sendMessage**(`state`: [*GlobalState*](../modules/commands_abstractcommand.md#globalstate), `recipient`: *PeerId*, `rawMessage`: *string*, `getIntermediateNodes?`: () => *Promise*<PeerId[]\>): *Promise*<string \| void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `state` | [*GlobalState*](../modules/commands_abstractcommand.md#globalstate) |
| `recipient` | *PeerId* |
| `rawMessage` | *string* |
| `getIntermediateNodes?` | () => *Promise*<PeerId[]\> |

**Returns:** *Promise*<string \| void\>

Defined in: [commands/sendMessage.ts:26](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/sendMessage.ts#L26)

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
