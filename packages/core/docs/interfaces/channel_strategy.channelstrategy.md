[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / [channel-strategy](../modules/channel_strategy.md) / ChannelStrategy

# Interface: ChannelStrategy

[channel-strategy](../modules/channel_strategy.md).ChannelStrategy

Staked nodes will likely want to automate opening and closing of channels. By
implementing the following interface, they can decide how to allocate their
stake to best attract traffic with a useful channel graph.

Implementors should bear in mind:

- Churn is expensive
- Path finding will prefer high stakes, and high availability of nodes.

## Implemented by

- [_PassiveStrategy_](../classes/channel_strategy.passivestrategy.md)
- [_PromiscuousStrategy_](../classes/channel_strategy.promiscuousstrategy.md)

## Table of contents

### Properties

- [name](channel_strategy.channelstrategy.md#name)

### Methods

- [tick](channel_strategy.channelstrategy.md#tick)

## Properties

### name

• **name**: _string_

Defined in: [packages/core/src/channel-strategy.ts:31](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/channel-strategy.ts#L31)

## Methods

### tick

▸ **tick**(`balance`: _BN_, `newChannels`: RoutingChannel[], `currentChannels`: RoutingChannel[], `networkPeers`: [_default_](../classes/network_network_peers.default.md), `getRandomChannel`: () => _Promise_<RoutingChannel\>): _Promise_<[[*ChannelsToOpen*](../modules/channel_strategy.md#channelstoopen)[], *PeerId*[]]\>

#### Parameters

| Name               | Type                                                     |
| :----------------- | :------------------------------------------------------- |
| `balance`          | _BN_                                                     |
| `newChannels`      | RoutingChannel[]                                         |
| `currentChannels`  | RoutingChannel[]                                         |
| `networkPeers`     | [_default_](../classes/network_network_peers.default.md) |
| `getRandomChannel` | () => _Promise_<RoutingChannel\>                         |

**Returns:** _Promise_<[[*ChannelsToOpen*](../modules/channel_strategy.md#channelstoopen)[], *PeerId*[]]\>

Defined in: [packages/core/src/channel-strategy.ts:33](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/channel-strategy.ts#L33)
