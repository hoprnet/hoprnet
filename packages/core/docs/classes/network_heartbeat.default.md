[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / [network/heartbeat](../modules/network_heartbeat.md) / default

# Class: default

[network/heartbeat](../modules/network_heartbeat.md).default

## Table of contents

### Constructors

- [constructor](network_heartbeat.default.md#constructor)

### Properties

- [timeout](network_heartbeat.default.md#timeout)

### Methods

- [\_\_forTestOnly\_checkNodes](network_heartbeat.default.md#__fortestonly_checknodes)
- [checkNodes](network_heartbeat.default.md#checknodes)
- [handleHeartbeatRequest](network_heartbeat.default.md#handleheartbeatrequest)
- [pingNode](network_heartbeat.default.md#pingnode)
- [start](network_heartbeat.default.md#start)
- [stop](network_heartbeat.default.md#stop)
- [tick](network_heartbeat.default.md#tick)

## Constructors

### constructor

\+ **new default**(`networkPeers`: [*default*](network_network_peers.default.md), `subscribe`: (`protocol`: *string*, `handler`: LibP2PHandlerFunction, `includeReply`: *boolean*) => *void*, `sendMessageAndExpectResponse`: (`dst`: *PeerId*, `proto`: *string*, `msg`: *Uint8Array*, `opts`: DialOpts) => *Promise*<Uint8Array\>, `hangUp`: (`addr`: *PeerId*) => *Promise*<void\>): [*default*](network_heartbeat.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `networkPeers` | [*default*](network_network_peers.default.md) |
| `subscribe` | (`protocol`: *string*, `handler`: LibP2PHandlerFunction, `includeReply`: *boolean*) => *void* |
| `sendMessageAndExpectResponse` | (`dst`: *PeerId*, `proto`: *string*, `msg`: *Uint8Array*, `opts`: DialOpts) => *Promise*<Uint8Array\> |
| `hangUp` | (`addr`: *PeerId*) => *Promise*<void\> |

**Returns:** [*default*](network_heartbeat.default.md)

Defined in: [packages/core/src/network/heartbeat.ts:13](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/heartbeat.ts#L13)

## Properties

### timeout

• `Private` **timeout**: *Timeout*

Defined in: [packages/core/src/network/heartbeat.ts:13](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/heartbeat.ts#L13)

## Methods

### \_\_forTestOnly\_checkNodes

▸ **__forTestOnly_checkNodes**(): *Promise*<void\>

**Returns:** *Promise*<void\>

Defined in: [packages/core/src/network/heartbeat.ts:91](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/heartbeat.ts#L91)

___

### checkNodes

▸ `Private` **checkNodes**(): *Promise*<void\>

**Returns:** *Promise*<void\>

Defined in: [packages/core/src/network/heartbeat.ts:60](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/heartbeat.ts#L60)

___

### handleHeartbeatRequest

▸ **handleHeartbeatRequest**(`msg`: *Uint8Array*, `remotePeer`: *PeerId*): *Uint8Array*

#### Parameters

| Name | Type |
| :------ | :------ |
| `msg` | *Uint8Array* |
| `remotePeer` | *PeerId* |

**Returns:** *Uint8Array*

Defined in: [packages/core/src/network/heartbeat.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/heartbeat.ts#L29)

___

### pingNode

▸ **pingNode**(`id`: *PeerId*): *Promise*<boolean\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `id` | *PeerId* |

**Returns:** *Promise*<boolean\>

Defined in: [packages/core/src/network/heartbeat.ts:35](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/heartbeat.ts#L35)

___

### start

▸ **start**(): *void*

**Returns:** *void*

Defined in: [packages/core/src/network/heartbeat.ts:81](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/heartbeat.ts#L81)

___

### stop

▸ **stop**(): *void*

**Returns:** *void*

Defined in: [packages/core/src/network/heartbeat.ts:86](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/heartbeat.ts#L86)

___

### tick

▸ `Private` **tick**(): *void*

**Returns:** *void*

Defined in: [packages/core/src/network/heartbeat.ts:74](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/heartbeat.ts#L74)
