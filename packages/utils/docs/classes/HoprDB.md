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

[db/db.ts:315](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L315)

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

[db/db.ts:706](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L706)

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

[db/db.ts:733](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L733)

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

[db/db.ts:476](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L476)

___

### close

▸ **close**(): `Promise`<`void`\>

#### Returns

`Promise`<`void`\>

#### Defined in

[db/db.ts:486](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L486)

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

[db/db.ts:305](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L305)

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

[db/db.ts:437](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L437)

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

[db/db.ts:417](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L417)

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

[db/db.ts:751](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L751)

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

[db/db.ts:226](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L226)

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

[db/db.ts:565](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L565)

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

[db/db.ts:791](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L791)

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

[db/db.ts:577](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L577)

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

[db/db.ts:381](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L381)

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

[db/db.ts:265](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L265)

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

[db/db.ts:542](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L542)

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

[db/db.ts:664](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L664)

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

[db/db.ts:660](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L660)

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

[db/db.ts:656](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L656)

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

[db/db.ts:546](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L546)

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

[db/db.ts:668](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L668)

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

[db/db.ts:674](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L674)

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

[db/db.ts:241](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L241)

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

[db/db.ts:246](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L246)

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

[db/db.ts:503](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L503)

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

[db/db.ts:507](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L507)

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

[db/db.ts:515](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L515)

___

### getEnvironmentId

▸ **getEnvironmentId**(): `Promise`<`string`\>

#### Returns

`Promise`<`string`\>

#### Defined in

[db/db.ts:684](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L684)

___

### getHoprBalance

▸ **getHoprBalance**(): `Promise`<[`Balance`](Balance.md)\>

#### Returns

`Promise`<[`Balance`](Balance.md)\>

#### Defined in

[db/db.ts:698](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L698)

___

### getLatestBlockNumber

▸ **getLatestBlockNumber**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db/db.ts:523](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L523)

___

### getLatestConfirmedSnapshotOrUndefined

▸ **getLatestConfirmedSnapshotOrUndefined**(): `Promise`<[`Snapshot`](Snapshot.md)\>

#### Returns

`Promise`<[`Snapshot`](Snapshot.md)\>

#### Defined in

[db/db.ts:534](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L534)

___

### getLosingTicketCount

▸ **getLosingTicketCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db/db.ts:607](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L607)

___

### getNeglectedTicketsCount

▸ **getNeglectedTicketsCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db/db.ts:595](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L595)

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

[db/db.ts:354](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L354)

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

[db/db.ts:603](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L603)

___

### getPendingTicketCount

▸ **getPendingTicketCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db/db.ts:599](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L599)

___

### getRedeemedTicketsCount

▸ **getRedeemedTicketsCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db/db.ts:591](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L591)

___

### getRedeemedTicketsValue

▸ **getRedeemedTicketsValue**(): `Promise`<[`Balance`](Balance.md)\>

#### Returns

`Promise`<[`Balance`](Balance.md)\>

#### Defined in

[db/db.ts:588](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L588)

___

### getRejectedTicketsCount

▸ **getRejectedTicketsCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[db/db.ts:648](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L648)

___

### getRejectedTicketsValue

▸ **getRejectedTicketsValue**(): `Promise`<[`Balance`](Balance.md)\>

#### Returns

`Promise`<[`Balance`](Balance.md)\>

#### Defined in

[db/db.ts:645](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L645)

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

[db/db.ts:458](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L458)

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

[db/db.ts:330](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L330)

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

[db/db.ts:204](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L204)

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

[db/db.ts:309](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L309)

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

[db/db.ts:824](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L824)

___

### isNetworkRegistryEnabled

▸ **isNetworkRegistryEnabled**(): `Promise`<`boolean`\>

Hopr Network Registry

#### Returns

`Promise`<`boolean`\>

true if register is enabled

#### Defined in

[db/db.ts:844](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L844)

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

[db/db.ts:200](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L200)

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

[db/db.ts:639](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L639)

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

[db/db.ts:611](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L611)

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

[db/db.ts:632](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L632)

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

[db/db.ts:651](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L651)

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

[db/db.ts:230](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L230)

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

[db/db.ts:218](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L218)

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

[db/db.ts:771](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L771)

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

[db/db.ts:441](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L441)

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

[db/db.ts:615](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L615)

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

[db/db.ts:511](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L511)

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

[db/db.ts:519](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L519)

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

[db/db.ts:801](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L801)

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

[db/db.ts:680](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L680)

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

[db/db.ts:702](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L702)

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

[db/db.ts:832](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L832)

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

[db/db.ts:491](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L491)

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

[db/db.ts:358](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L358)

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

[db/db.ts:359](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L359)

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

[db/db.ts:320](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L320)

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

[db/db.ts:716](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L716)

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

[db/db.ts:222](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L222)

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

[db/db.ts:569](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L569)

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

[db/db.ts:557](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L557)

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

[db/db.ts:530](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L530)

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

[db/db.ts:538](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L538)

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

[db/db.ts:688](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L688)

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

[db/db.ts:848](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L848)
