[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / [channel-strategy](../modules/channel_strategy.md) / PromiscuousStrategy

# Class: PromiscuousStrategy

[channel-strategy](../modules/channel_strategy.md).PromiscuousStrategy

## Implements

- [*ChannelStrategy*](../interfaces/channel_strategy.channelstrategy.md)

## Table of contents

### Constructors

- [constructor](channel_strategy.promiscuousstrategy.md#constructor)

### Properties

- [name](channel_strategy.promiscuousstrategy.md#name)

### Methods

- [tick](channel_strategy.promiscuousstrategy.md#tick)

## Constructors

### constructor

\+ **new PromiscuousStrategy**(): [*PromiscuousStrategy*](channel_strategy.promiscuousstrategy.md)

**Returns:** [*PromiscuousStrategy*](channel_strategy.promiscuousstrategy.md)

## Properties

### name

• **name**: *string*= 'promiscuous'

Implementation of: [ChannelStrategy](../interfaces/channel_strategy.channelstrategy.md).[name](../interfaces/channel_strategy.channelstrategy.md#name)

Defined in: [packages/core/src/channel-strategy.ts:63](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/channel-strategy.ts#L63)

## Methods

### tick

▸ **tick**(`balance`: *BN*, `_n`: RoutingChannel[], `currentChannels`: RoutingChannel[], `peers`: [*default*](network_network_peers.default.md), `getRandomChannel`: () => *Promise*<RoutingChannel\>): *Promise*<[[*ChannelsToOpen*](../modules/channel_strategy.md#channelstoopen)[], *PeerId*[]]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `balance` | *BN* |
| `_n` | RoutingChannel[] |
| `currentChannels` | RoutingChannel[] |
| `peers` | [*default*](network_network_peers.default.md) |
| `getRandomChannel` | () => *Promise*<RoutingChannel\> |

**Returns:** *Promise*<[[*ChannelsToOpen*](../modules/channel_strategy.md#channelstoopen)[], *PeerId*[]]\>

Implementation of: [ChannelStrategy](../interfaces/channel_strategy.channelstrategy.md)

Defined in: [packages/core/src/channel-strategy.ts:65](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/channel-strategy.ts#L65)
