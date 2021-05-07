[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [logs](../modules/logs.md) / LogStream

# Class: LogStream

[logs](../modules/logs.md).LogStream

## Table of contents

### Constructors

- [constructor](logs.logstream.md#constructor)

### Properties

- [connections](logs.logstream.md#connections)
- [messages](logs.logstream.md#messages)

### Methods

- [\_log](logs.logstream.md#_log)
- [\_sendMessage](logs.logstream.md#_sendmessage)
- [debug](logs.logstream.md#debug)
- [error](logs.logstream.md#error)
- [log](logs.logstream.md#log)
- [logConnectedPeers](logs.logstream.md#logconnectedpeers)
- [logFatalError](logs.logstream.md#logfatalerror)
- [logFullLine](logs.logstream.md#logfullline)
- [subscribe](logs.logstream.md#subscribe)
- [verbose](logs.logstream.md#verbose)
- [warn](logs.logstream.md#warn)

## Constructors

### constructor

\+ **new LogStream**(): [*LogStream*](logs.logstream.md)

**Returns:** [*LogStream*](logs.logstream.md)

Defined in: [logs.ts:20](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L20)

## Properties

### connections

• `Private` **connections**: *any*[]= []

Defined in: [logs.ts:20](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L20)

___

### messages

• `Private` **messages**: Message[]= []

Defined in: [logs.ts:19](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L19)

## Methods

### \_log

▸ **_log**(`msg`: Message): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `msg` | Message |

**Returns:** *void*

Defined in: [logs.ts:66](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L66)

___

### \_sendMessage

▸ **_sendMessage**(`m`: Message, `s`: *any*): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `m` | Message |
| `s` | *any* |

**Returns:** *void*

Defined in: [logs.ts:86](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L86)

___

### debug

▸ **debug**(`message`: *string*): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `message` | *string* |

**Returns:** *void*

Defined in: [logs.ts:48](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L48)

___

### error

▸ **error**(`message`: *string*, `trace`: *string*): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `message` | *string* |
| `trace` | *string* |

**Returns:** *void*

Defined in: [logs.ts:34](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L34)

___

### log

▸ **log**(...`args`: *string*[]): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `...args` | *string*[] |

**Returns:** *void*

Defined in: [logs.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L29)

___

### logConnectedPeers

▸ **logConnectedPeers**(`peers`: *string*[]): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `peers` | *string*[] |

**Returns:** *void*

Defined in: [logs.ts:61](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L61)

___

### logFatalError

▸ **logFatalError**(`message`: *string*): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `message` | *string* |

**Returns:** *void*

Defined in: [logs.ts:39](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L39)

___

### logFullLine

▸ **logFullLine**(...`args`: *string*[]): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `...args` | *string*[] |

**Returns:** *void*

Defined in: [logs.ts:56](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L56)

___

### subscribe

▸ **subscribe**(`sock`: *any*): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `sock` | *any* |

**Returns:** *void*

Defined in: [logs.ts:24](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L24)

___

### verbose

▸ **verbose**(`message`: *string*): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `message` | *string* |

**Returns:** *void*

Defined in: [logs.ts:52](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L52)

___

### warn

▸ **warn**(`message`: *string*): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `message` | *string* |

**Returns:** *void*

Defined in: [logs.ts:44](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L44)
