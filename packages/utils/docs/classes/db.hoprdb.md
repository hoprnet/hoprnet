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

\+ **new HoprDB**(`id`: [_Address_](types_primitives.address.md), `initialize`: _boolean_, `version`: _string_, `dbPath?`: _string_): [_HoprDB_](db.hoprdb.md)

#### Parameters

| Name         | Type                                     |
| :----------- | :--------------------------------------- |
| `id`         | [_Address_](types_primitives.address.md) |
| `initialize` | _boolean_                                |
| `version`    | _string_                                 |
| `dbPath?`    | _string_                                 |

**Returns:** [_HoprDB_](db.hoprdb.md)

Defined in: [db.ts:51](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L51)

## Properties

### db

• `Private` **db**: _LevelUp_<AbstractLevelDOWN<any, any\>, AbstractIterator<any, any\>\>

Defined in: [db.ts:51](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L51)

## Methods

### checkAndSetPacketTag

▸ **checkAndSetPacketTag**(`packetTag`: _Uint8Array_): _Promise_<boolean\>

#### Parameters

| Name        | Type         |
| :---------- | :----------- |
| `packetTag` | _Uint8Array_ |

**Returns:** _Promise_<boolean\>

Defined in: [db.ts:262](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L262)

---

### close

▸ **close**(): _Promise_<void\>

**Returns:** _Promise_<void\>

Defined in: [db.ts:324](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L324)

---

### del

▸ `Private` **del**(`key`: _Uint8Array_): _Promise_<void\>

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `key` | _Uint8Array_ |

**Returns:** _Promise_<void\>

Defined in: [db.ts:138](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L138)

---

### deleteAcknowledgement

▸ **deleteAcknowledgement**(`acknowledgement`: [_AcknowledgedTicket_](types_acknowledged.acknowledgedticket.md)): _Promise_<void\>

Delete acknowledged ticket in database

#### Parameters

| Name              | Type                                                             |
| :---------------- | :--------------------------------------------------------------- |
| `acknowledgement` | [_AcknowledgedTicket_](types_acknowledged.acknowledgedticket.md) |

**Returns:** _Promise_<void\>

Defined in: [db.ts:228](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L228)

---

### deleteAcknowledgements

▸ **deleteAcknowledgements**(`filter?`: { `signer`: _Uint8Array_ }): _Promise_<void\>

Delete acknowledged tickets

#### Parameters

| Name            | Type         | Description                 |
| :-------------- | :----------- | :-------------------------- |
| `filter?`       | _object_     | optionally filter by signer |
| `filter.signer` | _Uint8Array_ | -                           |

**Returns:** _Promise_<void\>

Defined in: [db.ts:201](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L201)

---

### deleteTicket

▸ **deleteTicket**(`key`: [_PublicKey_](types_primitives.publickey.md)): _Promise_<void\>

#### Parameters

| Name  | Type                                         |
| :---- | :------------------------------------------- |
| `key` | [_PublicKey_](types_primitives.publickey.md) |

**Returns:** _Promise_<void\>

Defined in: [db.ts:288](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L288)

---

### deleteTickets

▸ **deleteTickets**(`filter?`: { `signer`: _Uint8Array_ }): _Promise_<void\>

Get signed tickets, both unacknowledged and acknowledged

#### Parameters

| Name            | Type         | Description                 |
| :-------------- | :----------- | :-------------------------- |
| `filter?`       | _object_     | optionally filter by signer |
| `filter.signer` | _Uint8Array_ | -                           |

**Returns:** _Promise_<void\>

an array of signed tickets

Defined in: [db.ts:254](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L254)

---

### deleteUnacknowledgedTickets

▸ **deleteUnacknowledgedTickets**(`filter?`: { `signer`: _Uint8Array_ }): _Promise_<void\>

Delete unacknowledged tickets

#### Parameters

| Name            | Type         | Description                 |
| :-------------- | :----------- | :-------------------------- |
| `filter?`       | _object_     | optionally filter by signer |
| `filter.signer` | _Uint8Array_ | -                           |

**Returns:** _Promise_<void\>

Defined in: [db.ts:166](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L166)

---

### get

▸ `Private` **get**(`key`: _Uint8Array_): _Promise_<Uint8Array\>

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `key` | _Uint8Array_ |

**Returns:** _Promise_<Uint8Array\>

Defined in: [db.ts:99](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L99)

---

### getAccount

▸ **getAccount**(`address`: [_Address_](types_primitives.address.md)): _Promise_<[_AccountEntry_](types_accountentry.accountentry.md)\>

#### Parameters

| Name      | Type                                     |
| :-------- | :--------------------------------------- |
| `address` | [_Address_](types_primitives.address.md) |

