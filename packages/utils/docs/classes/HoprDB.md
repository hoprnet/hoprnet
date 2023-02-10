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
- [deleteObject](HoprDB.md#deleteobject)
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
- [getSerializedObject](HoprDB.md#getserializedobject)
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
- [putSerializedObject](HoprDB.md#putserializedobject)
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

[src/db/db.ts:153](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L153)

## Properties

### db

• `Private` **db**: `LevelUp`

#### Defined in

[src/db/db.ts:151](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L151)

___

### id

• `Private` **id**: [`PublicKey`](PublicKey.md)

#### Defined in

[src/db/db.ts:153](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L153)

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

[src/db/db.ts:397](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L397)

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

[src/db/db.ts:881](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L881)

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

[src/db/db.ts:918](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L918)

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

[src/db/db.ts:577](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L577)

___

### close

▸ **close**(): `Promise`<`any`\>

#### Returns

`Promise`<`any`\>

#### Defined in

[src/db/db.ts:587](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L587)

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

[src/db/db.ts:385](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L385)

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

[src/db/db.ts:532](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L532)

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

[src/db/db.ts:504](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L504)

___

### deleteObject

▸ **deleteObject**(`namespace`, `key`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `namespace` | `string` |
| `key` | `string` |

#### Returns

`Promise`<`void`\>

#### Defined in

[src/db/db.ts:1139](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L1139)

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

[src/db/db.ts:237](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L237)

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

[src/db/db.ts:974](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L974)

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

[src/db/db.ts:273](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L273)

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

[src/db/db.ts:694](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L694)

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

[src/db/db.ts:1037](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L1037)

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

[src/db/db.ts:715](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L715)

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

[src/db/db.ts:726](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L726)

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

[src/db/db.ts:459](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L459)

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

[src/db/db.ts:315](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L315)

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

[src/db/db.ts:355](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L355)

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

[src/db/db.ts:649](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L649)

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

[src/db/db.ts:823](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L823)

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

[src/db/db.ts:819](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L819)

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

[src/db/db.ts:815](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L815)

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

[src/db/db.ts:664](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L664)

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

[src/db/db.ts:827](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L827)

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

[src/db/db.ts:833](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L833)

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

[src/db/db.ts:653](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L653)

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

[src/db/db.ts:841](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L841)

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

[src/db/db.ts:847](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L847)

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

[src/db/db.ts:291](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L291)

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

[src/db/db.ts:296](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L296)

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

[src/db/db.ts:606](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L606)

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

[src/db/db.ts:610](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L610)

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

[src/db/db.ts:618](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L618)

___

### getEnvironmentId

▸ **getEnvironmentId**(): `Promise`<`string`\>

#### Returns

`Promise`<`string`\>

#### Defined in

[src/db/db.ts:859](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L859)

___

### getHoprBalance

▸ **getHoprBalance**(): `Promise`<[`Balance`](Balance.md)\>

#### Returns

`Promise`<[`Balance`](Balance.md)\>

#### Defined in

[src/db/db.ts:873](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L873)

___

### getLatestBlockNumber

▸ **getLatestBlockNumber**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[src/db/db.ts:630](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L630)

___

### getLatestConfirmedSnapshotOrUndefined

▸ **getLatestConfirmedSnapshotOrUndefined**(): `Promise`<[`Snapshot`](Snapshot.md)\>

#### Returns

`Promise`<[`Snapshot`](Snapshot.md)\>

#### Defined in

[src/db/db.ts:641](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L641)

___

### getLosingTicketCount

▸ **getLosingTicketCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[src/db/db.ts:760](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L760)

___

### getNeglectedTicketsCount

▸ **getNeglectedTicketsCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[src/db/db.ts:744](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L744)

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

[src/db/db.ts:436](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L436)

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

[src/db/db.ts:752](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L752)

___

### getPendingTicketCount

▸ **getPendingTicketCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[src/db/db.ts:748](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L748)

___

### getRedeemedTicketsCount

▸ **getRedeemedTicketsCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[src/db/db.ts:740](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L740)

___

### getRedeemedTicketsValue

▸ **getRedeemedTicketsValue**(): `Promise`<[`Balance`](Balance.md)\>

#### Returns

`Promise`<[`Balance`](Balance.md)\>

#### Defined in

[src/db/db.ts:737](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L737)

___

### getRejectedTicketsCount

▸ **getRejectedTicketsCount**(): `Promise`<`number`\>

#### Returns

`Promise`<`number`\>

#### Defined in

[src/db/db.ts:807](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L807)

___

### getRejectedTicketsValue

▸ **getRejectedTicketsValue**(): `Promise`<[`Balance`](Balance.md)\>

#### Returns

`Promise`<[`Balance`](Balance.md)\>

#### Defined in

[src/db/db.ts:804](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L804)

___

### getSerializedObject

▸ **getSerializedObject**(`namespace`, `key`): `Promise`<`Uint8Array`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `namespace` | `string` |
| `key` | `string` |

#### Returns

`Promise`<`Uint8Array`\>

#### Defined in

[src/db/db.ts:1131](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L1131)

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

[src/db/db.ts:559](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L559)

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

[src/db/db.ts:412](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L412)

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

[src/db/db.ts:215](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L215)

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

[src/db/db.ts:390](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L390)

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

[src/db/db.ts:155](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L155)

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

[src/db/db.ts:1078](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L1078)

___

### isNetworkRegistryEnabled

▸ **isNetworkRegistryEnabled**(): `Promise`<`boolean`\>

Check ifs Network registry is enabled

#### Returns

`Promise`<`boolean`\>

true if register is enabled or if key is not preset in the dababase

#### Defined in

[src/db/db.ts:1103](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L1103)

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

[src/db/db.ts:211](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L211)

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

[src/db/db.ts:798](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L798)

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

[src/db/db.ts:764](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L764)

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

[src/db/db.ts:791](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L791)

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

[src/db/db.ts:810](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L810)

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

[src/db/db.ts:280](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L280)

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

[src/db/db.ts:229](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L229)

___

### putSerializedObject

▸ **putSerializedObject**(`namespace`, `key`, `object`): `Promise`<`void`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `namespace` | `string` |
| `key` | `string` |
| `object` | `Uint8Array` |

#### Returns

`Promise`<`void`\>

#### Defined in

[src/db/db.ts:1123](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L1123)

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

[src/db/db.ts:995](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L995)

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

[src/db/db.ts:536](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L536)

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

[src/db/db.ts:768](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L768)

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

[src/db/db.ts:614](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L614)

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

[src/db/db.ts:626](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L626)

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

[src/db/db.ts:1047](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L1047)

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

[src/db/db.ts:855](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L855)

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

[src/db/db.ts:877](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L877)

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

[src/db/db.ts:1086](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L1086)

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

[src/db/db.ts:592](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L592)

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

[src/db/db.ts:443](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L443)

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

[src/db/db.ts:402](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L402)

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

[src/db/db.ts:896](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L896)

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

[src/db/db.ts:269](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L269)

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

[src/db/db.ts:698](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L698)

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

[src/db/db.ts:675](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L675)

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

[src/db/db.ts:637](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L637)

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

[src/db/db.ts:645](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L645)

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

[src/db/db.ts:863](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L863)

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

[src/db/db.ts:1107](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db/db.ts#L1107)
