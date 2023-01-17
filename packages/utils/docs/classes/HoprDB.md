[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / HoprDB

# Class: HoprDB

## Table of contents

### Constructors

- [constructor](HoprDB.md#constructor)

### Properties

- [db](HoprDB.md#db)
- [id](HoprDB.md#id)

### Methods

- [addBalance](HoprDB.md#addbalance)
- [addHoprBalance](HoprDB.md#addhoprbalance)
- [addToNetworkRegistry](HoprDB.md#addtonetworkregistry)
- [checkAndSetPacketTag](HoprDB.md#checkandsetpackettag)
- [close](HoprDB.md#close)
- [del](HoprDB.md#del)
- [delAcknowledgedTicket](HoprDB.md#delacknowledgedticket)
- [deleteAcknowledgedTicketsFromChannel](HoprDB.md#deleteacknowledgedticketsfromchannel)
- [dumpDatabase](HoprDB.md#dumpdatabase)
- [findHoprNodesUsingAccountInNetworkRegistry](HoprDB.md#findhoprnodesusingaccountinnetworkregistry)
- [get](HoprDB.md#get)
- [getAccount](HoprDB.md#getaccount)
- [getAccountFromNetworkRegistry](HoprDB.md#getaccountfromnetworkregistry)
- [getAccounts](HoprDB.md#getaccounts)
- [getAccountsIterable](HoprDB.md#getaccountsiterable)
- [getAcknowledgedTickets](HoprDB.md#getacknowledgedtickets)
- [getAll](HoprDB.md#getall)
- [getAllIterable](HoprDB.md#getalliterable)
- [getChannel](HoprDB.md#getchannel)
- [getChannelFrom](HoprDB.md#getchannelfrom)
- [getChannelTo](HoprDB.md#getchannelto)
- [getChannelX](HoprDB.md#getchannelx)
- [getChannels](HoprDB.md#getchannels)
- [getChannelsFrom](HoprDB.md#getchannelsfrom)
- [getChannelsFromIterable](HoprDB.md#getchannelsfromiterable)
- [getChannelsIterable](HoprDB.md#getchannelsiterable)
- [getChannelsTo](HoprDB.md#getchannelsto)
- [getChannelsToIterable](HoprDB.md#getchannelstoiterable)
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

[src/db/db.ts:146](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L146)

## Properties

### db

• `Private` **db**: `LevelUp`

#### Defined in

[src/db/db.ts:144](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L144)

___

### id

• `Private` **id**: [`PublicKey`](PublicKey.md)

#### Defined in

[src/db/db.ts:146](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L146)

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

[src/db/db.ts:390](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L390)

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

[src/db/db.ts:874](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L874)

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

[src/db/db.ts:911](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L911)

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

[src/db/db.ts:570](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L570)

___

### close

▸ **close**(): `Promise`<`any`\>

#### Returns

`Promise`<`any`\>

#### Defined in

[src/db/db.ts:580](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L580)

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

[src/db/db.ts:378](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L378)

___

### delAcknowledgedTicket

▸ **delAcknowledgedTicket**(`ack`): `Promise`<`void`\>

Deletes an acknowledged ticket in database

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `ack` | [`AcknowledgedTicket`](AcknowledgedTicket.md) | acknowledged ticket |

#### Returns

`Promise`<`void`\>

#### Defined in

[src/db/db.ts:525](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L525)

___

### deleteAcknowledgedTicketsFromChannel

▸ **deleteAcknowledgedTicketsFromChannel**(`channel`): `Promise`<`void`\>

Deletes all acknowledged tickets in a channel and updates
neglected tickets counter.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `channel` | [`ChannelEntry`](ChannelEntry.md) | in which channel to delete tickets |

#### Returns

`Promise`<`void`\>

#### Defined in

[src/db/db.ts:497](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L497)

___

### dumpDatabase

▸ **dumpDatabase**(`destFile`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `destFile` | `string` |

#### Returns

`void`

#### Defined in

[src/db/db.ts:230](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L230)

___

### findHoprNodesUsingAccountInNetworkRegistry

▸ **findHoprNodesUsingAccountInNetworkRegistry**(`account`): `Promise`<[`PublicKey`](PublicKey.md)[]\>

Do a reverse find by searching the stored account to return
the associated public keys of registered HOPR nodes.

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | [`Address`](Address.md) |

#### Returns

`Promise`<[`PublicKey`](PublicKey.md)[]\>

array of PublicKey of the associated HOPR nodes

#### Defined in

[src/db/db.ts:967](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L967)

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

[src/db/db.ts:266](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L266)

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

[src/db/db.ts:687](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L687)

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

[src/db/db.ts:1030](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L1030)

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

[src/db/db.ts:708](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L708)

___

### getAccountsIterable

▸ **getAccountsIterable**(`filter?`): `AsyncGenerator`<[`AccountEntry`](AccountEntry.md), `void`, `undefined`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `filter?` | (`account`: [`AccountEntry`](AccountEntry.md)) => `boolean` |

#### Returns

`AsyncGenerator`<[`AccountEntry`](AccountEntry.md), `void`, `undefined`\>

#### Defined in

[src/db/db.ts:719](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L719)

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

[src/db/db.ts:452](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L452)

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
| `filter?` | (`o`: `Element`) => `boolean` | [optional] filter deserialized objects |
| `map?` | (`i`: `Element`) => `TransformedElement` | [optional] transform deserialized and filtered objects |
| `sorter?` | (`e1`: `TransformedElement`, `e2`: `TransformedElement`) => `number` | [optional] sort deserialized, filtered and transformed objects |

#### Returns

`Promise`<`TransformedElement`[]\>

a Promises that resolves with the found elements

#### Defined in

[src/db/db.ts:308](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L308)

___

### getAllIterable

▸ `Protected` **getAllIterable**<`Element`, `TransformedElement`\>(`range`, `deserialize`, `filter?`, `map?`): `AsyncIterable`<`TransformedElement`\>

#### Type parameters

| Name | Type |
| :------ | :------ |
| `Element` | `Element` |
| `TransformedElement` | `Element` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `range` | `Object` |
| `range.prefix` | `Uint8Array` |
| `range.suffixLength` | `number` |
| `deserialize` | (`u`: `Uint8Array`) => `Element` |
| `filter?` | (`o`: `Element`) => `boolean` |
| `map?` | (`i`: `Element`) => `TransformedElement` |

#### Returns

`AsyncIterable`<`TransformedElement`\>

#### Defined in

[src/db/db.ts:348](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L348)

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

[src/db/db.ts:642](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L642)

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

[src/db/db.ts:816](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L816)

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

[src/db/db.ts:812](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L812)

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

[src/db/db.ts:808](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L808)

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

[src/db/db.ts:657](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L657)

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

[src/db/db.ts:820](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L820)

___

### getChannelsFromIterable

▸ **getChannelsFromIterable**(`address`): `AsyncGenerator`<[`ChannelEntry`](ChannelEntry.md), `void`, `unknown`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | [`Address`](Address.md) |

#### Returns

`AsyncGenerator`<[`ChannelEntry`](ChannelEntry.md), `void`, `unknown`\>

#### Defined in

[src/db/db.ts:826](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L826)

___

### getChannelsIterable

▸ **getChannelsIterable**(`filter?`): `AsyncIterable`<[`ChannelEntry`](ChannelEntry.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `filter?` | (`channel`: [`ChannelEntry`](ChannelEntry.md)) => `boolean` |

#### Returns

`AsyncIterable`<[`ChannelEntry`](ChannelEntry.md)\>

#### Defined in

[src/db/db.ts:646](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L646)

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

[src/db/db.ts:834](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L834)

___

### getChannelsToIterable

▸ **getChannelsToIterable**(`address`): `AsyncGenerator`<[`ChannelEntry`](ChannelEntry.md), `void`, `unknown`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | [`Address`](Address.md) |

#### Returns

`AsyncGenerator`<[`ChannelEntry`](ChannelEntry.md), `void`, `unknown`\>

#### Defined in

[src/db/db.ts:840](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L840)

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

[src/db/db.ts:284](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L284)

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

[src/db/db.ts:289](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L289)

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

[src/db/db.ts:599](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L599)

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

[src/db/db.ts:603](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L603)

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

[src/db/db.ts:611](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L611)

___

### getEnvironmentId

▸ **getEnvironmentId**(): `Promise`<`string`\>

#### Returns

`Promise`<`string`\>

#### Defined in

[src/db/db.ts:852](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L852)

___

### getHoprBalance

▸ **getHoprBalance**(): `Promise`<[`Balance`](Balance.md)\>

#### Returns

`Promise`<[`Balance`](Balance.md)\>

#### Defined in

[src/db/db.ts:866](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L866)

___

### getLatestBlockNumber

▸ **getLatestBlockNumber**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[src/db/db.ts:623](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L623)

___

### getLatestConfirmedSnapshotOrUndefined

▸ **getLatestConfirmedSnapshotOrUndefined**(): `Promise`<[`Snapshot`](Snapshot.md)\>

#### Returns

`Promise`<[`Snapshot`](Snapshot.md)\>

#### Defined in

[src/db/db.ts:634](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L634)

___

### getLosingTicketCount

▸ **getLosingTicketCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[src/db/db.ts:753](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L753)

___

### getNeglectedTicketsCount

▸ **getNeglectedTicketsCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[src/db/db.ts:737](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L737)

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

[src/db/db.ts:429](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L429)

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

[src/db/db.ts:745](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L745)

___

### getPendingTicketCount

▸ **getPendingTicketCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[src/db/db.ts:741](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L741)

___

### getRedeemedTicketsCount

▸ **getRedeemedTicketsCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[src/db/db.ts:733](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L733)

___

### getRedeemedTicketsValue

▸ **getRedeemedTicketsValue**(): `Promise`<[`Balance`](Balance.md)\>

#### Returns

`Promise`<[`Balance`](Balance.md)\>

#### Defined in

[src/db/db.ts:730](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L730)

___

### getRejectedTicketsCount

▸ **getRejectedTicketsCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[src/db/db.ts:800](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L800)

___

### getRejectedTicketsValue

▸ **getRejectedTicketsValue**(): `Promise`<[`Balance`](Balance.md)\>

#### Returns

`Promise`<[`Balance`](Balance.md)\>

#### Defined in

[src/db/db.ts:797](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L797)

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

[src/db/db.ts:552](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L552)

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

[src/db/db.ts:405](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L405)

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

[src/db/db.ts:208](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L208)

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

[src/db/db.ts:383](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L383)

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

[src/db/db.ts:148](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L148)

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

[src/db/db.ts:1071](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L1071)

___

### isNetworkRegistryEnabled

▸ **isNetworkRegistryEnabled**(): `Promise`<`boolean`\>

Check ifs Network registry is enabled

#### Returns

`Promise`<`boolean`\>

true if register is enabled or if key is not preset in the dababase

#### Defined in

[src/db/db.ts:1096](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L1096)

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

[src/db/db.ts:204](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L204)

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

[src/db/db.ts:791](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L791)

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

[src/db/db.ts:757](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L757)

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

[src/db/db.ts:784](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L784)

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

[src/db/db.ts:803](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L803)

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

[src/db/db.ts:273](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L273)

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

[src/db/db.ts:222](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L222)

___

### removeFromNetworkRegistry

▸ **removeFromNetworkRegistry**(`pubKey`, `account`, `snapshot`): `Promise`<`void`\>

Hopr Network Registry
Unlink hoprNode to an ETH address by removing the entry.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `pubKey` | [`PublicKey`](PublicKey.md) | the node's x |
| `account` | [`Address`](Address.md) | the account to use so we can search for the key in the database |
| `snapshot` | [`Snapshot`](Snapshot.md) |  |

#### Returns

`Promise`<`void`\>

#### Defined in

[src/db/db.ts:988](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L988)

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

[src/db/db.ts:529](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L529)

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

[src/db/db.ts:761](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L761)

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

[src/db/db.ts:607](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L607)

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

[src/db/db.ts:619](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L619)

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

[src/db/db.ts:1040](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L1040)

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

[src/db/db.ts:848](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L848)

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

[src/db/db.ts:870](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L870)

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

[src/db/db.ts:1079](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L1079)

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

[src/db/db.ts:585](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L585)

___

### storePendingAcknowledgement

▸ **storePendingAcknowledgement**(`halfKeyChallenge`, `isMessageSender`, `unackTicket?`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `halfKeyChallenge` | [`HalfKeyChallenge`](HalfKeyChallenge.md) |
| `isMessageSender` | `boolean` |
| `unackTicket?` | [`UnacknowledgedTicket`](UnacknowledgedTicket.md) |

#### Returns

`Promise`<`void`\>

#### Defined in

[src/db/db.ts:436](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L436)

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

[src/db/db.ts:395](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L395)

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

[src/db/db.ts:889](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L889)

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

[src/db/db.ts:262](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L262)

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

[src/db/db.ts:691](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L691)

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

[src/db/db.ts:668](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L668)

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

[src/db/db.ts:630](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L630)

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

[src/db/db.ts:638](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L638)

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

[src/db/db.ts:856](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L856)

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

[src/db/db.ts:1100](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L1100)
