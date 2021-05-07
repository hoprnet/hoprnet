[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / [network/heartbeat](../modules/network_heartbeat.md) / default

# Class: default

[network/heartbeat](../modules/network_heartbeat.md).default

## Table of contents

### Constructors

- [constructor](network_heartbeat.default.md#constructor)

### Properties

- [timeout](network_heartbeat.default.md#timeout)

### Methods

- [\_\_forTestOnly_checkNodes](network_heartbeat.default.md#__fortestonly_checknodes)
- [checkNodes](network_heartbeat.default.md#checknodes)
- [handleHeartbeatRequest](network_heartbeat.default.md#handleheartbeatrequest)
- [pingNode](network_heartbeat.default.md#pingnode)
- [start](network_heartbeat.default.md#start)
- [stop](network_heartbeat.default.md#stop)
- [tick](network_heartbeat.default.md#tick)

## Constructors

### constructor

\+ **new default**(`networkPeers`: [_default_](network_network_peers.default.md), `subscribe`: (`protocol`: _string_, `handler`: LibP2PHandlerFunction, `includeReply`: _boolean_) => _void_, `sendMessageAndExpectResponse`: (`dst`: _PeerId_, `proto`: _string_, `msg`: _Uint8Array_, `opts`: DialOpts) => _Promise_<Uint8Array\>, `hangUp`: (`addr`: _PeerId_) => _Promise_<void\>): [_default_](network_heartbeat.default.md)

#### Parameters

| Name                           | Type                                                                                                  |
| :----------------------------- | :---------------------------------------------------------------------------------------------------- |
| `networkPeers`                 | [_default_](network_network_peers.default.md)                                                         |
| `subscribe`                    | (`protocol`: _string_, `handler`: LibP2PHandlerFunction, `includeReply`: _boolean_) => _void_         |
| `sendMessageAndExpectResponse` | (`dst`: _PeerId_, `proto`: _string_, `msg`: _Uint8Array_, `opts`: DialOpts) => _Promise_<Uint8Array\> |
| `hangUp`                       | (`addr`: _PeerId_) => _Promise_<void\>                                                                |

**Returns:** [_default_](network_heartbeat.default.md)

Defined in: [packages/core/src/network/heartbeat.ts:13](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/heartbeat.ts#L13)

## Properties

### timeout

• `Private` **timeout**: _Timeout_

Defined in: [packages/core/src/network/heartbeat.ts:13](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/heartbeat.ts#L13)

## Methods

### \_\_forTestOnly_checkNodes

▸ **\_\_forTestOnly_checkNodes**(): _Promise_<void\>

**Returns:** _Promise_<void\>

Defined in: [packages/core/src/network/heartbeat.ts:91](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/heartbeat.ts#L91)

---

### checkNodes

▸ `Private` **checkNodes**(): _Promise_<void\>

**Returns:** _Promise_<void\>

Defined in: [packages/core/src/network/heartbeat.ts:60](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/heartbeat.ts#L60)

---

### handleHeartbeatRequest

▸ **handleHeartbeatRequest**(`msg`: _Uint8Array_, `remotePeer`: _PeerId_): _Uint8Array_

#### Parameters

| Name         | Type         |
| :----------- | :----------- |
| `msg`        | _Uint8Array_ |
| `remotePeer` | _PeerId_     |

**Returns:** _Uint8Array_

Defined in: [packages/core/src/network/heartbeat.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/heartbeat.ts#L29)

---

### pingNode

▸ **pingNode**(`id`: _PeerId_): _Promise_<boolean\>

#### Parameters

| Name | Type     |
| :--- | :------- |
| `id` | _PeerId_ |

**Returns:** _Promise_<boolean\>

Defined in: [packages/core/src/network/heartbeat.ts:35](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/heartbeat.ts#L35)

---

### start

▸ **start**(): _void_

**Returns:** _void_

Defined in: [packages/core/src/network/heartbeat.ts:81](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/heartbeat.ts#L81)

---

### stop

▸ **stop**(): _void_

**Returns:** _void_

Defined in: [packages/core/src/network/heartbeat.ts:86](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/heartbeat.ts#L86)

---

### tick

▸ `Private` **tick**(): _void_

**Returns:** _void_

Defined in: [packages/core/src/network/heartbeat.ts:74](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/network/heartbeat.ts#L74)
