[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / ChannelStrategyInterface

# Interface: ChannelStrategyInterface

Staked nodes will likely want to automate opening and closing of channels. By
implementing the following interface, they can decide how to allocate their
stake to best attract traffic with a useful channel graph.

Implementors should bear in mind:
- Churn is expensive
- Path finding will prefer high stakes, and high availability of nodes.

## Table of contents

### Properties

- [name](ChannelStrategyInterface.md#name)
- [tickInterval](ChannelStrategyInterface.md#tickinterval)

### Methods

- [configure](ChannelStrategyInterface.md#configure)
- [onChannelWillClose](ChannelStrategyInterface.md#onchannelwillclose)
- [onWinningTicket](ChannelStrategyInterface.md#onwinningticket)
- [shouldCommitToChannel](ChannelStrategyInterface.md#shouldcommittochannel)
- [tick](ChannelStrategyInterface.md#tick)

## Properties

### name

• **name**: `string`

#### Defined in

[packages/core/src/channel-strategy.ts:50](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L50)

___

### tickInterval

• **tickInterval**: `number`

#### Defined in

[packages/core/src/channel-strategy.ts:65](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L65)

## Methods

### configure

▸ **configure**(`settings`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `settings` | `any` |

#### Returns

`void`

#### Defined in

[packages/core/src/channel-strategy.ts:52](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L52)

___

### onChannelWillClose

▸ **onChannelWillClose**(`channel`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channel` | `ChannelEntry` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/channel-strategy.ts:61](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L61)

___

### onWinningTicket

▸ **onWinningTicket**(`t`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `t` | `AcknowledgedTicket` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/channel-strategy.ts:62](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L62)

___

### shouldCommitToChannel

▸ **shouldCommitToChannel**(`c`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `c` | `ChannelEntry` |

#### Returns

`boolean`

#### Defined in

[packages/core/src/channel-strategy.ts:63](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L63)

___

### tick

▸ **tick**(`balance`, `network_peer_ids`, `outgoing_channel`, `peer_quality`): [`StrategyTickResult`](../classes/StrategyTickResult.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `balance` | `BN` |
| `network_peer_ids` | `Iterator`<`string`, `any`, `undefined`\> |
| `outgoing_channel` | `OutgoingChannelStatus`[] |
| `peer_quality` | (`string`: `string`) => `number` |

#### Returns

[`StrategyTickResult`](../classes/StrategyTickResult.md)

#### Defined in

[packages/core/src/channel-strategy.ts:54](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L54)
