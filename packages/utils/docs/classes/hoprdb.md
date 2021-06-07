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
- [getCommitment](hoprdb.md#getcommitment)
- [getCurrentCommitment](hoprdb.md#getcurrentcommitment)
- [getCurrentTicketIndex](hoprdb.md#getcurrentticketindex)
- [getLatestBlockNumber](hoprdb.md#getlatestblocknumber)
- [getLatestConfirmedSnapshot](hoprdb.md#getlatestconfirmedsnapshot)
- [getTickets](hoprdb.md#gettickets)
- [getUnacknowledgedTicket](hoprdb.md#getunacknowledgedticket)
- [getUnacknowledgedTickets](hoprdb.md#getunacknowledgedtickets)
- [has](hoprdb.md#has)
- [keyOf](hoprdb.md#keyof)
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

[db.ts:46](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L46)

## Properties

### db

• `Private` **db**: `LevelUp`<AbstractLevelDOWN<any, any\>, AbstractIterator<any, any\>\>

#### Defined in

[db.ts:46](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L46)

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

[db.ts:225](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L225)

___

### close

▸ **close**(): `Promise`<void\>

#### Returns

`Promise`<void\>

#### Defined in

[db.ts:235](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L235)

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

[db.ts:133](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L133)

___

### delAcknowledgedTicket

▸ **delAcknowledgedTicket**(`challenge`): `Promise`<void\>

Delete acknowledged ticket in database

#### Parameters

| Name | Type |
| :------ | :------ |
| `challenge` | [EthereumChallenge](ethereumchallenge.md) |

#### Returns

`Promise`<void\>

#### Defined in

[db.ts:190](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L190)

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

[db.ts:94](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L94)

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

[db.ts:315](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L315)

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

[db.ts:324](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L324)

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

[db.ts:174](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L174)

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

[db.ts:109](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L109)

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

[db.ts:301](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L301)

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

[db.ts:306](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L306)

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

[db.ts:249](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L249)

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

[db.ts:253](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L253)

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

[db.ts:264](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L264)

___

### getLatestBlockNumber

▸ **getLatestBlockNumber**(): `Promise`<number\>

#### Returns

`Promise`<number\>

#### Defined in

[db.ts:283](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L283)

___

### getLatestConfirmedSnapshot

▸ **getLatestConfirmedSnapshot**(): `Promise`<[Snapshot](snapshot.md)\>

#### Returns

`Promise`<[Snapshot](snapshot.md)\>

#### Defined in

[db.ts:292](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L292)

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

[db.ts:215](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L215)

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

[db.ts:158](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L158)

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

[db.ts:142](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L142)

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

[db.ts:72](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L72)

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

[db.ts:68](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L68)

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

[db.ts:98](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L98)

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

[db.ts:86](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L86)

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

[db.ts:194](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L194)

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

[db.ts:257](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L257)

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

[db.ts:276](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L276)

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

[db.ts:239](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L239)

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

[db.ts:162](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L162)

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

[db.ts:90](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L90)

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

[db.ts:320](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L320)

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

[db.ts:311](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L311)

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

[db.ts:288](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L288)

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

[db.ts:297](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L297)

___

### createMock

▸ `Static` **createMock**(): [HoprDB](hoprdb.md)

#### Returns

[HoprDB](hoprdb.md)

#### Defined in

[db.ts:329](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L329)
