[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / HoprDB

# Class: HoprDB

## Table of contents

### Constructors

- [constructor](HoprDB.md#constructor)

### Properties

- [db](HoprDB.md#db)

### Methods

- [addBalance](HoprDB.md#addbalance)
- [addHoprBalance](HoprDB.md#addhoprbalance)
- [checkAndSetPacketTag](HoprDB.md#checkandsetpackettag)
- [close](HoprDB.md#close)
- [del](HoprDB.md#del)
- [delAcknowledgedTicket](HoprDB.md#delacknowledgedticket)
- [deleteAcknowledgedTicketsFromChannel](HoprDB.md#deleteacknowledgedticketsfromchannel)
- [deserializePendingAcknowledgement](HoprDB.md#deserializependingacknowledgement)
- [get](HoprDB.md#get)
- [getAccount](HoprDB.md#getaccount)
- [getAccounts](HoprDB.md#getaccounts)
- [getAcknowledgedTickets](HoprDB.md#getacknowledgedtickets)
- [getAll](HoprDB.md#getall)
- [getChannel](HoprDB.md#getchannel)
- [getChannelFrom](HoprDB.md#getchannelfrom)
- [getChannelTo](HoprDB.md#getchannelto)
- [getChannelX](HoprDB.md#getchannelx)
- [getChannels](HoprDB.md#getchannels)
- [getChannelsFrom](HoprDB.md#getchannelsfrom)
- [getChannelsTo](HoprDB.md#getchannelsto)
- [getCoerced](HoprDB.md#getcoerced)
- [getCoercedOrDefault](HoprDB.md#getcoercedordefault)
- [getCommitment](HoprDB.md#getcommitment)
- [getCurrentCommitment](HoprDB.md#getcurrentcommitment)
- [getCurrentTicketIndex](HoprDB.md#getcurrentticketindex)
- [getEnvironmentId](HoprDB.md#getenvironmentid)
- [getHoprBalance](HoprDB.md#gethoprbalance)
- [getLatestBlockNumber](HoprDB.md#getlatestblocknumber)
- [getLatestConfirmedSnapshotOrUndefined](HoprDB.md#getlatestconfirmedsnapshotorundefined)
- [getLosingTicketCount](HoprDB.md#getlosingticketcount)
- [getNeglectedTicketsCount](HoprDB.md#getneglectedticketscount)
- [getPendingAcknowledgement](HoprDB.md#getpendingacknowledgement)
- [getPendingBalanceTo](HoprDB.md#getpendingbalanceto)
- [getPendingTicketCount](HoprDB.md#getpendingticketcount)
- [getRedeemedTicketsCount](HoprDB.md#getredeemedticketscount)
- [getRedeemedTicketsValue](HoprDB.md#getredeemedticketsvalue)
- [getRejectedTicketsCount](HoprDB.md#getrejectedticketscount)
- [getRejectedTicketsValue](HoprDB.md#getrejectedticketsvalue)
- [getTickets](HoprDB.md#gettickets)
- [getUnacknowledgedTickets](HoprDB.md#getunacknowledgedtickets)
- [has](HoprDB.md#has)
- [increment](HoprDB.md#increment)
- [init](HoprDB.md#init)
- [keyOf](HoprDB.md#keyof)
- [markLosing](HoprDB.md#marklosing)
- [markPending](HoprDB.md#markpending)
- [markRedeemeed](HoprDB.md#markredeemeed)
- [markRejected](HoprDB.md#markrejected)
- [maybeGet](HoprDB.md#maybeget)
- [put](HoprDB.md#put)
- [replaceUnAckWithAck](HoprDB.md#replaceunackwithack)
- [resolvePending](HoprDB.md#resolvepending)
- [serializePendingAcknowledgement](HoprDB.md#serializependingacknowledgement)
- [setCurrentCommitment](HoprDB.md#setcurrentcommitment)
- [setCurrentTicketIndex](HoprDB.md#setcurrentticketindex)
- [setEnvironmentId](HoprDB.md#setenvironmentid)
- [setHoprBalance](HoprDB.md#sethoprbalance)
- [storeHashIntermediaries](HoprDB.md#storehashintermediaries)
- [storePendingAcknowledgement](HoprDB.md#storependingacknowledgement)
- [subBalance](HoprDB.md#subbalance)
- [subHoprBalance](HoprDB.md#subhoprbalance)
- [touch](HoprDB.md#touch)
- [updateAccount](HoprDB.md#updateaccount)
- [updateChannel](HoprDB.md#updatechannel)
- [updateLatestBlockNumber](HoprDB.md#updatelatestblocknumber)
- [updateLatestConfirmedSnapshot](HoprDB.md#updatelatestconfirmedsnapshot)
- [verifyEnvironmentId](HoprDB.md#verifyenvironmentid)
- [createMock](HoprDB.md#createmock)

## Constructors

### constructor

• **new HoprDB**(`id`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `id` | [`PublicKey`](PublicKey.md) |

#### Defined in

[db.ts:77](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L77)

## Properties

### db

• `Private` **db**: `LevelUp`<`AbstractLevelDOWN`<`any`, `any`\>, `AbstractIterator`<`any`, `any`\>\>

#### Defined in

[db.ts:75](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L75)

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

[db.ts:232](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L232)

___

### addHoprBalance

▸ **addHoprBalance**(`value`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `value` | [`Balance`](Balance.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:612](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L612)

___

### checkAndSetPacketTag

▸ **checkAndSetPacketTag**(`packetTag`): `Promise`<`boolean`\>

Checks whether the given packet tag is present in the database.
If not, sets the packet tag and return false, otherwise return
true.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `packetTag` | `Uint8Array` | packet tag to check for |

#### Returns

`Promise`<`boolean`\>

a Promise that resolves to true if packet tag is present in db

#### Defined in

[db.ts:400](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L400)

___

### close

▸ **close**(): `Promise`<`void`\>

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:410](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L410)

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

[db.ts:222](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L222)

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

[db.ts:362](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L362)

___

### deleteAcknowledgedTicketsFromChannel

▸ **deleteAcknowledgedTicketsFromChannel**(`channel`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channel` | [`ChannelEntry`](ChannelEntry.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:352](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L352)

___

### deserializePendingAcknowledgement

▸ `Private` **deserializePendingAcknowledgement**(`data`): [`PendingAckowledgement`](../modules.md#pendingackowledgement)

#### Parameters

| Name | Type |
| :------ | :------ |
| `data` | `Uint8Array` |

#### Returns

[`PendingAckowledgement`](../modules.md#pendingackowledgement)

#### Defined in

[db.ts:250](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L250)

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

[db.ts:158](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L158)

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

[db.ts:484](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L484)

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

[db.ts:493](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L493)

___

### getAcknowledgedTickets

▸ **getAcknowledgedTickets**(`filter?`): `Promise`<[`AcknowledgedTicket`](AcknowledgedTicket.md)[]\>

Get acknowledged tickets sorted by ticket index in ascending order.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `filter?` | `Object` | optionally filter by signer |
| `filter.channel?` | [`ChannelEntry`](ChannelEntry.md) | - |
| `filter.signer?` | [`PublicKey`](PublicKey.md) | - |

#### Returns

`Promise`<[`AcknowledgedTicket`](AcknowledgedTicket.md)[]\>

an array of all acknowledged tickets

#### Defined in

[db.ts:320](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L320)

___

### getAll

▸ `Private` **getAll**<`T`\>(`prefix`, `deserialize`, `filter?`, `sorter?`): `Promise`<`T`[]\>

#### Type parameters

| Name |
| :------ |
| `T` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `prefix` | `Uint8Array` |
| `deserialize` | (`u`: `Uint8Array`) => `T` |
| `filter?` | (`o`: `T`) => `boolean` |
| `sorter?` | (`e1`: `T`, `e2`: `T`) => `number` |

#### Returns

`Promise`<`T`[]\>

#### Defined in

[db.ts:186](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L186)

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

[db.ts:472](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L472)

___

### getChannelFrom

▸ **getChannelFrom**(`src`): `Promise`<[`ChannelEntry`](ChannelEntry.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `src` | [`PublicKey`](PublicKey.md) |

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)\>

#### Defined in

[db.ts:570](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L570)

___

### getChannelTo

▸ **getChannelTo**(`dest`): `Promise`<[`ChannelEntry`](ChannelEntry.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `dest` | [`PublicKey`](PublicKey.md) |

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)\>

#### Defined in

[db.ts:566](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L566)

___

### getChannelX

▸ **getChannelX**(`src`, `dest`): `Promise`<[`ChannelEntry`](ChannelEntry.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `src` | [`PublicKey`](PublicKey.md) |
| `dest` | [`PublicKey`](PublicKey.md) |

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)\>

#### Defined in

[db.ts:562](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L562)

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

[db.ts:476](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L476)

___

### getChannelsFrom

▸ **getChannelsFrom**(`address`): `Promise`<[`ChannelEntry`](ChannelEntry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | [`Address`](Address.md) |

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)[]\>

#### Defined in

[db.ts:574](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L574)

___

### getChannelsTo

▸ **getChannelsTo**(`address`): `Promise`<[`ChannelEntry`](ChannelEntry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | [`Address`](Address.md) |

#### Returns

`Promise`<[`ChannelEntry`](ChannelEntry.md)[]\>

#### Defined in

[db.ts:580](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L580)

___

### getCoerced

▸ `Private` **getCoerced**<`T`\>(`key`, `coerce`): `Promise`<`T`\>

#### Type parameters

| Name |
| :------ |
| `T` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | `Uint8Array` |
| `coerce` | (`u`: `Uint8Array`) => `T` |

#### Returns

`Promise`<`T`\>

#### Defined in

[db.ts:173](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L173)

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

[db.ts:178](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L178)

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

[db.ts:425](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L425)

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

[db.ts:429](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L429)

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

[db.ts:440](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L440)

___

### getEnvironmentId

▸ **getEnvironmentId**(): `Promise`<`string`\>

#### Returns

`Promise`<`string`\>

#### Defined in

[db.ts:590](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L590)

___

### getHoprBalance

▸ **getHoprBalance**(): `Promise`<[`Balance`](Balance.md)\>

#### Returns

`Promise`<[`Balance`](Balance.md)\>

#### Defined in

[db.ts:604](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L604)

___

### getLatestBlockNumber

▸ **getLatestBlockNumber**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db.ts:455](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L455)

___

### getLatestConfirmedSnapshotOrUndefined

▸ **getLatestConfirmedSnapshotOrUndefined**(): `Promise`<[`Snapshot`](Snapshot.md)\>

#### Returns

`Promise`<[`Snapshot`](Snapshot.md)\>

#### Defined in

[db.ts:464](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L464)

___

### getLosingTicketCount

▸ **getLosingTicketCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db.ts:516](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L516)

___

### getNeglectedTicketsCount

▸ **getNeglectedTicketsCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db.ts:504](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L504)

___

### getPendingAcknowledgement

▸ **getPendingAcknowledgement**(`halfKeyChallenge`): `Promise`<[`PendingAckowledgement`](../modules.md#pendingackowledgement)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `halfKeyChallenge` | [`HalfKeyChallenge`](HalfKeyChallenge.md) |

#### Returns

`Promise`<[`PendingAckowledgement`](../modules.md#pendingackowledgement)\>

#### Defined in

[db.ts:291](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L291)

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

[db.ts:512](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L512)

___

### getPendingTicketCount

▸ **getPendingTicketCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db.ts:508](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L508)

___

### getRedeemedTicketsCount

▸ **getRedeemedTicketsCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db.ts:500](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L500)

___

### getRedeemedTicketsValue

▸ **getRedeemedTicketsValue**(): `Promise`<[`Balance`](Balance.md)\>

#### Returns

`Promise`<[`Balance`](Balance.md)\>

#### Defined in

[db.ts:497](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L497)

___

### getRejectedTicketsCount

▸ **getRejectedTicketsCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db.ts:544](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L544)

___

### getRejectedTicketsValue

▸ **getRejectedTicketsValue**(): `Promise`<[`Balance`](Balance.md)\>

#### Returns

`Promise`<[`Balance`](Balance.md)\>

#### Defined in

[db.ts:541](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L541)

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

[db.ts:383](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L383)

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

[db.ts:269](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L269)

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

[db.ts:136](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L136)

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

[db.ts:226](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L226)

___

### init

▸ **init**(`initialize`, `dbPath`, `forceCreate?`, `environmentId`): `Promise`<`void`\>

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `initialize` | `boolean` | `undefined` |
| `dbPath` | `string` | `undefined` |
| `forceCreate` | `boolean` | `false` |
| `environmentId` | `string` | `undefined` |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:79](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L79)

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

[db.ts:132](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L132)

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

[db.ts:535](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L535)

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

[db.ts:520](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L520)

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

[db.ts:528](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L528)

___

### markRejected

▸ **markRejected**(`t`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `t` | [`Ticket`](Ticket.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:547](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L547)

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

[db.ts:162](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L162)

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

[db.ts:150](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L150)

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

[db.ts:366](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L366)

___

### resolvePending

▸ **resolvePending**(`ticket`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ticket` | `Partial`<[`Ticket`](Ticket.md)\> |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:524](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L524)

___

### serializePendingAcknowledgement

▸ `Private` **serializePendingAcknowledgement**(`isMessageSender`, `unackTicket?`): `Uint8Array`

#### Parameters

| Name | Type |
| :------ | :------ |
| `isMessageSender` | `boolean` |
| `unackTicket?` | [`UnacknowledgedTicket`](UnacknowledgedTicket.md) |

#### Returns

`Uint8Array`

#### Defined in

[db.ts:242](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L242)

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

[db.ts:433](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L433)

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

[db.ts:448](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L448)

___

### setEnvironmentId

▸ **setEnvironmentId**(`environment_id`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `environment_id` | `string` |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:586](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L586)

___

### setHoprBalance

▸ **setHoprBalance**(`value`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `value` | [`Balance`](Balance.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:608](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L608)

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

[db.ts:415](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L415)

___

### storePendingAcknowledgement

▸ **storePendingAcknowledgement**(`halfKeyChallenge`, `isMessageSender`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `halfKeyChallenge` | [`HalfKeyChallenge`](HalfKeyChallenge.md) |
| `isMessageSender` | ``true`` |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:297](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L297)

▸ **storePendingAcknowledgement**(`halfKeyChallenge`, `isMessageSender`, `unackTicket`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `halfKeyChallenge` | [`HalfKeyChallenge`](HalfKeyChallenge.md) |
| `isMessageSender` | ``false`` |
| `unackTicket` | [`UnacknowledgedTicket`](UnacknowledgedTicket.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:298](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L298)

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

[db.ts:237](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L237)

___

### subHoprBalance

▸ **subHoprBalance**(`value`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `value` | [`Balance`](Balance.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db.ts:616](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L616)

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

[db.ts:154](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L154)

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

[db.ts:489](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L489)

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

[db.ts:480](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L480)

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

[db.ts:460](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L460)

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

[db.ts:468](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L468)

___

### verifyEnvironmentId

▸ **verifyEnvironmentId**(`expectedId`): `Promise`<`boolean`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `expectedId` | `string` |

#### Returns

`Promise`<`boolean`\>

#### Defined in

[db.ts:594](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L594)

___

### createMock

▸ `Static` **createMock**(`id?`): [`HoprDB`](HoprDB.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `id?` | [`PublicKey`](PublicKey.md) |

#### Returns

[`HoprDB`](HoprDB.md)

#### Defined in

[db.ts:552](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L552)
