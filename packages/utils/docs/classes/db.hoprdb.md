[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / [db](../modules/db.md) / HoprDB

# Class: HoprDB

[db](../modules/db.md).HoprDB

## Table of contents

### Constructors

- [constructor](db.hoprdb.md#constructor)

### Properties

- [db](db.hoprdb.md#db)

### Methods

- [checkAndSetPacketTag](db.hoprdb.md#checkandsetpackettag)
- [close](db.hoprdb.md#close)
- [del](db.hoprdb.md#del)
- [deleteAcknowledgement](db.hoprdb.md#deleteacknowledgement)
- [deleteAcknowledgements](db.hoprdb.md#deleteacknowledgements)
- [deleteTicket](db.hoprdb.md#deleteticket)
- [deleteTickets](db.hoprdb.md#deletetickets)
- [deleteUnacknowledgedTickets](db.hoprdb.md#deleteunacknowledgedtickets)
- [get](db.hoprdb.md#get)
- [getAccount](db.hoprdb.md#getaccount)
- [getAccounts](db.hoprdb.md#getaccounts)
- [getAcknowledgements](db.hoprdb.md#getacknowledgements)
- [getAll](db.hoprdb.md#getall)
- [getChannel](db.hoprdb.md#getchannel)
- [getChannels](db.hoprdb.md#getchannels)
- [getCommitment](db.hoprdb.md#getcommitment)
- [getCurrentCommitment](db.hoprdb.md#getcurrentcommitment)
- [getLatestBlockNumber](db.hoprdb.md#getlatestblocknumber)
- [getLatestConfirmedSnapshot](db.hoprdb.md#getlatestconfirmedsnapshot)
- [getTicketCounter](db.hoprdb.md#getticketcounter)
- [getTickets](db.hoprdb.md#gettickets)
- [getUnacknowledgedTickets](db.hoprdb.md#getunacknowledgedtickets)
- [getUnacknowledgedTicketsByKey](db.hoprdb.md#getunacknowledgedticketsbykey)
- [has](db.hoprdb.md#has)
- [keyOf](db.hoprdb.md#keyof)
- [maybeGet](db.hoprdb.md#maybeget)
- [put](db.hoprdb.md#put)
- [replaceTicketWithAcknowledgement](db.hoprdb.md#replaceticketwithacknowledgement)
- [setCurrentCommitment](db.hoprdb.md#setcurrentcommitment)
- [storeHashIntermediaries](db.hoprdb.md#storehashintermediaries)
- [storeUnacknowledgedTicket](db.hoprdb.md#storeunacknowledgedticket)
- [storeUnacknowledgedTickets](db.hoprdb.md#storeunacknowledgedtickets)
- [touch](db.hoprdb.md#touch)
- [updateAccount](db.hoprdb.md#updateaccount)
- [updateAcknowledgement](db.hoprdb.md#updateacknowledgement)
- [updateChannel](db.hoprdb.md#updatechannel)
- [updateLatestBlockNumber](db.hoprdb.md#updatelatestblocknumber)
- [updateLatestConfirmedSnapshot](db.hoprdb.md#updatelatestconfirmedsnapshot)
- [createMock](db.hoprdb.md#createmock)

## Constructors

### constructor

\+ **new HoprDB**(`id`: [*Address*](types_primitives.address.md), `initialize`: *boolean*, `version`: *string*, `dbPath?`: *string*): [*HoprDB*](db.hoprdb.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `id` | [*Address*](types_primitives.address.md) |
| `initialize` | *boolean* |
| `version` | *string* |
| `dbPath?` | *string* |

**Returns:** [*HoprDB*](db.hoprdb.md)

Defined in: [db.ts:51](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L51)

## Properties

### db

• `Private` **db**: *LevelUp*<AbstractLevelDOWN<any, any\>, AbstractIterator<any, any\>\>

Defined in: [db.ts:51](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L51)

## Methods

### checkAndSetPacketTag

▸ **checkAndSetPacketTag**(`packetTag`: *Uint8Array*): *Promise*<boolean\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `packetTag` | *Uint8Array* |

**Returns:** *Promise*<boolean\>

Defined in: [db.ts:262](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L262)

___

### close

▸ **close**(): *Promise*<void\>

**Returns:** *Promise*<void\>

Defined in: [db.ts:324](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L324)

___

### del

▸ `Private` **del**(`key`: *Uint8Array*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |

**Returns:** *Promise*<void\>

Defined in: [db.ts:138](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L138)

___

### deleteAcknowledgement

▸ **deleteAcknowledgement**(`acknowledgement`: [*AcknowledgedTicket*](types_acknowledged.acknowledgedticket.md)): *Promise*<void\>

Delete acknowledged ticket in database

#### Parameters

| Name | Type |
| :------ | :------ |
| `acknowledgement` | [*AcknowledgedTicket*](types_acknowledged.acknowledgedticket.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:228](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L228)

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

Defined in: [db.ts:201](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L201)

___

### deleteTicket

▸ **deleteTicket**(`key`: [*PublicKey*](types_primitives.publickey.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | [*PublicKey*](types_primitives.publickey.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:288](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L288)

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

Defined in: [db.ts:254](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L254)

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

Defined in: [db.ts:166](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L166)

___

### get

▸ `Private` **get**(`key`: *Uint8Array*): *Promise*<Uint8Array\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |

**Returns:** *Promise*<Uint8Array\>

Defined in: [db.ts:99](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L99)

___

### getAccount

▸ **getAccount**(`address`: [*Address*](types_primitives.address.md)): *Promise*<[*AccountEntry*](types_accountentry.accountentry.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | [*Address*](types_primitives.address.md) |

**Returns:** *Promise*<[*AccountEntry*](types_accountentry.accountentry.md)\>

Defined in: [db.ts:382](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L382)

___

### getAccounts

▸ **getAccounts**(`filter?`: (`account`: [*AccountEntry*](types_accountentry.accountentry.md)) => *boolean*): *Promise*<[*AccountEntry*](types_accountentry.accountentry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `filter?` | (`account`: [*AccountEntry*](types_accountentry.accountentry.md)) => *boolean* |

**Returns:** *Promise*<[*AccountEntry*](types_accountentry.accountentry.md)[]\>

Defined in: [db.ts:391](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L391)

___

### getAcknowledgements

▸ **getAcknowledgements**(`filter?`: { `signer`: *Uint8Array*  }): *Promise*<[*AcknowledgedTicket*](types_acknowledged.acknowledgedticket.md)[]\>

Get all acknowledged tickets

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `filter?` | *object* | optionally filter by signer |
| `filter.signer` | *Uint8Array* | - |

**Returns:** *Promise*<[*AcknowledgedTicket*](types_acknowledged.acknowledgedticket.md)[]\>

an array of all acknowledged tickets

Defined in: [db.ts:186](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L186)

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

Defined in: [db.ts:114](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L114)

___

### getChannel

▸ **getChannel**(`channelId`: [*Hash*](types_primitives.hash.md)): *Promise*<[*ChannelEntry*](types_channelentry.channelentry.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [*Hash*](types_primitives.hash.md) |

**Returns:** *Promise*<[*ChannelEntry*](types_channelentry.channelentry.md)\>

Defined in: [db.ts:368](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L368)

___

### getChannels

▸ **getChannels**(`filter?`: (`channel`: [*ChannelEntry*](types_channelentry.channelentry.md)) => *boolean*): *Promise*<[*ChannelEntry*](types_channelentry.channelentry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `filter?` | (`channel`: [*ChannelEntry*](types_channelentry.channelentry.md)) => *boolean* |

**Returns:** *Promise*<[*ChannelEntry*](types_channelentry.channelentry.md)[]\>

Defined in: [db.ts:373](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L373)

___

### getCommitment

▸ **getCommitment**(`channelId`: [*Hash*](types_primitives.hash.md), `iteration`: *number*): *Promise*<Uint8Array\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [*Hash*](types_primitives.hash.md) |
| `iteration` | *number* |

**Returns:** *Promise*<Uint8Array\>

Defined in: [db.ts:338](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L338)

___

### getCurrentCommitment

▸ **getCurrentCommitment**(`channelId`: [*Hash*](types_primitives.hash.md)): *Promise*<[*Hash*](types_primitives.hash.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [*Hash*](types_primitives.hash.md) |

**Returns:** *Promise*<[*Hash*](types_primitives.hash.md)\>

Defined in: [db.ts:342](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L342)

___

### getLatestBlockNumber

▸ **getLatestBlockNumber**(): *Promise*<number\>

**Returns:** *Promise*<number\>

Defined in: [db.ts:350](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L350)

___

### getLatestConfirmedSnapshot

▸ **getLatestConfirmedSnapshot**(): *Promise*<[*Snapshot*](types_snapshot.snapshot.md)\>

**Returns:** *Promise*<[*Snapshot*](types_snapshot.snapshot.md)\>

Defined in: [db.ts:359](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L359)

___

### getTicketCounter

▸ `Private` **getTicketCounter**(): *Promise*<Uint8Array\>

**Returns:** *Promise*<Uint8Array\>

Defined in: [db.ts:308](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L308)

___

### getTickets

▸ **getTickets**(`filter?`: { `signer`: *Uint8Array*  }): *Promise*<[*Ticket*](types_ticket.ticket.md)[]\>

Get signed tickets, both unacknowledged and acknowledged

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `filter?` | *object* | optionally filter by signer |
| `filter.signer` | *Uint8Array* | - |

**Returns:** *Promise*<[*Ticket*](types_ticket.ticket.md)[]\>

an array of signed tickets

Defined in: [db.ts:238](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L238)

___

### getUnacknowledgedTickets

▸ **getUnacknowledgedTickets**(`filter?`: { `signer`: *Uint8Array*  }): *Promise*<[*UnacknowledgedTicket*](types_unacknowledgedticket.unacknowledgedticket.md)[]\>

Get all unacknowledged tickets

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `filter?` | *object* | optionally filter by signer |
| `filter.signer` | *Uint8Array* | - |

**Returns:** *Promise*<[*UnacknowledgedTicket*](types_unacknowledgedticket.unacknowledgedticket.md)[]\>

an array of all unacknowledged tickets

Defined in: [db.ts:147](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L147)

___

### getUnacknowledgedTicketsByKey

▸ **getUnacknowledgedTicketsByKey**(`key`: [*PublicKey*](types_primitives.publickey.md)): *Promise*<[*UnacknowledgedTicket*](types_unacknowledgedticket.unacknowledgedticket.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | [*PublicKey*](types_primitives.publickey.md) |

**Returns:** *Promise*<[*UnacknowledgedTicket*](types_unacknowledgedticket.unacknowledgedticket.md)\>

Defined in: [db.ts:272](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L272)

___

### has

▸ `Private` **has**(`key`: *Uint8Array*): *Promise*<boolean\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |

**Returns:** *Promise*<boolean\>

Defined in: [db.ts:77](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L77)

___

### keyOf

▸ `Private` **keyOf**(...`segments`: *Uint8Array*[]): *Uint8Array*

#### Parameters

| Name | Type |
| :------ | :------ |
| `...segments` | *Uint8Array*[] |

**Returns:** *Uint8Array*

Defined in: [db.ts:73](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L73)

___

### maybeGet

▸ `Private` **maybeGet**(`key`: *Uint8Array*): *Promise*<Uint8Array\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |

**Returns:** *Promise*<Uint8Array\>

Defined in: [db.ts:103](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L103)

___

### put

▸ `Private` **put**(`key`: *Uint8Array*, `value`: *Uint8Array*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |
| `value` | *Uint8Array* |

**Returns:** *Promise*<void\>

Defined in: [db.ts:91](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L91)

___

### replaceTicketWithAcknowledgement

▸ **replaceTicketWithAcknowledgement**(`key`: [*PublicKey*](types_primitives.publickey.md), `acknowledgment`: [*AcknowledgedTicket*](types_acknowledged.acknowledgedticket.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | [*PublicKey*](types_primitives.publickey.md) |
| `acknowledgment` | [*AcknowledgedTicket*](types_acknowledged.acknowledgedticket.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:292](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L292)

___

### setCurrentCommitment

▸ **setCurrentCommitment**(`channelId`: [*Hash*](types_primitives.hash.md), `commitment`: [*Hash*](types_primitives.hash.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [*Hash*](types_primitives.hash.md) |
| `commitment` | [*Hash*](types_primitives.hash.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:346](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L346)

___

### storeHashIntermediaries

▸ **storeHashIntermediaries**(`channelId`: [*Hash*](types_primitives.hash.md), `intermediates`: [*Intermediate*](../interfaces/crypto_hashiterator.intermediate.md)[]): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [*Hash*](types_primitives.hash.md) |
| `intermediates` | [*Intermediate*](../interfaces/crypto_hashiterator.intermediate.md)[] |

**Returns:** *Promise*<void\>

Defined in: [db.ts:328](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L328)

___

### storeUnacknowledgedTicket

▸ **storeUnacknowledgedTicket**(`challenge`: [*PublicKey*](types_primitives.publickey.md)): *Promise*<Uint8Array\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `challenge` | [*PublicKey*](types_primitives.publickey.md) |

**Returns:** *Promise*<Uint8Array\>

Defined in: [db.ts:318](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L318)

___

### storeUnacknowledgedTickets

▸ **storeUnacknowledgedTickets**(`key`: *Uint8Array*, `unacknowledged`: [*UnacknowledgedTicket*](types_unacknowledgedticket.unacknowledgedticket.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |
| `unacknowledged` | [*UnacknowledgedTicket*](types_unacknowledgedticket.unacknowledgedticket.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:258](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L258)

___

### touch

▸ `Private` **touch**(`key`: *Uint8Array*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |

**Returns:** *Promise*<void\>

Defined in: [db.ts:95](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L95)

___

### updateAccount

▸ **updateAccount**(`account`: [*AccountEntry*](types_accountentry.accountentry.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | [*AccountEntry*](types_accountentry.accountentry.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:387](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L387)

___

### updateAcknowledgement

▸ **updateAcknowledgement**(`ackTicket`: [*AcknowledgedTicket*](types_acknowledged.acknowledgedticket.md), `index`: *Uint8Array*): *Promise*<void\>

Update acknowledged ticket in database

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `ackTicket` | [*AcknowledgedTicket*](types_acknowledged.acknowledgedticket.md) | Uint8Array |
| `index` | *Uint8Array* | Uint8Array |

**Returns:** *Promise*<void\>

Defined in: [db.ts:220](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L220)

___

### updateChannel

▸ **updateChannel**(`channelId`: [*Hash*](types_primitives.hash.md), `channel`: [*ChannelEntry*](types_channelentry.channelentry.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [*Hash*](types_primitives.hash.md) |
| `channel` | [*ChannelEntry*](types_channelentry.channelentry.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:378](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L378)

___

### updateLatestBlockNumber

▸ **updateLatestBlockNumber**(`blockNumber`: *BN*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `blockNumber` | *BN* |

**Returns:** *Promise*<void\>

Defined in: [db.ts:355](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L355)

___

### updateLatestConfirmedSnapshot

▸ **updateLatestConfirmedSnapshot**(`snapshot`: [*Snapshot*](types_snapshot.snapshot.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `snapshot` | [*Snapshot*](types_snapshot.snapshot.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:364](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L364)

___

### createMock

▸ `Static` **createMock**(): [*HoprDB*](db.hoprdb.md)

**Returns:** [*HoprDB*](db.hoprdb.md)

Defined in: [db.ts:396](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L396)
