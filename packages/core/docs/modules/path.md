[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / path

# Module: path

## Table of contents

### Type aliases

- [Path](path.md#path)

### Functions

- [findPath](path.md#findpath)

## Type aliases

### Path

Ƭ **Path**: PeerId[]

Defined in: [packages/core/src/path/index.ts:10](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/path/index.ts#L10)

## Functions

### findPath

▸ **findPath**(`start`: PeerId, `destination`: PeerId, `hops`: _number_, `networkPeers`: [_default_](../classes/network_network_peers.default.md), `getChannelsFromPeer`: (`p`: PeerId) => _Promise_<Edge[]\>, `randomness`: _number_): _Promise_<[_Path_](path.md#path)\>

Find a path through the payment channels.

#### Parameters

| Name                  | Type                                                     |
| :-------------------- | :------------------------------------------------------- |
| `start`               | PeerId                                                   |
| `destination`         | PeerId                                                   |
| `hops`                | _number_                                                 |
| `networkPeers`        | [_default_](../classes/network_network_peers.default.md) |
| `getChannelsFromPeer` | (`p`: PeerId) => _Promise_<Edge[]\>                      |
| `randomness`          | _number_                                                 |

**Returns:** _Promise_<[_Path_](path.md#path)\>

path as Array<PeerId> (including start, but not including
destination

Defined in: [packages/core/src/path/index.ts:30](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/path/index.ts#L30)
