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
- [onAckedTicket](ChannelStrategyInterface.md#onackedticket)
- [onChannelWillClose](ChannelStrategyInterface.md#onchannelwillclose)
- [shouldCommitToChannel](ChannelStrategyInterface.md#shouldcommittochannel)
- [tick](ChannelStrategyInterface.md#tick)

## Properties

### name

• **name**: `string`

#### Defined in

[packages/core/src/channel-strategy.ts:49](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L49)

___

### tickInterval

• **tickInterval**: `number`

#### Defined in

[packages/core/src/channel-strategy.ts:64](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L64)

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

[packages/core/src/channel-strategy.ts:51](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L51)

___

### onAckedTicket

▸ **onAckedTicket**(`t`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `t` | `AcknowledgedTicket` |

#### Returns

`Promise`<`void`\>

#### Defined in

[packages/core/src/channel-strategy.ts:61](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L61)

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

[packages/core/src/channel-strategy.ts:60](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L60)

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

[packages/core/src/channel-strategy.ts:62](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L62)

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

[packages/core/src/channel-strategy.ts:53](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L53)
