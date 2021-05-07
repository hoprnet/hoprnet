[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / [admin](../modules/admin.md) / AdminServer

# Class: AdminServer

[admin](../modules/admin.md).AdminServer

## Table of contents

### Constructors

- [constructor](admin.adminserver.md#constructor)

### Properties

- [app](admin.adminserver.md#app)
- [cmds](admin.adminserver.md#cmds)
- [node](admin.adminserver.md#node)
- [server](admin.adminserver.md#server)
- [wsServer](admin.adminserver.md#wsserver)

### Methods

- [registerNode](admin.adminserver.md#registernode)
- [setup](admin.adminserver.md#setup)

## Constructors

### constructor

\+ **new AdminServer**(`logs`: [_LogStream_](logs.logstream.md), `host`: _string_, `port`: _number_): [_AdminServer_](admin.adminserver.md)

#### Parameters

| Name   | Type                             |
| :----- | :------------------------------- |
| `logs` | [_LogStream_](logs.logstream.md) |
| `host` | _string_                         |
| `port` | _number_                         |

**Returns:** [_AdminServer_](admin.adminserver.md)

Defined in: [admin.ts:23](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/admin.ts#L23)

## Properties

### app

• `Private` **app**: _any_

Defined in: [admin.ts:19](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/admin.ts#L19)

---

### cmds

• `Private` **cmds**: _any_

Defined in: [admin.ts:23](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/admin.ts#L23)

---

### node

• `Private` **node**: _Hopr_

Defined in: [admin.ts:21](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/admin.ts#L21)

---

### server

• `Private` **server**: _Server_

Defined in: [admin.ts:20](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/admin.ts#L20)

---

### wsServer

• `Private` **wsServer**: _any_

Defined in: [admin.ts:22](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/admin.ts#L22)

## Methods

### registerNode

▸ **registerNode**(`node`: _Hopr_, `cmds`: _any_, `settings?`: _any_): _void_

#### Parameters

| Name        | Type   |
| :---------- | :----- |
| `node`      | _Hopr_ |
| `cmds`      | _any_  |
| `settings?` | _any_  |

**Returns:** _void_

Defined in: [admin.ts:81](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/admin.ts#L81)

---

### setup

▸ **setup**(): _Promise_<void\>

**Returns:** _Promise_<void\>

Defined in: [admin.ts:27](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/admin.ts#L27)
