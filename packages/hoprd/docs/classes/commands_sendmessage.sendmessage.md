[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [commands/sendMessage](../modules/commands_sendmessage.md) / SendMessage

# Class: SendMessage

[commands/sendMessage](../modules/commands_sendmessage.md).SendMessage

## Hierarchy

- [*SendMessageBase*](commands_sendmessage.sendmessagebase.md)

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

\+ **new SendMessage**(`node`: *Hopr*): [*SendMessage*](commands_sendmessage.sendmessage.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `node` | *Hopr* |

**Returns:** [*SendMessage*](commands_sendmessage.sendmessage.md)

Inherited from: [SendMessageBase](commands_sendmessage.sendmessagebase.md)

Defined in: [commands/sendMessage.ts:8](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/sendMessage.ts#L8)

## Properties

### hidden

• **hidden**: *boolean*= false

Inherited from: [SendMessageBase](commands_sendmessage.sendmessagebase.md).[hidden](commands_sendmessage.sendmessagebase.md#hidden)

Defined in: [commands/abstractCommand.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L15)

___

### node

• **node**: *Hopr*

Inherited from: [SendMessageBase](commands_sendmessage.sendmessagebase.md).[node](commands_sendmessage.sendmessagebase.md#node)

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

Inherited from: [SendMessageBase](commands_sendmessage.sendmessagebase.md)

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

Inherited from: [SendMessageBase](commands_sendmessage.sendmessagebase.md)

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

Inherited from: [SendMessageBase](commands_sendmessage.sendmessagebase.md)

Defined in: [commands/sendMessage.ts:41](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/sendMessage.ts#L41)

___

### execute

▸ **execute**(`query`: *string*, `state`: [*GlobalState*](../modules/commands_abstractcommand.md#globalstate)): *Promise*<[*CommandResponse*](../modules/commands_abstractcommand.md#commandresponse)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `query` | *string* |
| `state` | [*GlobalState*](../modules/commands_abstractcommand.md#globalstate) |

**Returns:** *Promise*<[*CommandResponse*](../modules/commands_abstractcommand.md#commandresponse)\>

Overrides: [SendMessageBase](commands_sendmessage.sendmessagebase.md)

Defined in: [commands/sendMessage.ts:52](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/sendMessage.ts#L52)

___

### help

▸ **help**(): *string*

**Returns:** *string*

Inherited from: [SendMessageBase](commands_sendmessage.sendmessagebase.md)

Defined in: [commands/sendMessage.ts:17](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/sendMessage.ts#L17)

___

### name

▸ **name**(): *string*

**Returns:** *string*

Inherited from: [SendMessageBase](commands_sendmessage.sendmessagebase.md)

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

Inherited from: [SendMessageBase](commands_sendmessage.sendmessagebase.md)

Defined in: [commands/sendMessage.ts:26](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/sendMessage.ts#L26)

___

### usage

▸ `Protected` **usage**(`parameters`: *string*[]): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `parameters` | *string*[] |

**Returns:** *string*

Inherited from: [SendMessageBase](commands_sendmessage.sendmessagebase.md)

Defined in: [commands/abstractCommand.ts:49](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/abstractCommand.ts#L49)
