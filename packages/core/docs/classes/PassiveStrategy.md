[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / PassiveStrategy

# Class: PassiveStrategy

## Hierarchy

- `RustStrategyWrapper`<`RS_PromiscuousStrategy`\>

  ↳ **`PassiveStrategy`**

## Table of contents

### Constructors

- [constructor](PassiveStrategy.md#constructor)

### Properties

- [tickInterval](PassiveStrategy.md#tickinterval)

### Accessors

- [name](PassiveStrategy.md#name)

### Methods

- [onChannelWillClose](PassiveStrategy.md#onchannelwillclose)
- [onWinningTicket](PassiveStrategy.md#onwinningticket)
- [shouldCommitToChannel](PassiveStrategy.md#shouldcommittochannel)
- [tick](PassiveStrategy.md#tick)

## Constructors

### constructor

• **new PassiveStrategy**()

#### Overrides

RustStrategyWrapper&lt;RS\_PromiscuousStrategy\&gt;.constructor

#### Defined in

[packages/core/src/channel-strategy.ts:107](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L107)

## Properties

### tickInterval

• **tickInterval**: `number` = `CHECK_TIMEOUT`

#### Inherited from

RustStrategyWrapper.tickInterval

#### Defined in

[packages/core/src/channel-strategy.ts:75](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L75)

## Accessors

### name

• `get` **name**(): `any`

#### Returns

`any`

#### Inherited from

RustStrategyWrapper.name

#### Defined in

[packages/core/src/channel-strategy.ts:95](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L95)

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

#### Inherited from

RustStrategyWrapper.onChannelWillClose

#### Defined in

[packages/core/src/channel-strategy.ts:57](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L57)

___

### onWinningTicket

▸ **onWinningTicket**(`ackTicket`, `chain`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ackTicket` | `AcknowledgedTicket` |
| `chain` | `default` |

#### Returns

`Promise`<`void`\>

#### Inherited from

RustStrategyWrapper.onWinningTicket

#### Defined in

[packages/core/src/channel-strategy.ts:51](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L51)

___

### shouldCommitToChannel

▸ **shouldCommitToChannel**(`c`): `boolean`

#### Parameters

| Name | Type |
| :------ | :------ |
| `c` | `ChannelEntry` |

#### Returns

`boolean`

#### Inherited from

RustStrategyWrapper.shouldCommitToChannel

#### Defined in

[packages/core/src/channel-strategy.ts:70](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L70)

___

### tick

▸ **tick**(`balance`, `network_peer_ids`, `outgoing_channels`, `peer_quality`): [`StrategyTickResult`](StrategyTickResult.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `balance` | `BN` |
| `network_peer_ids` | `Iterator`<`string`, `any`, `undefined`\> |
| `outgoing_channels` | `OutgoingChannelStatus`[] |
| `peer_quality` | (`string`: `any`) => `number` |

#### Returns

[`StrategyTickResult`](StrategyTickResult.md)

#### Inherited from

RustStrategyWrapper.tick

#### Defined in

[packages/core/src/channel-strategy.ts:86](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/channel-strategy.ts#L86)
