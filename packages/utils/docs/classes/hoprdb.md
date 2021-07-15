[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / HoprDB

# Class: HoprDB

## Table of contents

### Constructors

- [constructor](hoprdb.md#constructor)

### Properties

- [db](hoprdb.md#db)

### Methods

- [addBalance](hoprdb.md#addbalance)
- [checkAndSetPacketTag](hoprdb.md#checkandsetpackettag)
- [close](hoprdb.md#close)
- [del](hoprdb.md#del)
- [delAcknowledgedTicket](hoprdb.md#delacknowledgedticket)
- [get](hoprdb.md#get)
- [getAccount](hoprdb.md#getaccount)
- [getAccounts](hoprdb.md#getaccounts)
- [getAcknowledgedTickets](hoprdb.md#getacknowledgedtickets)
- [getAll](hoprdb.md#getall)
- [getChannel](hoprdb.md#getchannel)
- [getChannels](hoprdb.md#getchannels)
- [getCoercedOrDefault](hoprdb.md#getcoercedordefault)
- [getCommitment](hoprdb.md#getcommitment)
- [getCurrentCommitment](hoprdb.md#getcurrentcommitment)
- [getCurrentTicketIndex](hoprdb.md#getcurrentticketindex)
- [getLatestBlockNumber](hoprdb.md#getlatestblocknumber)
- [getLatestConfirmedSnapshot](hoprdb.md#getlatestconfirmedsnapshot)
- [getLosingTicketCount](hoprdb.md#getlosingticketcount)
- [getPendingBalanceTo](hoprdb.md#getpendingbalanceto)
- [getPendingTicketCount](hoprdb.md#getpendingticketcount)
- [getRedeemedTicketsCount](hoprdb.md#getredeemedticketscount)
- [getRedeemedTicketsValue](hoprdb.md#getredeemedticketsvalue)
- [getTickets](hoprdb.md#gettickets)
- [getUnacknowledgedTicket](hoprdb.md#getunacknowledgedticket)
- [getUnacknowledgedTickets](hoprdb.md#getunacknowledgedtickets)
- [has](hoprdb.md#has)
- [increment](hoprdb.md#increment)
- [keyOf](hoprdb.md#keyof)
- [markLosing](hoprdb.md#marklosing)
- [markPending](hoprdb.md#markpending)
- [markRedeemeed](hoprdb.md#markredeemeed)
- [maybeGet](hoprdb.md#maybeget)
- [put](hoprdb.md#put)
- [replaceUnAckWithAck](hoprdb.md#replaceunackwithack)
- [setCurrentCommitment](hoprdb.md#setcurrentcommitment)
- [setCurrentTicketIndex](hoprdb.md#setcurrentticketindex)
- [storeHashIntermediaries](hoprdb.md#storehashintermediaries)
- [storeUnacknowledgedTicket](hoprdb.md#storeunacknowledgedticket)
- [subBalance](hoprdb.md#subbalance)
- [touch](hoprdb.md#touch)
- [updateAccount](hoprdb.md#updateaccount)
- [updateChannel](hoprdb.md#updatechannel)
- [updateLatestBlockNumber](hoprdb.md#updatelatestblocknumber)
- [updateLatestConfirmedSnapshot](hoprdb.md#updatelatestconfirmedsnapshot)
- [createMock](hoprdb.md#createmock)

## Constructors

### constructor

• **new HoprDB**(`id`, `initialize`, `version`, `dbPath?`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `id` | [`Address`](address.md) |
| `initialize` | `boolean` |
| `version` | `string` |
| `dbPath?` | `string` |

#### Defined in

[db.ts:52](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L52)

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
| `amount` | [`Balance`](balance.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:157](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L157)

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

[db.ts:255](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L255)

___

### close

▸ **close**(): `Promise`<`void`\>

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:265](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L265)

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

[db.ts:147](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L147)

___

### delAcknowledgedTicket

▸ **delAcknowledgedTicket**(`ack`): `Promise`<`void`\>

Delete acknowledged ticket in database

#### Parameters

| Name | Type |
| :------ | :------ |
| `ack` | [`AcknowledgedTicket`](acknowledgedticket.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:220](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L220)

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

[db.ts:100](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L100)

___

### getAccount

▸ **getAccount**(`address`): `Promise`<[`AccountEntry`](accountentry.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | [`Address`](address.md) |

#### Returns

`Promise`<[`AccountEntry`](accountentry.md)\>

#### Defined in

[db.ts:339](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L339)

___

### getAccounts

▸ **getAccounts**(`filter?`): `Promise`<[`AccountEntry`](accountentry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `filter?` | (`account`: [`AccountEntry`](accountentry.md)) => `boolean` |

#### Returns

`Promise`<[`AccountEntry`](accountentry.md)[]\>

#### Defined in

[db.ts:348](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L348)

___

### getAcknowledgedTickets

▸ **getAcknowledgedTickets**(`filter?`): `Promise`<[`AcknowledgedTicket`](acknowledgedticket.md)[]\>

Get acknowledged tickets

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `filter?` | `Object` | optionally filter by signer |
| `filter.signer` | [`PublicKey`](publickey.md) | - |

#### Returns

`Promise`<[`AcknowledgedTicket`](acknowledgedticket.md)[]\>

an array of all acknowledged tickets

#### Defined in

[db.ts:204](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L204)

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

[db.ts:123](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L123)

___

### getChannel

▸ **getChannel**(`channelId`): `Promise`<[`ChannelEntry`](channelentry.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [`Hash`](hash.md) |

#### Returns

`Promise`<[`ChannelEntry`](channelentry.md)\>

#### Defined in

[db.ts:326](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L326)

___

### getChannels

▸ **getChannels**(`filter?`): `Promise`<[`ChannelEntry`](channelentry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `filter?` | (`channel`: [`ChannelEntry`](channelentry.md)) => `boolean` |

#### Returns

`Promise`<[`ChannelEntry`](channelentry.md)[]\>

#### Defined in

[db.ts:330](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L330)

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

[db.ts:115](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L115)

___

### getCommitment

▸ **getCommitment**(`channelId`, `iteration`): `Promise`<`Uint8Array`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [`Hash`](hash.md) |
| `iteration` | `number` |

#### Returns

`Promise`<`Uint8Array`\>

#### Defined in

[db.ts:279](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L279)

___

### getCurrentCommitment

▸ **getCurrentCommitment**(`channelId`): `Promise`<[`Hash`](hash.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [`Hash`](hash.md) |

#### Returns

`Promise`<[`Hash`](hash.md)\>

#### Defined in

[db.ts:283](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L283)

___

### getCurrentTicketIndex

▸ **getCurrentTicketIndex**(`channelId`): `Promise`<[`UINT256`](uint256.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [`Hash`](hash.md) |

#### Returns

`Promise`<[`UINT256`](uint256.md)\>

#### Defined in

[db.ts:294](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L294)

___

### getLatestBlockNumber

▸ **getLatestBlockNumber**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db.ts:309](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L309)

___

### getLatestConfirmedSnapshot

▸ **getLatestConfirmedSnapshot**(): `Promise`<[`Snapshot`](snapshot.md)\>

#### Returns

`Promise`<[`Snapshot`](snapshot.md)\>

#### Defined in

[db.ts:318](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L318)

___

### getLosingTicketCount

▸ **getLosingTicketCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db.ts:368](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L368)

___

### getPendingBalanceTo

▸ **getPendingBalanceTo**(`counterparty`): `Promise`<[`Balance`](balance.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | [`Address`](address.md) |

#### Returns

`Promise`<[`Balance`](balance.md)\>

#### Defined in

[db.ts:364](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L364)

___

### getPendingTicketCount

▸ **getPendingTicketCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db.ts:360](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L360)

___

### getRedeemedTicketsCount

▸ **getRedeemedTicketsCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db.ts:356](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L356)

___

### getRedeemedTicketsValue

▸ **getRedeemedTicketsValue**(): `Promise`<[`Balance`](balance.md)\>

#### Returns

`Promise`<[`Balance`](balance.md)\>

#### Defined in

[db.ts:353](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L353)

___

### getTickets

▸ **getTickets**(`filter?`): `Promise`<[`Ticket`](ticket.md)[]\>

Get tickets, both unacknowledged and acknowledged

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `filter?` | `Object` | optionally filter by signer |
| `filter.signer` | [`PublicKey`](publickey.md) | - |

#### Returns

`Promise`<[`Ticket`](ticket.md)[]\>

an array of signed tickets

#### Defined in

[db.ts:245](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L245)

___

### getUnacknowledgedTicket

▸ **getUnacknowledgedTicket**(`halfKeyChallenge`): `Promise`<[`UnacknowledgedTicket`](unacknowledgedticket.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `halfKeyChallenge` | [`HalfKeyChallenge`](halfkeychallenge.md) |

#### Returns

`Promise`<[`UnacknowledgedTicket`](unacknowledgedticket.md)\>

#### Defined in

[db.ts:188](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L188)

___

### getUnacknowledgedTickets

▸ **getUnacknowledgedTickets**(`filter?`): `Promise`<[`UnacknowledgedTicket`](unacknowledgedticket.md)[]\>

Get unacknowledged tickets.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `filter?` | `Object` | optionally filter by signer |
| `filter.signer` | [`PublicKey`](publickey.md) | - |

#### Returns

`Promise`<[`UnacknowledgedTicket`](unacknowledgedticket.md)[]\>

an array of all unacknowledged tickets

#### Defined in

[db.ts:172](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L172)

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

[db.ts:78](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L78)

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

[db.ts:151](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L151)

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

[db.ts:74](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L74)

___

### markLosing

▸ **markLosing**(`t`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `t` | [`UnacknowledgedTicket`](unacknowledgedticket.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:383](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L383)

___

### markPending

▸ **markPending**(`ticket`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ticket` | [`Ticket`](ticket.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:372](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L372)

___

### markRedeemeed

▸ **markRedeemeed**(`a`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `a` | [`AcknowledgedTicket`](acknowledgedticket.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:376](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L376)

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

[db.ts:104](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L104)

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

[db.ts:92](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L92)

___

### replaceUnAckWithAck

▸ **replaceUnAckWithAck**(`halfKeyChallenge`, `ackTicket`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `halfKeyChallenge` | [`HalfKeyChallenge`](halfkeychallenge.md) |
| `ackTicket` | [`AcknowledgedTicket`](acknowledgedticket.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:224](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L224)

___

### setCurrentCommitment

▸ **setCurrentCommitment**(`channelId`, `commitment`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [`Hash`](hash.md) |
| `commitment` | [`Hash`](hash.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:287](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L287)

___

### setCurrentTicketIndex

▸ **setCurrentTicketIndex**(`channelId`, `ticketIndex`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [`Hash`](hash.md) |
| `ticketIndex` | [`UINT256`](uint256.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:302](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L302)

___

### storeHashIntermediaries

▸ **storeHashIntermediaries**(`channelId`, `intermediates`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [`Hash`](hash.md) |
| `intermediates` | [`Intermediate`](../interfaces/intermediate.md)[] |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:269](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L269)

___

### storeUnacknowledgedTicket

▸ **storeUnacknowledgedTicket**(`halfKeyChallenge`, `unackTicket`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `halfKeyChallenge` | [`HalfKeyChallenge`](halfkeychallenge.md) |
| `unackTicket` | [`UnacknowledgedTicket`](unacknowledgedticket.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:192](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L192)

___

### subBalance

▸ `Private` **subBalance**(`key`, `amount`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | `Uint8Array` |
| `amount` | [`Balance`](balance.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:162](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L162)

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

[db.ts:96](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L96)

___

### updateAccount

▸ **updateAccount**(`account`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | [`AccountEntry`](accountentry.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:344](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L344)

___

### updateChannel

▸ **updateChannel**(`channelId`, `channel`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [`Hash`](hash.md) |
| `channel` | [`ChannelEntry`](channelentry.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:335](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L335)

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

[db.ts:314](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L314)

___

### updateLatestConfirmedSnapshot

▸ **updateLatestConfirmedSnapshot**(`snapshot`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `snapshot` | [`Snapshot`](snapshot.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:322](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L322)

___

### createMock

▸ `Static` **createMock**(): [`HoprDB`](hoprdb.md)

#### Returns

[`HoprDB`](hoprdb.md)

#### Defined in

[db.ts:389](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L389)