**Returns:** _Promise_<[_AccountEntry_](types_accountentry.accountentry.md)\>

Defined in: [db.ts:382](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L382)

---

### getAccounts

▸ **getAccounts**(`filter?`: (`account`: [_AccountEntry_](types_accountentry.accountentry.md)) => _boolean_): _Promise_<[_AccountEntry_](types_accountentry.accountentry.md)[]\>

#### Parameters

| Name      | Type                                                                           |
| :-------- | :----------------------------------------------------------------------------- |
| `filter?` | (`account`: [_AccountEntry_](types_accountentry.accountentry.md)) => _boolean_ |

**Returns:** _Promise_<[_AccountEntry_](types_accountentry.accountentry.md)[]\>

Defined in: [db.ts:391](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L391)

---

### getAcknowledgements

▸ **getAcknowledgements**(`filter?`: { `signer`: _Uint8Array_ }): _Promise_<[_AcknowledgedTicket_](types_acknowledged.acknowledgedticket.md)[]\>

Get all acknowledged tickets

#### Parameters

| Name            | Type         | Description                 |
| :-------------- | :----------- | :-------------------------- |
| `filter?`       | _object_     | optionally filter by signer |
| `filter.signer` | _Uint8Array_ | -                           |

**Returns:** _Promise_<[_AcknowledgedTicket_](types_acknowledged.acknowledgedticket.md)[]\>

an array of all acknowledged tickets

Defined in: [db.ts:186](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L186)

---

### getAll

▸ `Private` **getAll**<T\>(`prefix`: _Uint8Array_, `deserialize`: (`u`: _Uint8Array_) => T, `filter`: (`o`: T) => _boolean_): _Promise_<T[]\>

#### Type parameters

| Name |
| :--- |
| `T`  |

#### Parameters

| Name          | Type                     |
| :------------ | :----------------------- |
| `prefix`      | _Uint8Array_             |
| `deserialize` | (`u`: _Uint8Array_) => T |
| `filter`      | (`o`: T) => _boolean_    |

**Returns:** _Promise_<T[]\>

Defined in: [db.ts:114](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L114)

---

### getChannel

▸ **getChannel**(`channelId`: [_Hash_](types_primitives.hash.md)): _Promise_<[_ChannelEntry_](types_channelentry.channelentry.md)\>

#### Parameters

| Name        | Type                               |
| :---------- | :--------------------------------- |
| `channelId` | [_Hash_](types_primitives.hash.md) |

**Returns:** _Promise_<[_ChannelEntry_](types_channelentry.channelentry.md)\>

Defined in: [db.ts:368](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L368)

---

### getChannels

▸ **getChannels**(`filter?`: (`channel`: [_ChannelEntry_](types_channelentry.channelentry.md)) => _boolean_): _Promise_<[_ChannelEntry_](types_channelentry.channelentry.md)[]\>

#### Parameters

| Name      | Type                                                                           |
| :-------- | :----------------------------------------------------------------------------- |
| `filter?` | (`channel`: [_ChannelEntry_](types_channelentry.channelentry.md)) => _boolean_ |

**Returns:** _Promise_<[_ChannelEntry_](types_channelentry.channelentry.md)[]\>

Defined in: [db.ts:373](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L373)

---

### getCommitment

▸ **getCommitment**(`channelId`: [_Hash_](types_primitives.hash.md), `iteration`: _number_): _Promise_<Uint8Array\>

#### Parameters

| Name        | Type                               |
| :---------- | :--------------------------------- |
| `channelId` | [_Hash_](types_primitives.hash.md) |
| `iteration` | _number_                           |

**Returns:** _Promise_<Uint8Array\>

Defined in: [db.ts:338](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L338)

---

### getCurrentCommitment

▸ **getCurrentCommitment**(`channelId`: [_Hash_](types_primitives.hash.md)): _Promise_<[_Hash_](types_primitives.hash.md)\>

#### Parameters

| Name        | Type                               |
| :---------- | :--------------------------------- |
| `channelId` | [_Hash_](types_primitives.hash.md) |

**Returns:** _Promise_<[_Hash_](types_primitives.hash.md)\>

Defined in: [db.ts:342](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L342)

---

### getLatestBlockNumber

▸ **getLatestBlockNumber**(): _Promise_<number\>

**Returns:** _Promise_<number\>

Defined in: [db.ts:350](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L350)

---

### getLatestConfirmedSnapshot

▸ **getLatestConfirmedSnapshot**(): _Promise_<[_Snapshot_](types_snapshot.snapshot.md)\>

**Returns:** _Promise_<[_Snapshot_](types_snapshot.snapshot.md)\>

