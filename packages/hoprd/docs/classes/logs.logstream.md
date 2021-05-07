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

\+ **new LogStream**(): [_LogStream_](logs.logstream.md)

**Returns:** [_LogStream_](logs.logstream.md)

Defined in: [logs.ts:20](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L20)

## Properties

### connections

• `Private` **connections**: _any_[]= []

Defined in: [logs.ts:20](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L20)

---

### messages

• `Private` **messages**: Message[]= []

Defined in: [logs.ts:19](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L19)

## Methods

### \_log

▸ **\_log**(`msg`: Message): _void_

#### Parameters

| Name  | Type    |
| :---- | :------ |
| `msg` | Message |

**Returns:** _void_

Defined in: [logs.ts:66](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L66)

---

### \_sendMessage

▸ **\_sendMessage**(`m`: Message, `s`: _any_): _void_

#### Parameters

| Name | Type    |
| :--- | :------ |
| `m`  | Message |
| `s`  | _any_   |

**Returns:** _void_

Defined in: [logs.ts:86](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L86)

---

### debug

▸ **debug**(`message`: _string_): _void_

#### Parameters

| Name      | Type     |
| :-------- | :------- |
| `message` | _string_ |

**Returns:** _void_

Defined in: [logs.ts:48](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L48)

---

### error

▸ **error**(`message`: _string_, `trace`: _string_): _void_

#### Parameters

| Name      | Type     |
| :-------- | :------- |
| `message` | _string_ |
| `trace`   | _string_ |

**Returns:** _void_

Defined in: [logs.ts:34](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L34)

---

### log

▸ **log**(...`args`: _string_[]): _void_

#### Parameters

| Name      | Type       |
| :-------- | :--------- |
| `...args` | _string_[] |

**Returns:** _void_

Defined in: [logs.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L29)

---

### logConnectedPeers

▸ **logConnectedPeers**(`peers`: _string_[]): _void_

#### Parameters

| Name    | Type       |
| :------ | :--------- |
| `peers` | _string_[] |

**Returns:** _void_

Defined in: [logs.ts:61](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L61)

---

### logFatalError

▸ **logFatalError**(`message`: _string_): _void_

#### Parameters

| Name      | Type     |
| :-------- | :------- |
| `message` | _string_ |

**Returns:** _void_

Defined in: [logs.ts:39](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L39)

---

### logFullLine

▸ **logFullLine**(...`args`: _string_[]): _void_

#### Parameters

| Name      | Type       |
| :-------- | :--------- |
| `...args` | _string_[] |

**Returns:** _void_

Defined in: [logs.ts:56](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L56)

---

### subscribe

▸ **subscribe**(`sock`: _any_): _void_

#### Parameters

| Name   | Type  |
| :----- | :---- |
| `sock` | _any_ |

**Returns:** _void_

Defined in: [logs.ts:24](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L24)

---

### verbose

▸ **verbose**(`message`: _string_): _void_

#### Parameters

| Name      | Type     |
| :-------- | :------- |
| `message` | _string_ |

**Returns:** _void_

Defined in: [logs.ts:52](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L52)

---

### warn

▸ **warn**(`message`: _string_): _void_

#### Parameters

| Name      | Type     |
| :-------- | :------- |
| `message` | _string_ |

**Returns:** _void_

Defined in: [logs.ts:44](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/logs.ts#L44)
