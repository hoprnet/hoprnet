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

- [onChannelWillClose](ChannelStrategyInterface.md#onchannelwillclose)
- [onWinningTicket](ChannelStrategyInterface.md#onwinningticket)
- [shouldCommitToChannel](ChannelStrategyInterface.md#shouldcommittochannel)
- [tick](ChannelStrategyInterface.md#tick)

## Properties

### name

• **name**: `string`

#### Defined in

[packages/core/src/channel-strategy.ts:29](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L29)

___

### tickInterval

• **tickInterval**: `number`

#### Defined in

[packages/core/src/channel-strategy.ts:42](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L42)

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

[packages/core/src/channel-strategy.ts:38](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L38)

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

[packages/core/src/channel-strategy.ts:39](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L39)

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

[packages/core/src/channel-strategy.ts:40](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L40)

___

### tick

▸ **tick**(`balance`, `network_peer_ids`, `outgoing_channel`, `peer_quality`): [`StrategyTickResult`](../classes/StrategyTickResult.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `balance` | `BN` |
| `network_peer_ids` | `Iterator`<`string`, `any`, `undefined`\> |
| `outgoing_channel` | `OutgoingChannelStatus`[] |
| `peer_quality` | (`string`: `any`) => `number` |

#### Returns

[`StrategyTickResult`](../classes/StrategyTickResult.md)

#### Defined in

[packages/core/src/channel-strategy.ts:31](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L31)
