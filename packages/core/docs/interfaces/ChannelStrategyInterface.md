[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / ChannelStrategyInterface

# Interface: ChannelStrategyInterface

Staked nodes will likely want to automate opening and closing of channels. By
implementing the following interface, they can decide how to allocate their
stake to best attract traffic with a useful channel graph.

Implementors should bear in mind:
- Churn is expensive
- Path finding will prefer high stakes, and high availability of nodes.

## Implemented by

- [`PassiveStrategy`](../classes/PassiveStrategy.md)
- [`PromiscuousStrategy`](../classes/PromiscuousStrategy.md)

## Table of contents

### Properties

- [name](ChannelStrategyInterface.md#name)
- [tickInterval](ChannelStrategyInterface.md#tickinterval)

### Methods

- [onChannelWillClose](ChannelStrategyInterface.md#onchannelwillclose)
- [onWinningTicket](ChannelStrategyInterface.md#onwinningticket)
- [shouldCommitToChannel](ChannelStrategyInterface.md#shouldcommittochannel)
- [tick](ChannelStrategyInterface.md#tick)

## Properties

### name

• **name**: `string`

#### Defined in

[packages/core/src/channel-strategy.ts:37](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L37)

___

### tickInterval

• **tickInterval**: `number`

#### Defined in

[packages/core/src/channel-strategy.ts:51](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L51)

## Methods

### onChannelWillClose

▸ **onChannelWillClose**(`channel`, `chain`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channel` | `ChannelEntry` |
| `chain` | `default` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/channel-strategy.ts:47](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L47)

___

### onWinningTicket

▸ **onWinningTicket**(`t`, `chain`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `t` | `AcknowledgedTicket` |
| `chain` | `default` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/channel-strategy.ts:48](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L48)

___

### shouldCommitToChannel

▸ **shouldCommitToChannel**(`c`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `c` | `ChannelEntry` |

#### Returns

`Promise`<`boolean`\>

#### Defined in

[packages/core/src/channel-strategy.ts:49](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L49)

___

### tick

▸ **tick**(`balance`, `currentChannels`, `networkPeers`, `getRandomChannel`): `Promise`<[`StrategyTickResult`](../modules.md#strategytickresult)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `balance` | `BN` |
| `currentChannels` | `ChannelEntry`[] |
| `networkPeers` | `NetworkPeers` |
| `getRandomChannel` | () => `Promise`<`ChannelEntry`\> |

#### Returns

`Promise`<[`StrategyTickResult`](../modules.md#strategytickresult)\>

#### Defined in

[packages/core/src/channel-strategy.ts:39](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L39)