Defined in: [db.ts:359](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L359)

---

### getTicketCounter

▸ `Private` **getTicketCounter**(): _Promise_<Uint8Array\>

**Returns:** _Promise_<Uint8Array\>

Defined in: [db.ts:308](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L308)

---

### getTickets

▸ **getTickets**(`filter?`: { `signer`: _Uint8Array_ }): _Promise_<[_Ticket_](types_ticket.ticket.md)[]\>

Get signed tickets, both unacknowledged and acknowledged

#### Parameters

| Name            | Type         | Description                 |
| :-------------- | :----------- | :-------------------------- |
| `filter?`       | _object_     | optionally filter by signer |
| `filter.signer` | _Uint8Array_ | -                           |

**Returns:** _Promise_<[_Ticket_](types_ticket.ticket.md)[]\>

an array of signed tickets

Defined in: [db.ts:238](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L238)

---

### getUnacknowledgedTickets

▸ **getUnacknowledgedTickets**(`filter?`: { `signer`: _Uint8Array_ }): _Promise_<[_UnacknowledgedTicket_](types_unacknowledgedticket.unacknowledgedticket.md)[]\>

Get all unacknowledged tickets

#### Parameters

| Name            | Type         | Description                 |
| :-------------- | :----------- | :-------------------------- |
| `filter?`       | _object_     | optionally filter by signer |
| `filter.signer` | _Uint8Array_ | -                           |

**Returns:** _Promise_<[_UnacknowledgedTicket_](types_unacknowledgedticket.unacknowledgedticket.md)[]\>

an array of all unacknowledged tickets

Defined in: [db.ts:147](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L147)

---

### getUnacknowledgedTicketsByKey

▸ **getUnacknowledgedTicketsByKey**(`key`: [_PublicKey_](types_primitives.publickey.md)): _Promise_<[_UnacknowledgedTicket_](types_unacknowledgedticket.unacknowledgedticket.md)\>

#### Parameters

| Name  | Type                                         |
| :---- | :------------------------------------------- |
| `key` | [_PublicKey_](types_primitives.publickey.md) |

**Returns:** _Promise_<[_UnacknowledgedTicket_](types_unacknowledgedticket.unacknowledgedticket.md)\>

Defined in: [db.ts:272](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L272)

---

### has

▸ `Private` **has**(`key`: _Uint8Array_): _Promise_<boolean\>

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `key` | _Uint8Array_ |

**Returns:** _Promise_<boolean\>

Defined in: [db.ts:77](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L77)

---

### keyOf

▸ `Private` **keyOf**(...`segments`: _Uint8Array_[]): _Uint8Array_

#### Parameters

| Name          | Type           |
| :------------ | :------------- |
| `...segments` | _Uint8Array_[] |

**Returns:** _Uint8Array_

Defined in: [db.ts:73](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L73)

---

### maybeGet

▸ `Private` **maybeGet**(`key`: _Uint8Array_): _Promise_<Uint8Array\>

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `key` | _Uint8Array_ |

**Returns:** _Promise_<Uint8Array\>

Defined in: [db.ts:103](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L103)

---

### put

▸ `Private` **put**(`key`: _Uint8Array_, `value`: _Uint8Array_): _Promise_<void\>

#### Parameters

| Name    | Type         |
| :------ | :----------- |
| `key`   | _Uint8Array_ |
| `value` | _Uint8Array_ |

**Returns:** _Promise_<void\>

Defined in: [db.ts:91](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L91)

---

### replaceTicketWithAcknowledgement

▸ **replaceTicketWithAcknowledgement**(`key`: [_PublicKey_](types_primitives.publickey.md), `acknowledgment`: [_AcknowledgedTicket_](types_acknowledged.acknowledgedticket.md)): _Promise_<void\>

#### Parameters

| Name             | Type                                                             |
| :--------------- | :--------------------------------------------------------------- |
| `key`            | [_PublicKey_](types_primitives.publickey.md)                     |
| `acknowledgment` | [_AcknowledgedTicket_](types_acknowledged.acknowledgedticket.md) |

**Returns:** _Promise_<void\>

Defined in: [db.ts:292](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L292)

---

### setCurrentCommitment

▸ **setCurrentCommitment**(`channelId`: [_Hash_](types_primitives.hash.md), `commitment`: [_Hash_](types_primitives.hash.md)): _Promise_<void\>

#### Parameters

| Name         | Type                               |
| :----------- | :--------------------------------- |
| `channelId`  | [_Hash_](types_primitives.hash.md) |
| `commitment` | [_Hash_](types_primitives.hash.md) |

**Returns:** _Promise_<void\>

Defined in: [db.ts:346](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L346)

