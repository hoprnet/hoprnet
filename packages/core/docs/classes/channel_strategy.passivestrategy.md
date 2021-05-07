[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / [channel-strategy](../modules/channel_strategy.md) / PassiveStrategy

# Class: PassiveStrategy

[channel-strategy](../modules/channel_strategy.md).PassiveStrategy

## Implements

- [_ChannelStrategy_](../interfaces/channel_strategy.channelstrategy.md)

## Table of contents

### Constructors

- [constructor](channel_strategy.passivestrategy.md#constructor)

### Properties

- [name](channel_strategy.passivestrategy.md#name)

### Methods

- [tick](channel_strategy.passivestrategy.md#tick)

## Constructors

### constructor

\+ **new PassiveStrategy**(): [_PassiveStrategy_](channel_strategy.passivestrategy.md)

**Returns:** [_PassiveStrategy_](channel_strategy.passivestrategy.md)

## Properties

### name

• **name**: _string_= 'passive'

Implementation of: [ChannelStrategy](../interfaces/channel_strategy.channelstrategy.md).[name](../interfaces/channel_strategy.channelstrategy.md#name)

Defined in: [packages/core/src/channel-strategy.ts:49](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/channel-strategy.ts#L49)

## Methods

### tick

▸ **tick**(`_balance`: _BN_, `_n`: RoutingChannel[], `_c`: RoutingChannel[], `_p`: [_default_](network_network_peers.default.md)): _Promise_<[[*ChannelsToOpen*](../modules/channel_strategy.md#channelstoopen)[], *PeerId*[]]\>

#### Parameters

| Name       | Type                                          |
| :--------- | :-------------------------------------------- |
| `_balance` | _BN_                                          |
| `_n`       | RoutingChannel[]                              |
| `_c`       | RoutingChannel[]                              |
| `_p`       | [_default_](network_network_peers.default.md) |

**Returns:** _Promise_<[[*ChannelsToOpen*](../modules/channel_strategy.md#channelstoopen)[], *PeerId*[]]\>

Implementation of: [ChannelStrategy](../interfaces/channel_strategy.channelstrategy.md)

Defined in: [packages/core/src/channel-strategy.ts:51](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/channel-strategy.ts#L51)
