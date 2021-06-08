[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / HoprDB

# Class: HoprDB

## Table of contents

### Constructors

- [constructor](hoprdb.md#constructor)

### Properties

- [db](hoprdb.md#db)

### Methods

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
- [markRedeemeed](hoprdb.md#markredeemeed)
- [maybeGet](hoprdb.md#maybeget)
- [put](hoprdb.md#put)
- [replaceUnAckWithAck](hoprdb.md#replaceunackwithack)
- [setCurrentCommitment](hoprdb.md#setcurrentcommitment)
- [setCurrentTicketIndex](hoprdb.md#setcurrentticketindex)
- [storeHashIntermediaries](hoprdb.md#storehashintermediaries)
- [storeUnacknowledgedTicket](hoprdb.md#storeunacknowledgedticket)
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
| `id` | [Address](address.md) |
| `initialize` | `boolean` |
| `version` | `string` |
| `dbPath?` | `string` |

#### Defined in

[db.ts:50](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L50)

## Properties

### db

• `Private` **db**: `LevelUp`<AbstractLevelDOWN<any, any\>, AbstractIterator<any, any\>\>

#### Defined in

[db.ts:50](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L50)

## Methods

### checkAndSetPacketTag

▸ **checkAndSetPacketTag**(`packetTag`): `Promise`<boolean\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `packetTag` | `Uint8Array` |

#### Returns

`Promise`<boolean\>

#### Defined in

[db.ts:243](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L243)

___

### close

▸ **close**(): `Promise`<void\>

#### Returns

`Promise`<void\>

#### Defined in

[db.ts:253](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L253)

___

### del

▸ `Private` **del**(`key`): `Promise`<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | `Uint8Array` |

#### Returns

`Promise`<void\>

#### Defined in

[db.ts:145](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L145)

___

### delAcknowledgedTicket

▸ **delAcknowledgedTicket**(`ack`): `Promise`<void\>

Delete acknowledged ticket in database

#### Parameters

| Name | Type |
| :------ | :------ |
| `ack` | [AcknowledgedTicket](acknowledgedticket.md) |

#### Returns

`Promise`<void\>

#### Defined in

[db.ts:208](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L208)

___

### get

▸ `Private` **get**(`key`): `Promise`<Uint8Array\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | `Uint8Array` |

#### Returns

`Promise`<Uint8Array\>

#### Defined in

[db.ts:98](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L98)

___

### getAccount

▸ **getAccount**(`address`): `Promise`<[AccountEntry](accountentry.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | [Address](address.md) |

#### Returns

`Promise`<[AccountEntry](accountentry.md)\>

#### Defined in

[db.ts:327](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L327)

___

### getAccounts

▸ **getAccounts**(`filter?`): `Promise`<[AccountEntry](accountentry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `filter?` | (`account`: [AccountEntry](accountentry.md)) => `boolean` |

#### Returns

`Promise`<[AccountEntry](accountentry.md)[]\>

#### Defined in

[db.ts:336](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L336)

___

### getAcknowledgedTickets

▸ **getAcknowledgedTickets**(`filter?`): `Promise`<[AcknowledgedTicket](acknowledgedticket.md)[]\>

Get acknowledged tickets

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `filter?` | `Object` | optionally filter by signer |
| `filter.signer` | [PublicKey](publickey.md) | - |

#### Returns

`Promise`<[AcknowledgedTicket](acknowledgedticket.md)[]\>

an array of all acknowledged tickets

#### Defined in

[db.ts:192](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L192)

___

### getAll

▸ `Private` **getAll**<T\>(`prefix`, `deserialize`, `filter`): `Promise`<T[]\>

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

`Promise`<T[]\>

#### Defined in

[db.ts:121](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L121)

___

### getChannel

▸ **getChannel**(`channelId`): `Promise`<[ChannelEntry](channelentry.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [Hash](hash.md) |

#### Returns

`Promise`<[ChannelEntry](channelentry.md)\>

#### Defined in

[db.ts:314](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L314)

___

### getChannels

▸ **getChannels**(`filter?`): `Promise`<[ChannelEntry](channelentry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `filter?` | (`channel`: [ChannelEntry](channelentry.md)) => `boolean` |

#### Returns

`Promise`<[ChannelEntry](channelentry.md)[]\>

#### Defined in

[db.ts:318](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L318)

___

### getCoercedOrDefault

▸ `Private` **getCoercedOrDefault**<T\>(`key`, `coerce`, `defaultVal`): `Promise`<T\>

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

`Promise`<T\>

#### Defined in

[db.ts:113](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L113)

___

### getCommitment

▸ **getCommitment**(`channelId`, `iteration`): `Promise`<Uint8Array\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [Hash](hash.md) |
| `iteration` | `number` |

#### Returns

`Promise`<Uint8Array\>

#### Defined in

[db.ts:267](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L267)

___

### getCurrentCommitment

▸ **getCurrentCommitment**(`channelId`): `Promise`<[Hash](hash.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [Hash](hash.md) |

#### Returns

`Promise`<[Hash](hash.md)\>

#### Defined in

[db.ts:271](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L271)

___

### getCurrentTicketIndex

▸ **getCurrentTicketIndex**(`channelId`): `Promise`<[UINT256](uint256.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [Hash](hash.md) |

#### Returns

`Promise`<[UINT256](uint256.md)\>

#### Defined in

[db.ts:282](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L282)

___

### getLatestBlockNumber

▸ **getLatestBlockNumber**(): `Promise`<number\>

#### Returns

`Promise`<number\>

#### Defined in

[db.ts:297](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L297)

___

### getLatestConfirmedSnapshot

▸ **getLatestConfirmedSnapshot**(): `Promise`<[Snapshot](snapshot.md)\>

#### Returns

`Promise`<[Snapshot](snapshot.md)\>

#### Defined in

[db.ts:306](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L306)

___

### getLosingTicketCount

▸ **getLosingTicketCount**(): `Promise`<number\>

#### Returns

`Promise`<number\>

#### Defined in

[db.ts:352](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L352)

___

### getPendingTicketCount

▸ **getPendingTicketCount**(): `Promise`<number\>

#### Returns

`Promise`<number\>

#### Defined in

[db.ts:348](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L348)

___

### getRedeemedTicketsCount

▸ **getRedeemedTicketsCount**(): `Promise`<number\>

#### Returns

`Promise`<number\>

#### Defined in

[db.ts:344](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L344)

___

### getRedeemedTicketsValue

▸ **getRedeemedTicketsValue**(): `Promise`<[Balance](balance.md)\>

#### Returns

`Promise`<[Balance](balance.md)\>

#### Defined in

[db.ts:341](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L341)

___

### getTickets

▸ **getTickets**(`filter?`): `Promise`<[Ticket](ticket.md)[]\>

Get tickets, both unacknowledged and acknowledged

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `filter?` | `Object` | optionally filter by signer |
| `filter.signer` | [PublicKey](publickey.md) | - |

#### Returns

`Promise`<[Ticket](ticket.md)[]\>

an array of signed tickets

#### Defined in

[db.ts:233](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L233)

___

### getUnacknowledgedTicket

▸ **getUnacknowledgedTicket**(`halfKeyChallenge`): `Promise`<[UnacknowledgedTicket](unacknowledgedticket.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `halfKeyChallenge` | [HalfKeyChallenge](halfkeychallenge.md) |

#### Returns

`Promise`<[UnacknowledgedTicket](unacknowledgedticket.md)\>

#### Defined in

[db.ts:176](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L176)

___

### getUnacknowledgedTickets

▸ **getUnacknowledgedTickets**(`filter?`): `Promise`<[UnacknowledgedTicket](unacknowledgedticket.md)[]\>

Get unacknowledged tickets.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `filter?` | `Object` | optionally filter by signer |
| `filter.signer` | [PublicKey](publickey.md) | - |

#### Returns

`Promise`<[UnacknowledgedTicket](unacknowledgedticket.md)[]\>

an array of all unacknowledged tickets

#### Defined in

[db.ts:160](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L160)

___

### has

▸ `Private` **has**(`key`): `Promise`<boolean\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | `Uint8Array` |

#### Returns

`Promise`<boolean\>

#### Defined in

[db.ts:76](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L76)

___

### increment

▸ `Private` **increment**(`key`): `Promise`<number\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | `Uint8Array` |

#### Returns

`Promise`<number\>

#### Defined in

[db.ts:149](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L149)

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

[db.ts:72](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L72)

___

### markLosing

▸ **markLosing**(`t`): `Promise`<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `t` | [UnacknowledgedTicket](unacknowledgedticket.md) |

#### Returns

`Promise`<void\>

#### Defined in

[db.ts:362](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L362)

___

### markRedeemeed

▸ **markRedeemeed**(`a`): `Promise`<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `a` | [AcknowledgedTicket](acknowledgedticket.md) |

#### Returns

`Promise`<void\>

#### Defined in

[db.ts:356](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L356)

___

### maybeGet

▸ `Private` **maybeGet**(`key`): `Promise`<Uint8Array\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | `Uint8Array` |

#### Returns

`Promise`<Uint8Array\>

#### Defined in

[db.ts:102](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L102)

___

### put

▸ `Private` **put**(`key`, `value`): `Promise`<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | `Uint8Array` |
| `value` | `Uint8Array` |

#### Returns

`Promise`<void\>

#### Defined in

[db.ts:90](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L90)

___

### replaceUnAckWithAck

▸ **replaceUnAckWithAck**(`halfKeyChallenge`, `ackTicket`): `Promise`<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `halfKeyChallenge` | [HalfKeyChallenge](halfkeychallenge.md) |
| `ackTicket` | [AcknowledgedTicket](acknowledgedticket.md) |

#### Returns

`Promise`<void\>

#### Defined in

[db.ts:212](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L212)

___

### setCurrentCommitment

▸ **setCurrentCommitment**(`channelId`, `commitment`): `Promise`<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [Hash](hash.md) |
| `commitment` | [Hash](hash.md) |

#### Returns

`Promise`<void\>

#### Defined in

[db.ts:275](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L275)

___

### setCurrentTicketIndex

▸ **setCurrentTicketIndex**(`channelId`, `ticketIndex`): `Promise`<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [Hash](hash.md) |
| `ticketIndex` | [UINT256](uint256.md) |

#### Returns

`Promise`<void\>

#### Defined in

[db.ts:290](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L290)

___

### storeHashIntermediaries

▸ **storeHashIntermediaries**(`channelId`, `intermediates`): `Promise`<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [Hash](hash.md) |
| `intermediates` | [Intermediate](../interfaces/intermediate.md)[] |

#### Returns

`Promise`<void\>

#### Defined in

[db.ts:257](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L257)

___

### storeUnacknowledgedTicket

▸ **storeUnacknowledgedTicket**(`halfKeyChallenge`, `unackTicket`): `Promise`<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `halfKeyChallenge` | [HalfKeyChallenge](halfkeychallenge.md) |
| `unackTicket` | [UnacknowledgedTicket](unacknowledgedticket.md) |

#### Returns

`Promise`<void\>

#### Defined in

[db.ts:180](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L180)

___

### touch

▸ `Private` **touch**(`key`): `Promise`<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | `Uint8Array` |

#### Returns

`Promise`<void\>

#### Defined in

[db.ts:94](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L94)

___

### updateAccount

▸ **updateAccount**(`account`): `Promise`<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | [AccountEntry](accountentry.md) |

#### Returns

`Promise`<void\>

#### Defined in

[db.ts:332](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L332)

___

### updateChannel

▸ **updateChannel**(`channelId`, `channel`): `Promise`<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [Hash](hash.md) |
| `channel` | [ChannelEntry](channelentry.md) |

#### Returns

`Promise`<void\>

#### Defined in

[db.ts:323](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L323)

___

### updateLatestBlockNumber

▸ **updateLatestBlockNumber**(`blockNumber`): `Promise`<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `blockNumber` | `BN` |

#### Returns

`Promise`<void\>

#### Defined in

[db.ts:302](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L302)

___

### updateLatestConfirmedSnapshot

▸ **updateLatestConfirmedSnapshot**(`snapshot`): `Promise`<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `snapshot` | [Snapshot](snapshot.md) |

#### Returns

`Promise`<void\>

#### Defined in

[db.ts:310](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L310)

___

### createMock

▸ `Static` **createMock**(): [HoprDB](hoprdb.md)

#### Returns

[HoprDB](hoprdb.md)

#### Defined in

[db.ts:367](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L367)
