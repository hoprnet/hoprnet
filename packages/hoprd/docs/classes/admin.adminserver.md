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

\+ **new AdminServer**(`logs`: [*LogStream*](logs.logstream.md), `host`: *string*, `port`: *number*): [*AdminServer*](admin.adminserver.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `logs` | [*LogStream*](logs.logstream.md) |
| `host` | *string* |
| `port` | *number* |

**Returns:** [*AdminServer*](admin.adminserver.md)

Defined in: [admin.ts:23](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/admin.ts#L23)

## Properties

### app

• `Private` **app**: *any*

Defined in: [admin.ts:19](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/admin.ts#L19)

___

### cmds

• `Private` **cmds**: *any*

Defined in: [admin.ts:23](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/admin.ts#L23)

___

### node

• `Private` **node**: *Hopr*

Defined in: [admin.ts:21](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/admin.ts#L21)

___

### server

• `Private` **server**: *Server*

Defined in: [admin.ts:20](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/admin.ts#L20)

___

### wsServer

• `Private` **wsServer**: *any*

Defined in: [admin.ts:22](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/admin.ts#L22)

## Methods

### registerNode

▸ **registerNode**(`node`: *Hopr*, `cmds`: *any*, `settings?`: *any*): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `node` | *Hopr* |
| `cmds` | *any* |
| `settings?` | *any* |

**Returns:** *void*

Defined in: [admin.ts:81](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/admin.ts#L81)

___

### setup

▸ **setup**(): *Promise*<void\>

**Returns:** *Promise*<void\>

Defined in: [admin.ts:27](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/admin.ts#L27)
