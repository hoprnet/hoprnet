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
- [deleteAcknowledgement](hoprdb.md#deleteacknowledgement)
- [deleteAcknowledgements](hoprdb.md#deleteacknowledgements)
- [deleteTicket](hoprdb.md#deleteticket)
- [deleteTickets](hoprdb.md#deletetickets)
- [deleteUnacknowledgedTickets](hoprdb.md#deleteunacknowledgedtickets)
- [get](hoprdb.md#get)
- [getAccount](hoprdb.md#getaccount)
- [getAccounts](hoprdb.md#getaccounts)
- [getAcknowledgedTickets](hoprdb.md#getacknowledgedtickets)
- [getAll](hoprdb.md#getall)
- [getChannel](hoprdb.md#getchannel)
- [getChannels](hoprdb.md#getchannels)
- [getCommitment](hoprdb.md#getcommitment)
- [getCurrentCommitment](hoprdb.md#getcurrentcommitment)
- [getLatestBlockNumber](hoprdb.md#getlatestblocknumber)
- [getLatestConfirmedSnapshot](hoprdb.md#getlatestconfirmedsnapshot)
- [getTicketCounter](hoprdb.md#getticketcounter)
- [getTickets](hoprdb.md#gettickets)
- [getUnacknowledgedTickets](hoprdb.md#getunacknowledgedtickets)
- [getUnacknowledgedTicketsByKey](hoprdb.md#getunacknowledgedticketsbykey)
- [has](hoprdb.md#has)
- [keyOf](hoprdb.md#keyof)
- [maybeGet](hoprdb.md#maybeget)
- [put](hoprdb.md#put)
- [replaceTicketWithAcknowledgement](hoprdb.md#replaceticketwithacknowledgement)
- [setCurrentCommitment](hoprdb.md#setcurrentcommitment)
- [storeHashIntermediaries](hoprdb.md#storehashintermediaries)
- [storeUnacknowledgedTicket](hoprdb.md#storeunacknowledgedticket)
- [storeUnacknowledgedTickets](hoprdb.md#storeunacknowledgedtickets)
- [touch](hoprdb.md#touch)
- [updateAccount](hoprdb.md#updateaccount)
- [updateAcknowledgement](hoprdb.md#updateacknowledgement)
- [updateChannel](hoprdb.md#updatechannel)
- [updateLatestBlockNumber](hoprdb.md#updatelatestblocknumber)
- [updateLatestConfirmedSnapshot](hoprdb.md#updatelatestconfirmedsnapshot)
- [createMock](hoprdb.md#createmock)

## Constructors

### constructor

\+ **new HoprDB**(`id`: [*Address*](address.md), `initialize`: *boolean*, `version`: *string*, `dbPath?`: *string*): [*HoprDB*](hoprdb.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `id` | [*Address*](address.md) |
| `initialize` | *boolean* |
| `version` | *string* |
| `dbPath?` | *string* |

**Returns:** [*HoprDB*](hoprdb.md)

Defined in: [db.ts:50](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L50)

## Properties

### db

• `Private` **db**: *LevelUp*<AbstractLevelDOWN<any, any\>, AbstractIterator<any, any\>\>

Defined in: [db.ts:50](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L50)

## Methods

### checkAndSetPacketTag

▸ **checkAndSetPacketTag**(`packetTag`: *Uint8Array*): *Promise*<boolean\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `packetTag` | *Uint8Array* |

**Returns:** *Promise*<boolean\>

Defined in: [db.ts:261](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L261)

___

### close

▸ **close**(): *Promise*<void\>

**Returns:** *Promise*<void\>

Defined in: [db.ts:324](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L324)

___

### del

▸ `Private` **del**(`key`: *Uint8Array*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |

**Returns:** *Promise*<void\>

Defined in: [db.ts:137](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L137)

___

### deleteAcknowledgement

▸ **deleteAcknowledgement**(`acknowledgement`: [*AcknowledgedTicket*](acknowledgedticket.md)): *Promise*<void\>

Delete acknowledged ticket in database

#### Parameters

| Name | Type |
| :------ | :------ |
| `acknowledgement` | [*AcknowledgedTicket*](acknowledgedticket.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:227](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L227)

___

### deleteAcknowledgements

▸ **deleteAcknowledgements**(`filter?`: { `signer`: *Uint8Array*  }): *Promise*<void\>

Delete acknowledged tickets

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `filter?` | *object* | optionally filter by signer |
| `filter.signer` | *Uint8Array* | - |

**Returns:** *Promise*<void\>

Defined in: [db.ts:200](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L200)

___

### deleteTicket

▸ **deleteTicket**(`challenge`: [*HalfKeyChallenge*](halfkeychallenge.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `challenge` | [*HalfKeyChallenge*](halfkeychallenge.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:288](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L288)

___

### deleteTickets

▸ **deleteTickets**(`filter?`: { `signer`: *Uint8Array*  }): *Promise*<void\>

Get signed tickets, both unacknowledged and acknowledged

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `filter?` | *object* | optionally filter by signer |
| `filter.signer` | *Uint8Array* | - |

**Returns:** *Promise*<void\>

an array of signed tickets

Defined in: [db.ts:253](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L253)

___

### deleteUnacknowledgedTickets

▸ **deleteUnacknowledgedTickets**(`filter?`: { `signer`: *Uint8Array*  }): *Promise*<void\>

Delete unacknowledged tickets

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `filter?` | *object* | optionally filter by signer |
| `filter.signer` | *Uint8Array* | - |

**Returns:** *Promise*<void\>

Defined in: [db.ts:165](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L165)

___

### get

▸ `Private` **get**(`key`: *Uint8Array*): *Promise*<Uint8Array\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |

**Returns:** *Promise*<Uint8Array\>

Defined in: [db.ts:98](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L98)

___

### getAccount

▸ **getAccount**(`address`: [*Address*](address.md)): *Promise*<[*AccountEntry*](accountentry.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | [*Address*](address.md) |

**Returns:** *Promise*<[*AccountEntry*](accountentry.md)\>

Defined in: [db.ts:382](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L382)

___

### getAccounts

▸ **getAccounts**(`filter?`: (`account`: [*AccountEntry*](accountentry.md)) => *boolean*): *Promise*<[*AccountEntry*](accountentry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `filter?` | (`account`: [*AccountEntry*](accountentry.md)) => *boolean* |

**Returns:** *Promise*<[*AccountEntry*](accountentry.md)[]\>

Defined in: [db.ts:391](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L391)

___

### getAcknowledgedTickets

▸ **getAcknowledgedTickets**(`filter?`: { `signer`: *Uint8Array*  }): *Promise*<[*AcknowledgedTicket*](acknowledgedticket.md)[]\>

Get all acknowledged tickets

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `filter?` | *object* | optionally filter by signer |
| `filter.signer` | *Uint8Array* | - |

**Returns:** *Promise*<[*AcknowledgedTicket*](acknowledgedticket.md)[]\>

an array of all acknowledged tickets

Defined in: [db.ts:185](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L185)

___

### getAll

▸ `Private` **getAll**<T\>(`prefix`: *Uint8Array*, `deserialize`: (`u`: *Uint8Array*) => T, `filter`: (`o`: T) => *boolean*): *Promise*<T[]\>

#### Type parameters

| Name |
| :------ |
| `T` |

#### Parameters

| Name | Type |
| :------ | :------ |
| `prefix` | *Uint8Array* |
| `deserialize` | (`u`: *Uint8Array*) => T |
| `filter` | (`o`: T) => *boolean* |

**Returns:** *Promise*<T[]\>

Defined in: [db.ts:113](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L113)

___

### getChannel

▸ **getChannel**(`channelId`: [*Hash*](hash.md)): *Promise*<[*ChannelEntry*](channelentry.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [*Hash*](hash.md) |

**Returns:** *Promise*<[*ChannelEntry*](channelentry.md)\>

Defined in: [db.ts:368](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L368)

___

### getChannels

▸ **getChannels**(`filter?`: (`channel`: [*ChannelEntry*](channelentry.md)) => *boolean*): *Promise*<[*ChannelEntry*](channelentry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `filter?` | (`channel`: [*ChannelEntry*](channelentry.md)) => *boolean* |

**Returns:** *Promise*<[*ChannelEntry*](channelentry.md)[]\>

Defined in: [db.ts:373](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L373)

___

### getCommitment

▸ **getCommitment**(`channelId`: [*Hash*](hash.md), `iteration`: *number*): *Promise*<Uint8Array\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [*Hash*](hash.md) |
| `iteration` | *number* |

**Returns:** *Promise*<Uint8Array\>

Defined in: [db.ts:338](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L338)

___

### getCurrentCommitment

▸ **getCurrentCommitment**(`channelId`: [*Hash*](hash.md)): *Promise*<[*Hash*](hash.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [*Hash*](hash.md) |

**Returns:** *Promise*<[*Hash*](hash.md)\>

Defined in: [db.ts:342](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L342)

___

### getLatestBlockNumber

▸ **getLatestBlockNumber**(): *Promise*<number\>

**Returns:** *Promise*<number\>

Defined in: [db.ts:350](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L350)

___

### getLatestConfirmedSnapshot

▸ **getLatestConfirmedSnapshot**(): *Promise*<[*Snapshot*](snapshot.md)\>

**Returns:** *Promise*<[*Snapshot*](snapshot.md)\>

Defined in: [db.ts:359](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L359)

___

### getTicketCounter

▸ `Private` **getTicketCounter**(): *Promise*<Uint8Array\>

**Returns:** *Promise*<Uint8Array\>

Defined in: [db.ts:308](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L308)

___

### getTickets

▸ **getTickets**(`filter?`: { `signer`: *Uint8Array*  }): *Promise*<[*Ticket*](ticket.md)[]\>

Get signed tickets, both unacknowledged and acknowledged

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `filter?` | *object* | optionally filter by signer |
| `filter.signer` | *Uint8Array* | - |

**Returns:** *Promise*<[*Ticket*](ticket.md)[]\>

an array of signed tickets

Defined in: [db.ts:237](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L237)

___

### getUnacknowledgedTickets

▸ **getUnacknowledgedTickets**(`filter?`: { `signer`: *Uint8Array*  }): *Promise*<[*UnacknowledgedTicket*](unacknowledgedticket.md)[]\>

Get all unacknowledged tickets

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `filter?` | *object* | optionally filter by signer |
| `filter.signer` | *Uint8Array* | - |

**Returns:** *Promise*<[*UnacknowledgedTicket*](unacknowledgedticket.md)[]\>

an array of all unacknowledged tickets

Defined in: [db.ts:146](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L146)

___

### getUnacknowledgedTicketsByKey

▸ **getUnacknowledgedTicketsByKey**(`challenge`: [*HalfKeyChallenge*](halfkeychallenge.md)): *Promise*<[*UnacknowledgedTicket*](unacknowledgedticket.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `challenge` | [*HalfKeyChallenge*](halfkeychallenge.md) |

**Returns:** *Promise*<[*UnacknowledgedTicket*](unacknowledgedticket.md)\>

Defined in: [db.ts:271](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L271)

___

### has

▸ `Private` **has**(`key`: *Uint8Array*): *Promise*<boolean\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |

**Returns:** *Promise*<boolean\>

Defined in: [db.ts:76](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L76)

___

### keyOf

▸ `Private` **keyOf**(...`segments`: *Uint8Array*[]): *Uint8Array*

#### Parameters

| Name | Type |
| :------ | :------ |
| `...segments` | *Uint8Array*[] |

**Returns:** *Uint8Array*

Defined in: [db.ts:72](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L72)

___

### maybeGet

▸ `Private` **maybeGet**(`key`: *Uint8Array*): *Promise*<Uint8Array\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |

**Returns:** *Promise*<Uint8Array\>

Defined in: [db.ts:102](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L102)

___

### put

▸ `Private` **put**(`key`: *Uint8Array*, `value`: *Uint8Array*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |
| `value` | *Uint8Array* |

**Returns:** *Promise*<void\>

Defined in: [db.ts:90](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L90)

___

### replaceTicketWithAcknowledgement

▸ **replaceTicketWithAcknowledgement**(`keyHalfChallenge`: [*HalfKeyChallenge*](halfkeychallenge.md), `acknowledgment`: [*AcknowledgedTicket*](acknowledgedticket.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `keyHalfChallenge` | [*HalfKeyChallenge*](halfkeychallenge.md) |
| `acknowledgment` | [*AcknowledgedTicket*](acknowledgedticket.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:292](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L292)

___

### setCurrentCommitment

▸ **setCurrentCommitment**(`channelId`: [*Hash*](hash.md), `commitment`: [*Hash*](hash.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [*Hash*](hash.md) |
| `commitment` | [*Hash*](hash.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:346](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L346)

___

### storeHashIntermediaries

▸ **storeHashIntermediaries**(`channelId`: [*Hash*](hash.md), `intermediates`: [*Intermediate*](../interfaces/intermediate.md)[]): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [*Hash*](hash.md) |
| `intermediates` | [*Intermediate*](../interfaces/intermediate.md)[] |

**Returns:** *Promise*<void\>

Defined in: [db.ts:328](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L328)

___

### storeUnacknowledgedTicket

▸ **storeUnacknowledgedTicket**(`challenge`: [*HalfKeyChallenge*](halfkeychallenge.md)): *Promise*<Uint8Array\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `challenge` | [*HalfKeyChallenge*](halfkeychallenge.md) |

**Returns:** *Promise*<Uint8Array\>

Defined in: [db.ts:318](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L318)

___

### storeUnacknowledgedTickets

▸ **storeUnacknowledgedTickets**(`challenge`: [*HalfKeyChallenge*](halfkeychallenge.md), `unacknowledged`: [*UnacknowledgedTicket*](unacknowledgedticket.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `challenge` | [*HalfKeyChallenge*](halfkeychallenge.md) |
| `unacknowledged` | [*UnacknowledgedTicket*](unacknowledgedticket.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:257](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L257)

___

### touch

▸ `Private` **touch**(`key`: *Uint8Array*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |

**Returns:** *Promise*<void\>

Defined in: [db.ts:94](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L94)

___

### updateAccount

▸ **updateAccount**(`account`: [*AccountEntry*](accountentry.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | [*AccountEntry*](accountentry.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:387](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L387)

___

### updateAcknowledgement

▸ **updateAcknowledgement**(`ackTicket`: [*AcknowledgedTicket*](acknowledgedticket.md), `index`: *Uint8Array*): *Promise*<void\>

Update acknowledged ticket in database

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `ackTicket` | [*AcknowledgedTicket*](acknowledgedticket.md) | Uint8Array |
| `index` | *Uint8Array* | Uint8Array |

**Returns:** *Promise*<void\>

Defined in: [db.ts:219](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L219)

___

### updateChannel

▸ **updateChannel**(`channelId`: [*Hash*](hash.md), `channel`: [*ChannelEntry*](channelentry.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [*Hash*](hash.md) |
| `channel` | [*ChannelEntry*](channelentry.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:378](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L378)

___

### updateLatestBlockNumber

▸ **updateLatestBlockNumber**(`blockNumber`: *BN*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `blockNumber` | *BN* |

**Returns:** *Promise*<void\>

Defined in: [db.ts:355](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L355)

___

### updateLatestConfirmedSnapshot

▸ **updateLatestConfirmedSnapshot**(`snapshot`: [*Snapshot*](snapshot.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `snapshot` | [*Snapshot*](snapshot.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:364](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L364)

___

### createMock

▸ `Static` **createMock**(): [*HoprDB*](hoprdb.md)

**Returns:** [*HoprDB*](hoprdb.md)

Defined in: [db.ts:396](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L396)