---

### storeHashIntermediaries

▸ **storeHashIntermediaries**(`channelId`: [_Hash_](types_primitives.hash.md), `intermediates`: [_Intermediate_](../interfaces/crypto_hashiterator.intermediate.md)[]): _Promise_<void\>

#### Parameters

| Name            | Type                                                                  |
| :-------------- | :-------------------------------------------------------------------- |
| `channelId`     | [_Hash_](types_primitives.hash.md)                                    |
| `intermediates` | [_Intermediate_](../interfaces/crypto_hashiterator.intermediate.md)[] |

**Returns:** _Promise_<void\>

Defined in: [db.ts:328](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L328)

---

### storeUnacknowledgedTicket

▸ **storeUnacknowledgedTicket**(`challenge`: [_PublicKey_](types_primitives.publickey.md)): _Promise_<Uint8Array\>

#### Parameters

| Name        | Type                                         |
| :---------- | :------------------------------------------- |
| `challenge` | [_PublicKey_](types_primitives.publickey.md) |

**Returns:** _Promise_<Uint8Array\>

Defined in: [db.ts:318](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L318)

---

### storeUnacknowledgedTickets

▸ **storeUnacknowledgedTickets**(`key`: _Uint8Array_, `unacknowledged`: [_UnacknowledgedTicket_](types_unacknowledgedticket.unacknowledgedticket.md)): _Promise_<void\>

#### Parameters

| Name             | Type                                                                         |
| :--------------- | :--------------------------------------------------------------------------- |
| `key`            | _Uint8Array_                                                                 |
| `unacknowledged` | [_UnacknowledgedTicket_](types_unacknowledgedticket.unacknowledgedticket.md) |

**Returns:** _Promise_<void\>

Defined in: [db.ts:258](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L258)

---

### touch

▸ `Private` **touch**(`key`: _Uint8Array_): _Promise_<void\>

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `key` | _Uint8Array_ |

**Returns:** _Promise_<void\>

Defined in: [db.ts:95](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L95)

---

### updateAccount

▸ **updateAccount**(`account`: [_AccountEntry_](types_accountentry.accountentry.md)): _Promise_<void\>

#### Parameters

| Name      | Type                                                 |
| :-------- | :--------------------------------------------------- |
| `account` | [_AccountEntry_](types_accountentry.accountentry.md) |

**Returns:** _Promise_<void\>

Defined in: [db.ts:387](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L387)

---

### updateAcknowledgement

▸ **updateAcknowledgement**(`ackTicket`: [_AcknowledgedTicket_](types_acknowledged.acknowledgedticket.md), `index`: _Uint8Array_): _Promise_<void\>

Update acknowledged ticket in database

#### Parameters

| Name        | Type                                                             | Description |
| :---------- | :--------------------------------------------------------------- | :---------- |
| `ackTicket` | [_AcknowledgedTicket_](types_acknowledged.acknowledgedticket.md) | Uint8Array  |
| `index`     | _Uint8Array_                                                     | Uint8Array  |

**Returns:** _Promise_<void\>

Defined in: [db.ts:220](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L220)

---

### updateChannel

▸ **updateChannel**(`channelId`: [_Hash_](types_primitives.hash.md), `channel`: [_ChannelEntry_](types_channelentry.channelentry.md)): _Promise_<void\>

#### Parameters

| Name        | Type                                                 |
| :---------- | :--------------------------------------------------- |
| `channelId` | [_Hash_](types_primitives.hash.md)                   |
| `channel`   | [_ChannelEntry_](types_channelentry.channelentry.md) |

**Returns:** _Promise_<void\>

Defined in: [db.ts:378](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L378)

---

### updateLatestBlockNumber

▸ **updateLatestBlockNumber**(`blockNumber`: _BN_): _Promise_<void\>

#### Parameters

| Name          | Type |
| :------------ | :--- |
| `blockNumber` | _BN_ |

**Returns:** _Promise_<void\>

Defined in: [db.ts:355](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L355)

---

### updateLatestConfirmedSnapshot

▸ **updateLatestConfirmedSnapshot**(`snapshot`: [_Snapshot_](types_snapshot.snapshot.md)): _Promise_<void\>

#### Parameters

| Name       | Type                                     |
| :--------- | :--------------------------------------- |
| `snapshot` | [_Snapshot_](types_snapshot.snapshot.md) |

**Returns:** _Promise_<void\>

Defined in: [db.ts:364](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L364)

---

### createMock

▸ `Static` **createMock**(): [_HoprDB_](db.hoprdb.md)

**Returns:** [_HoprDB_](db.hoprdb.md)

Defined in: [db.ts:396](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/db.ts#L396)
