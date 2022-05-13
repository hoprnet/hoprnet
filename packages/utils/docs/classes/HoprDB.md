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
- [addToNetworkRegistry](HoprDB.md#addtonetworkregistry)
- [checkAndSetPacketTag](HoprDB.md#checkandsetpackettag)
- [close](HoprDB.md#close)
- [del](HoprDB.md#del)
- [delAcknowledgedTicket](HoprDB.md#delacknowledgedticket)
- [deleteAcknowledgedTicketsFromChannel](HoprDB.md#deleteacknowledgedticketsfromchannel)
- [findHoprNodeUsingAccountInNetworkRegistry](HoprDB.md#findhoprnodeusingaccountinnetworkregistry)
- [get](HoprDB.md#get)
- [getAccount](HoprDB.md#getaccount)
- [getAccountFromNetworkRegistry](HoprDB.md#getaccountfromnetworkregistry)
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
- [isEligible](HoprDB.md#iseligible)
- [isNetworkRegistryEnabled](HoprDB.md#isnetworkregistryenabled)
- [keyOf](HoprDB.md#keyof)
- [markLosing](HoprDB.md#marklosing)
- [markPending](HoprDB.md#markpending)
- [markRedeemeed](HoprDB.md#markredeemeed)
- [markRejected](HoprDB.md#markrejected)
- [maybeGet](HoprDB.md#maybeget)
- [put](HoprDB.md#put)
- [removeFromNetworkRegistry](HoprDB.md#removefromnetworkregistry)
- [replaceUnAckWithAck](HoprDB.md#replaceunackwithack)
- [resolvePending](HoprDB.md#resolvepending)
- [setCurrentCommitment](HoprDB.md#setcurrentcommitment)
- [setCurrentTicketIndex](HoprDB.md#setcurrentticketindex)
- [setEligible](HoprDB.md#seteligible)
- [setEnvironmentId](HoprDB.md#setenvironmentid)
- [setHoprBalance](HoprDB.md#sethoprbalance)
- [setNetworkRegistryEnabled](HoprDB.md#setnetworkregistryenabled)
- [storeHashIntermediaries](HoprDB.md#storehashintermediaries)
- [storePendingAcknowledgement](HoprDB.md#storependingacknowledgement)
- [subBalance](HoprDB.md#subbalance)
- [subHoprBalance](HoprDB.md#subhoprbalance)
- [touch](HoprDB.md#touch)
- [updateAccountAndSnapshot](HoprDB.md#updateaccountandsnapshot)
- [updateChannelAndSnapshot](HoprDB.md#updatechannelandsnapshot)
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

[db/db.ts:143](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L143)

## Properties

### db

• `Private` **db**: `LevelUp`<`AbstractLevelDOWN`<`any`, `any`\>, `AbstractIterator`<`any`, `any`\>\>

#### Defined in

[db/db.ts:141](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L141)

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

[db/db.ts:313](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L313)

___

### addHoprBalance

▸ **addHoprBalance**(`value`, `snapshot`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `value` | [`Balance`](Balance.md) |
| `snapshot` | [`Snapshot`](Snapshot.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db/db.ts:704](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L704)

___

### addToNetworkRegistry

▸ **addToNetworkRegistry**(`pubKey`, `account`, `snapshot`): `Promise`<`void`\>

Hopr Network Registry
Link hoprNode to an ETH address.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `pubKey` | [`PublicKey`](PublicKey.md) | the node to register |
| `account` | [`Address`](Address.md) | the account that made the transaction |
| `snapshot` | [`Snapshot`](Snapshot.md) |  |

#### Returns

`Promise`<`void`\>

#### Defined in

[db/db.ts:731](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L731)

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

[db/db.ts:474](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L474)

___

### close

▸ **close**(): `Promise`<`void`\>

#### Returns

`Promise`<`void`\>

#### Defined in

[db/db.ts:484](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L484)

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

[db/db.ts:303](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L303)

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

[db/db.ts:435](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L435)

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

[db/db.ts:415](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L415)

___

### findHoprNodeUsingAccountInNetworkRegistry

▸ **findHoprNodeUsingAccountInNetworkRegistry**(`account`): `Promise`<[`PublicKey`](PublicKey.md)\>

Do a reverse find by searching the stored account to return
the associated public key of the HoprNode.

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | [`Address`](Address.md) |

#### Returns

`Promise`<[`PublicKey`](PublicKey.md)\>

PublicKey of the associated HoprNode

#### Defined in

[db/db.ts:749](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L749)

___

### get

▸ `Protected` **get**(`key`): `Promise`<`Uint8Array`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | `Uint8Array` |

#### Returns

`Promise`<`Uint8Array`\>

#### Defined in

[db/db.ts:224](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L224)

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

[db/db.ts:563](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L563)

___

### getAccountFromNetworkRegistry

▸ **getAccountFromNetworkRegistry**(`hoprNode`): `Promise`<[`Address`](Address.md)\>

Hopr Network Registry
Get address associated with hoprNode.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `hoprNode` | [`PublicKey`](PublicKey.md) | the node to register |

#### Returns

`Promise`<[`Address`](Address.md)\>

ETH address

#### Defined in

[db/db.ts:789](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L789)

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

[db/db.ts:575](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L575)

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

[db/db.ts:379](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L379)

___

### getAll

▸ `Protected` **getAll**<`Element`, `TransformedElement`\>(`range`, `deserialize`, `filter?`, `map?`, `sorter?`): `Promise`<`TransformedElement`[]\>

Gets a elements from the database of a kind.
Optionally applies `filter`then `map` then `sort` to the result.

#### Type parameters

| Name | Type |
| :------ | :------ |
| `Element` | `Element` |
| `TransformedElement` | `Element` |

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `range` | `Object` | - |
| `range.prefix` | `Uint8Array` | key prefix, such as `channels-` |
| `range.suffixLength` | `number` | length of the appended identifier to distinguish elements |
| `deserialize` | (`u`: `Uint8Array`) => `Element` | function to parse serialized objects |
| `filter?` | (`o`: `Element`) => `boolean` | filter deserialized objects |
| `map?` | (`i`: `Element`) => `TransformedElement` | transform deserialized and filtered objects |
| `sorter?` | (`e1`: `TransformedElement`, `e2`: `TransformedElement`) => `number` | sort deserialized, filtered and transformed objects |

#### Returns

`Promise`<`TransformedElement`[]\>

a Promises that resolves with the found elements

#### Defined in

[db/db.ts:263](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L263)

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

[db/db.ts:540](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L540)

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

[db/db.ts:662](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L662)

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

[db/db.ts:658](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L658)

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

[db/db.ts:654](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L654)

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

[db/db.ts:544](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L544)

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

[db/db.ts:666](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L666)

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

[db/db.ts:672](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L672)

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

[db/db.ts:239](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L239)

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

[db/db.ts:244](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L244)

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

[db/db.ts:501](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L501)

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

[db/db.ts:505](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L505)

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

[db/db.ts:513](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L513)

___

### getEnvironmentId

▸ **getEnvironmentId**(): `Promise`<`string`\>

#### Returns

`Promise`<`string`\>

#### Defined in

[db/db.ts:682](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L682)

___

### getHoprBalance

▸ **getHoprBalance**(): `Promise`<[`Balance`](Balance.md)\>

#### Returns

`Promise`<[`Balance`](Balance.md)\>

#### Defined in

[db/db.ts:696](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L696)

___

### getLatestBlockNumber

▸ **getLatestBlockNumber**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db/db.ts:521](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L521)

___

### getLatestConfirmedSnapshotOrUndefined

▸ **getLatestConfirmedSnapshotOrUndefined**(): `Promise`<[`Snapshot`](Snapshot.md)\>

#### Returns

`Promise`<[`Snapshot`](Snapshot.md)\>

#### Defined in

[db/db.ts:532](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L532)

___

### getLosingTicketCount

▸ **getLosingTicketCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db/db.ts:605](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L605)

___

### getNeglectedTicketsCount

▸ **getNeglectedTicketsCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db/db.ts:593](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L593)

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

[db/db.ts:352](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L352)

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

[db/db.ts:601](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L601)

___

### getPendingTicketCount

▸ **getPendingTicketCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db/db.ts:597](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L597)

___

### getRedeemedTicketsCount

▸ **getRedeemedTicketsCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db/db.ts:589](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L589)

___

### getRedeemedTicketsValue

▸ **getRedeemedTicketsValue**(): `Promise`<[`Balance`](Balance.md)\>

#### Returns

`Promise`<[`Balance`](Balance.md)\>

#### Defined in

[db/db.ts:586](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L586)

___

### getRejectedTicketsCount

▸ **getRejectedTicketsCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db/db.ts:646](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L646)

___

### getRejectedTicketsValue

▸ **getRejectedTicketsValue**(): `Promise`<[`Balance`](Balance.md)\>

#### Returns

`Promise`<[`Balance`](Balance.md)\>

#### Defined in

[db/db.ts:643](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L643)

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

[db/db.ts:456](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L456)

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

[db/db.ts:328](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L328)

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

[db/db.ts:202](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L202)

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

[db/db.ts:307](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L307)

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

[db/db.ts:145](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L145)

___

### isEligible

▸ **isEligible**(`account`): `Promise`<`boolean`\>

Hopr Network Registry

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `account` | [`Address`](Address.md) | the account that made the transaction |

#### Returns

`Promise`<`boolean`\>

true if account is eligible

#### Defined in

[db/db.ts:822](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L822)

___

### isNetworkRegistryEnabled

▸ **isNetworkRegistryEnabled**(): `Promise`<`boolean`\>

Hopr Network Registry

#### Returns

`Promise`<`boolean`\>

true if register is enabled

#### Defined in

[db/db.ts:842](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L842)

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

[db/db.ts:198](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L198)

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

[db/db.ts:637](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L637)

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

[db/db.ts:609](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L609)

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

[db/db.ts:630](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L630)

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

[db/db.ts:649](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L649)

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

[db/db.ts:228](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L228)

___

### put

▸ `Protected` **put**(`key`, `value`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | `Uint8Array` |
| `value` | `Uint8Array` |

#### Returns

`Promise`<`void`\>

#### Defined in

[db/db.ts:216](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L216)

___

### removeFromNetworkRegistry

▸ **removeFromNetworkRegistry**(`account`, `snapshot`): `Promise`<`void`\>

Hopr Network Registry
Unlink hoprNode to an ETH address by removing the entry.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `account` | [`Address`](Address.md) | the account to use so we can search for the key in the database |
| `snapshot` | [`Snapshot`](Snapshot.md) |  |

#### Returns

`Promise`<`void`\>

#### Defined in

[db/db.ts:769](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L769)

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

[db/db.ts:439](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L439)

___

### resolvePending

▸ **resolvePending**(`ticket`, `snapshot`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `ticket` | `Partial`<[`Ticket`](Ticket.md)\> |
| `snapshot` | [`Snapshot`](Snapshot.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db/db.ts:613](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L613)

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

[db/db.ts:509](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L509)

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

[db/db.ts:517](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L517)

___

### setEligible

▸ **setEligible**(`account`, `eligible`, `snapshot`): `Promise`<`void`\>

Hopr Network Registry
Set address as eligible.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `account` | [`Address`](Address.md) | the account that made the transaction |
| `eligible` | `boolean` | - |
| `snapshot` | [`Snapshot`](Snapshot.md) |  |

#### Returns

`Promise`<`void`\>

#### Defined in

[db/db.ts:799](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L799)

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

[db/db.ts:678](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L678)

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

[db/db.ts:700](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L700)

___

### setNetworkRegistryEnabled

▸ **setNetworkRegistryEnabled**(`enabled`, `snapshot`): `Promise`<`void`\>

Hopr Network Registry

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `enabled` | `boolean` | whether register is enabled |
| `snapshot` | [`Snapshot`](Snapshot.md) | - |

#### Returns

`Promise`<`void`\>

#### Defined in

[db/db.ts:830](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L830)

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

[db/db.ts:489](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L489)

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

[db/db.ts:356](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L356)

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

[db/db.ts:357](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L357)

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

[db/db.ts:318](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L318)

___

### subHoprBalance

▸ **subHoprBalance**(`value`, `snapshot`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `value` | [`Balance`](Balance.md) |
| `snapshot` | [`Snapshot`](Snapshot.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db/db.ts:714](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L714)

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

[db/db.ts:220](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L220)

___

### updateAccountAndSnapshot

▸ **updateAccountAndSnapshot**(`account`, `snapshot`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | [`AccountEntry`](AccountEntry.md) |
| `snapshot` | [`Snapshot`](Snapshot.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db/db.ts:567](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L567)

___

### updateChannelAndSnapshot

▸ **updateChannelAndSnapshot**(`channelId`, `channel`, `snapshot`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [`Hash`](Hash.md) |
| `channel` | [`ChannelEntry`](ChannelEntry.md) |
| `snapshot` | [`Snapshot`](Snapshot.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[db/db.ts:555](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L555)

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

[db/db.ts:528](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L528)

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

[db/db.ts:536](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L536)

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

[db/db.ts:686](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L686)

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

[db/db.ts:846](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L846)
