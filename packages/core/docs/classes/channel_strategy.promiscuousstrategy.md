[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / [channel-strategy](../modules/channel_strategy.md) / PromiscuousStrategy

# Class: PromiscuousStrategy

[channel-strategy](../modules/channel_strategy.md).PromiscuousStrategy

## Implements

- [_ChannelStrategy_](../interfaces/channel_strategy.channelstrategy.md)

## Table of contents

### Constructors

- [constructor](channel_strategy.promiscuousstrategy.md#constructor)

### Properties

- [name](channel_strategy.promiscuousstrategy.md#name)

### Methods

- [tick](channel_strategy.promiscuousstrategy.md#tick)

## Constructors

### constructor

\+ **new PromiscuousStrategy**(): [_PromiscuousStrategy_](channel_strategy.promiscuousstrategy.md)

**Returns:** [_PromiscuousStrategy_](channel_strategy.promiscuousstrategy.md)

## Properties

### name

• **name**: _string_= 'promiscuous'

Implementation of: [ChannelStrategy](../interfaces/channel_strategy.channelstrategy.md).[name](../interfaces/channel_strategy.channelstrategy.md#name)

Defined in: [packages/core/src/channel-strategy.ts:63](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/channel-strategy.ts#L63)

## Methods

### tick

▸ **tick**(`balance`: _BN_, `_n`: RoutingChannel[], `currentChannels`: RoutingChannel[], `peers`: [_default_](network_network_peers.default.md), `getRandomChannel`: () => _Promise_<RoutingChannel\>): _Promise_<[[*ChannelsToOpen*](../modules/channel_strategy.md#channelstoopen)[], *PeerId*[]]\>

#### Parameters

| Name               | Type                                          |
| :----------------- | :-------------------------------------------- |
| `balance`          | _BN_                                          |
| `_n`               | RoutingChannel[]                              |
| `currentChannels`  | RoutingChannel[]                              |
| `peers`            | [_default_](network_network_peers.default.md) |
| `getRandomChannel` | () => _Promise_<RoutingChannel\>              |

**Returns:** _Promise_<[[*ChannelsToOpen*](../modules/channel_strategy.md#channelstoopen)[], *PeerId*[]]\>

Implementation of: [ChannelStrategy](../interfaces/channel_strategy.channelstrategy.md)

Defined in: [packages/core/src/channel-strategy.ts:65](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/channel-strategy.ts#L65)
