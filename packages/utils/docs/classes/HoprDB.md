[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / HoprDB

# Class: HoprDB

## Table of contents

### Constructors

- [constructor](HoprDB.md#constructor)

### Properties

- [db](HoprDB.md#db)

### Methods

- [addBalance](HoprDB.md#addbalance)
- [checkAndSetPacketTag](HoprDB.md#checkandsetpackettag)
- [close](HoprDB.md#close)
- [del](HoprDB.md#del)
- [delAcknowledgedTicket](HoprDB.md#delacknowledgedticket)
- [get](HoprDB.md#get)
- [getAccount](HoprDB.md#getaccount)
- [getAccounts](HoprDB.md#getaccounts)
- [getAcknowledgedTickets](HoprDB.md#getacknowledgedtickets)
- [getAll](HoprDB.md#getall)
- [getChannel](HoprDB.md#getchannel)
- [getChannels](HoprDB.md#getchannels)
- [getCoercedOrDefault](HoprDB.md#getcoercedordefault)
- [getCommitment](HoprDB.md#getcommitment)
- [getCurrentCommitment](HoprDB.md#getcurrentcommitment)
- [getCurrentTicketIndex](HoprDB.md#getcurrentticketindex)
- [getLatestBlockNumber](HoprDB.md#getlatestblocknumber)
- [getLatestConfirmedSnapshot](HoprDB.md#getlatestconfirmedsnapshot)
- [getLosingTicketCount](HoprDB.md#getlosingticketcount)
- [getPendingBalanceTo](HoprDB.md#getpendingbalanceto)
- [getPendingTicketCount](HoprDB.md#getpendingticketcount)
- [getRedeemedTicketsCount](HoprDB.md#getredeemedticketscount)
- [getRedeemedTicketsValue](HoprDB.md#getredeemedticketsvalue)
- [getTickets](HoprDB.md#gettickets)
- [getUnacknowledgedTicket](HoprDB.md#getunacknowledgedticket)
- [getUnacknowledgedTickets](HoprDB.md#getunacknowledgedtickets)
- [has](HoprDB.md#has)
- [increment](HoprDB.md#increment)
- [keyOf](HoprDB.md#keyof)
- [markLosing](HoprDB.md#marklosing)
- [markPending](HoprDB.md#markpending)
- [markRedeemeed](HoprDB.md#markredeemeed)
- [maybeGet](HoprDB.md#maybeget)
- [put](HoprDB.md#put)
- [replaceUnAckWithAck](HoprDB.md#replaceunackwithack)
- [setCurrentCommitment](HoprDB.md#setcurrentcommitment)
- [setCurrentTicketIndex](HoprDB.md#setcurrentticketindex)
- [storeHashIntermediaries](HoprDB.md#storehashintermediaries)
- [storeUnacknowledgedTicket](HoprDB.md#storeunacknowledgedticket)
- [subBalance](HoprDB.md#subbalance)
- [touch](HoprDB.md#touch)
- [updateAccount](HoprDB.md#updateaccount)
- [updateChannel](HoprDB.md#updatechannel)
- [updateLatestBlockNumber](HoprDB.md#updatelatestblocknumber)
- [updateLatestConfirmedSnapshot](HoprDB.md#updatelatestconfirmedsnapshot)
- [createMock](HoprDB.md#createmock)

## Constructors

### constructor

• **new HoprDB**(`id`, `initialize`, `version`, `dbPath?`, `forceCreate?`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `id` | [`Address`](Address.md) |
| `initialize` | `boolean` |
| `version` | `string` |
| `dbPath?` | `string` |
| `forceCreate?` | `boolean` |

#### Defined in

[db.ts:54](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L54)

## Properties

### db

• `Private` **db**: `LevelUp`<`AbstractLevelDOWN`<`any`, `any`\>, `AbstractIterator`<`any`, `any`\>\>

#### Defined in

[db.ts:52](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L52)

## Methods

### addBalance

▸ `Private` **addBalance**(`key`, `amount`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | `Uint8Array` |
| `amount` | [`Balance`](Balance.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:162](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L162)

___

### checkAndSetPacketTag

▸ **checkAndSetPacketTag**(`packetTag`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `packetTag` | `Uint8Array` |

#### Returns

`Promise`<`boolean`\>

#### Defined in

[db.ts:260](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L260)

___

### close

▸ **close**(): `Promise`<`void`\>

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:270](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L270)

___

### del

▸ `Private` **del**(`key`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | `Uint8Array` |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:152](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L152)

___

### delAcknowledgedTicket

▸ **delAcknowledgedTicket**(`ack`): `Promise`<`void`\>

Delete acknowledged ticket in database

#### Parameters

| Name | Type |
| :------ | :------ |
| `ack` | [`AcknowledgedTicket`](AcknowledgedTicket.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:225](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L225)

___

### get

▸ `Private` **get**(`key`): `Promise`<`Uint8Array`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | `Uint8Array` |

#### Returns

`Promise`<`Uint8Array`\>

#### Defined in

[db.ts:105](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L105)

___

### getAccount

▸ **getAccount**(`address`): `Promise`<[`AccountEntry`](AccountEntry.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | [`Address`](Address.md) |

#### Returns

`Promise`<[`AccountEntry`](AccountEntry.md)\>

#### Defined in

[db.ts:344](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L344)

___

### getAccounts

▸ **getAccounts**(`filter?`): `Promise`<[`AccountEntry`](AccountEntry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `filter?` | (`account`: [`AccountEntry`](AccountEntry.md)) => `boolean` |

#### Returns

`Promise`<[`AccountEntry`](AccountEntry.md)[]\>

#### Defined in

[db.ts:353](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L353)

___

### getAcknowledgedTickets

▸ **getAcknowledgedTickets**(`filter?`): `Promise`<[`AcknowledgedTicket`](AcknowledgedTicket.md)[]\>

Get acknowledged tickets

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `filter?` | `Object` | optionally filter by signer |
| `filter.signer` | [`PublicKey`](PublicKey.md) | - |

#### Returns

`Promise`<[`AcknowledgedTicket`](AcknowledgedTicket.md)[]\>

an array of all acknowledged tickets

#### Defined in

[db.ts:209](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L209)

___

### getAll

▸ `Private` **getAll**<`T`\>(`prefix`, `deserialize`, `filter`): `Promise`<`T`[]\>

#### Type parameters

| Name |
| :------ |
| `T` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `prefix` | `Uint8Array` |
| `deserialize` | (`u`: `Uint8Array`) => `T` |
| `filter` | (`o`: `T`) => `boolean` |

#### Returns

`Promise`<`T`[]\>

#### Defined in

[db.ts:128](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L128)

___

### getChannel

▸ **getChannel**(`channelId`): `Promise`<[`ChannelEntry`](ChannelEntry.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [`Hash`](Hash.md) |

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)\>

#### Defined in

[db.ts:331](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L331)

___

### getChannels

▸ **getChannels**(`filter?`): `Promise`<[`ChannelEntry`](ChannelEntry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `filter?` | (`channel`: [`ChannelEntry`](ChannelEntry.md)) => `boolean` |

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)[]\>

#### Defined in

[db.ts:335](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L335)

___

### getCoercedOrDefault

▸ `Private` **getCoercedOrDefault**<`T`\>(`key`, `coerce`, `defaultVal`): `Promise`<`T`\>

#### Type parameters

| Name |
| :------ |
| `T` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | `Uint8Array` |
| `coerce` | (`u`: `Uint8Array`) => `T` |
| `defaultVal` | `T` |

#### Returns

`Promise`<`T`\>

#### Defined in

[db.ts:120](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L120)

___

### getCommitment

▸ **getCommitment**(`channelId`, `iteration`): `Promise`<`Uint8Array`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [`Hash`](Hash.md) |
| `iteration` | `number` |

#### Returns

`Promise`<`Uint8Array`\>

#### Defined in

[db.ts:284](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L284)

___

### getCurrentCommitment

▸ **getCurrentCommitment**(`channelId`): `Promise`<[`Hash`](Hash.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [`Hash`](Hash.md) |

#### Returns

`Promise`<[`Hash`](Hash.md)\>

#### Defined in

[db.ts:288](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L288)

___

### getCurrentTicketIndex

▸ **getCurrentTicketIndex**(`channelId`): `Promise`<[`UINT256`](UINT256.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [`Hash`](Hash.md) |

#### Returns

`Promise`<[`UINT256`](UINT256.md)\>

#### Defined in

[db.ts:299](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L299)

___

### getLatestBlockNumber

▸ **getLatestBlockNumber**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db.ts:314](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L314)

___

### getLatestConfirmedSnapshot

▸ **getLatestConfirmedSnapshot**(): `Promise`<[`Snapshot`](Snapshot.md)\>

#### Returns

`Promise`<[`Snapshot`](Snapshot.md)\>

#### Defined in

[db.ts:323](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L323)

___

### getLosingTicketCount

▸ **getLosingTicketCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db.ts:373](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L373)

___

### getPendingBalanceTo

▸ **getPendingBalanceTo**(`counterparty`): `Promise`<[`Balance`](Balance.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | [`Address`](Address.md) |

#### Returns

`Promise`<[`Balance`](Balance.md)\>

#### Defined in

[db.ts:369](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L369)

___

### getPendingTicketCount

▸ **getPendingTicketCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db.ts:365](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L365)

___

### getRedeemedTicketsCount

▸ **getRedeemedTicketsCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db.ts:361](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L361)

___

### getRedeemedTicketsValue

▸ **getRedeemedTicketsValue**(): `Promise`<[`Balance`](Balance.md)\>

#### Returns

`Promise`<[`Balance`](Balance.md)\>

#### Defined in

[db.ts:358](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L358)

___

### getTickets

▸ **getTickets**(`filter?`): `Promise`<[`Ticket`](Ticket.md)[]\>

Get tickets, both unacknowledged and acknowledged

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `filter?` | `Object` | optionally filter by signer |
| `filter.signer` | [`PublicKey`](PublicKey.md) | - |

#### Returns

`Promise`<[`Ticket`](Ticket.md)[]\>

an array of signed tickets

#### Defined in

[db.ts:250](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L250)

___

### getUnacknowledgedTicket

▸ **getUnacknowledgedTicket**(`halfKeyChallenge`): `Promise`<[`UnacknowledgedTicket`](UnacknowledgedTicket.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `halfKeyChallenge` | [`HalfKeyChallenge`](HalfKeyChallenge.md) |

#### Returns

`Promise`<[`UnacknowledgedTicket`](UnacknowledgedTicket.md)\>

#### Defined in

[db.ts:193](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L193)

___

### getUnacknowledgedTickets

▸ **getUnacknowledgedTickets**(`filter?`): `Promise`<[`UnacknowledgedTicket`](UnacknowledgedTicket.md)[]\>

Get unacknowledged tickets.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `filter?` | `Object` | optionally filter by signer |
| `filter.signer` | [`PublicKey`](PublicKey.md) | - |

#### Returns

`Promise`<[`UnacknowledgedTicket`](UnacknowledgedTicket.md)[]\>

an array of all unacknowledged tickets

#### Defined in

[db.ts:177](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L177)

___

### has

▸ `Private` **has**(`key`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | `Uint8Array` |

#### Returns

`Promise`<`boolean`\>

#### Defined in

[db.ts:83](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L83)

___

### increment

▸ `Private` **increment**(`key`): `Promise`<`number`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | `Uint8Array` |

#### Returns

`Promise`<`number`\>

#### Defined in

[db.ts:156](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L156)

___

### keyOf

▸ `Private` **keyOf**(...`segments`): `Uint8Array`

#### Parameters

| Name | Type |
| :------ | :------ |
| `...segments` | `Uint8Array`[] |

#### Returns

`Uint8Array`

#### Defined in

[db.ts:79](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L79)

___

### markLosing

▸ **markLosing**(`t`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `t` | [`UnacknowledgedTicket`](UnacknowledgedTicket.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:388](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L388)

___

### markPending

▸ **markPending**(`ticket`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ticket` | [`Ticket`](Ticket.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:377](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L377)

___

### markRedeemeed

▸ **markRedeemeed**(`a`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `a` | [`AcknowledgedTicket`](AcknowledgedTicket.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:381](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L381)

___

### maybeGet

▸ `Private` **maybeGet**(`key`): `Promise`<`Uint8Array`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | `Uint8Array` |

#### Returns

`Promise`<`Uint8Array`\>

#### Defined in

[db.ts:109](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L109)

___

### put

▸ `Private` **put**(`key`, `value`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | `Uint8Array` |
| `value` | `Uint8Array` |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:97](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L97)

___

### replaceUnAckWithAck

▸ **replaceUnAckWithAck**(`halfKeyChallenge`, `ackTicket`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `halfKeyChallenge` | [`HalfKeyChallenge`](HalfKeyChallenge.md) |
| `ackTicket` | [`AcknowledgedTicket`](AcknowledgedTicket.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:229](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L229)

___

### setCurrentCommitment

▸ **setCurrentCommitment**(`channelId`, `commitment`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [`Hash`](Hash.md) |
| `commitment` | [`Hash`](Hash.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:292](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L292)

___

### setCurrentTicketIndex

▸ **setCurrentTicketIndex**(`channelId`, `ticketIndex`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [`Hash`](Hash.md) |
| `ticketIndex` | [`UINT256`](UINT256.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:307](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L307)

___

### storeHashIntermediaries

▸ **storeHashIntermediaries**(`channelId`, `intermediates`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [`Hash`](Hash.md) |
| `intermediates` | [`Intermediate`](../interfaces/Intermediate.md)[] |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:274](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L274)

___

### storeUnacknowledgedTicket

▸ **storeUnacknowledgedTicket**(`halfKeyChallenge`, `unackTicket`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `halfKeyChallenge` | [`HalfKeyChallenge`](HalfKeyChallenge.md) |
| `unackTicket` | [`UnacknowledgedTicket`](UnacknowledgedTicket.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:197](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L197)

___

### subBalance

▸ `Private` **subBalance**(`key`, `amount`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | `Uint8Array` |
| `amount` | [`Balance`](Balance.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:167](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L167)

___

### touch

▸ `Private` **touch**(`key`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | `Uint8Array` |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:101](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L101)

___

### updateAccount

▸ **updateAccount**(`account`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | [`AccountEntry`](AccountEntry.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:349](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L349)

___

### updateChannel

▸ **updateChannel**(`channelId`, `channel`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [`Hash`](Hash.md) |
| `channel` | [`ChannelEntry`](ChannelEntry.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:340](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L340)

___

### updateLatestBlockNumber

▸ **updateLatestBlockNumber**(`blockNumber`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `blockNumber` | `BN` |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:319](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L319)

___

### updateLatestConfirmedSnapshot

▸ **updateLatestConfirmedSnapshot**(`snapshot`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `snapshot` | [`Snapshot`](Snapshot.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:327](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L327)

___

### createMock

▸ `Static` **createMock**(): [`HoprDB`](HoprDB.md)

#### Returns

[`HoprDB`](HoprDB.md)

#### Defined in

[db.ts:394](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L394)
