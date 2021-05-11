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

\+ **new HoprDB**(`id`: [_Address_](address.md), `initialize`: _boolean_, `version`: _string_, `dbPath?`: _string_): [_HoprDB_](hoprdb.md)

#### Parameters

| Name         | Type                    |
| :----------- | :---------------------- |
| `id`         | [_Address_](address.md) |
| `initialize` | _boolean_               |
| `version`    | _string_                |
| `dbPath?`    | _string_                |

**Returns:** [_HoprDB_](hoprdb.md)

Defined in: [db.ts:49](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L49)

## Properties

### db

• `Private` **db**: _LevelUp_<AbstractLevelDOWN<any, any\>, AbstractIterator<any, any\>\>

Defined in: [db.ts:49](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L49)

## Methods

### checkAndSetPacketTag

▸ **checkAndSetPacketTag**(`packetTag`: _Uint8Array_): _Promise_<boolean\>

#### Parameters

| Name        | Type         |
| :---------- | :----------- |
| `packetTag` | _Uint8Array_ |

**Returns:** _Promise_<boolean\>

Defined in: [db.ts:260](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L260)

---

### close

▸ **close**(): _Promise_<void\>

**Returns:** _Promise_<void\>

Defined in: [db.ts:323](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L323)

---

### del

▸ `Private` **del**(`key`: _Uint8Array_): _Promise_<void\>

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `key` | _Uint8Array_ |

**Returns:** _Promise_<void\>

Defined in: [db.ts:136](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L136)

---

### deleteAcknowledgement

▸ **deleteAcknowledgement**(`acknowledgement`: [_AcknowledgedTicket_](acknowledgedticket.md)): _Promise_<void\>

Delete acknowledged ticket in database

#### Parameters

| Name              | Type                                          |
| :---------------- | :-------------------------------------------- |
| `acknowledgement` | [_AcknowledgedTicket_](acknowledgedticket.md) |

**Returns:** _Promise_<void\>

Defined in: [db.ts:226](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L226)

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

Defined in: [db.ts:199](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L199)

---

### deleteTicket

▸ **deleteTicket**(`key`: [_PublicKey_](publickey.md)): _Promise_<void\>

#### Parameters

| Name  | Type                        |
| :---- | :-------------------------- |
| `key` | [_PublicKey_](publickey.md) |

**Returns:** _Promise_<void\>

Defined in: [db.ts:287](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L287)

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

Defined in: [db.ts:252](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L252)

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

Defined in: [db.ts:164](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L164)

---

### get

▸ `Private` **get**(`key`: _Uint8Array_): _Promise_<Uint8Array\>

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `key` | _Uint8Array_ |

**Returns:** _Promise_<Uint8Array\>

Defined in: [db.ts:97](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L97)

---

### getAccount

▸ **getAccount**(`address`: [_Address_](address.md)): _Promise_<[_AccountEntry_](accountentry.md)\>

#### Parameters

| Name      | Type                    |
| :-------- | :---------------------- |
| `address` | [_Address_](address.md) |

**Returns:** _Promise_<[_AccountEntry_](accountentry.md)\>

Defined in: [db.ts:381](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L381)

---

### getAccounts

▸ **getAccounts**(`filter?`: (`account`: [_AccountEntry_](accountentry.md)) => _boolean_): _Promise_<[_AccountEntry_](accountentry.md)[]\>

#### Parameters

| Name      | Type                                                        |
| :-------- | :---------------------------------------------------------- |
| `filter?` | (`account`: [_AccountEntry_](accountentry.md)) => _boolean_ |

**Returns:** _Promise_<[_AccountEntry_](accountentry.md)[]\>

Defined in: [db.ts:390](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L390)

---

### getAcknowledgedTickets

▸ **getAcknowledgedTickets**(`filter?`: { `signer`: _Uint8Array_ }): _Promise_<[_AcknowledgedTicket_](acknowledgedticket.md)[]\>

Get all acknowledged tickets

#### Parameters

| Name            | Type         | Description                 |
| :-------------- | :----------- | :-------------------------- |
| `filter?`       | _object_     | optionally filter by signer |
| `filter.signer` | _Uint8Array_ | -                           |

**Returns:** _Promise_<[_AcknowledgedTicket_](acknowledgedticket.md)[]\>

an array of all acknowledged tickets

Defined in: [db.ts:184](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L184)

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

Defined in: [db.ts:112](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L112)

---

### getChannel

▸ **getChannel**(`channelId`: [_Hash_](hash.md)): _Promise_<[_ChannelEntry_](channelentry.md)\>

#### Parameters

| Name        | Type              |
| :---------- | :---------------- |
| `channelId` | [_Hash_](hash.md) |

**Returns:** _Promise_<[_ChannelEntry_](channelentry.md)\>

Defined in: [db.ts:367](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L367)

---

### getChannels

▸ **getChannels**(`filter?`: (`channel`: [_ChannelEntry_](channelentry.md)) => _boolean_): _Promise_<[_ChannelEntry_](channelentry.md)[]\>

#### Parameters

| Name      | Type                                                        |
| :-------- | :---------------------------------------------------------- |
| `filter?` | (`channel`: [_ChannelEntry_](channelentry.md)) => _boolean_ |

**Returns:** _Promise_<[_ChannelEntry_](channelentry.md)[]\>

Defined in: [db.ts:372](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L372)

---

### getCommitment

▸ **getCommitment**(`channelId`: [_Hash_](hash.md), `iteration`: _number_): _Promise_<Uint8Array\>

#### Parameters

| Name        | Type              |
| :---------- | :---------------- |
| `channelId` | [_Hash_](hash.md) |
| `iteration` | _number_          |

**Returns:** _Promise_<Uint8Array\>

Defined in: [db.ts:337](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L337)

---

### getCurrentCommitment

▸ **getCurrentCommitment**(`channelId`: [_Hash_](hash.md)): _Promise_<[_Hash_](hash.md)\>

#### Parameters

| Name        | Type              |
| :---------- | :---------------- |
| `channelId` | [_Hash_](hash.md) |

**Returns:** _Promise_<[_Hash_](hash.md)\>

Defined in: [db.ts:341](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L341)

---

### getLatestBlockNumber

▸ **getLatestBlockNumber**(): _Promise_<number\>

**Returns:** _Promise_<number\>

Defined in: [db.ts:349](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L349)

---

### getLatestConfirmedSnapshot

▸ **getLatestConfirmedSnapshot**(): _Promise_<[_Snapshot_](snapshot.md)\>

**Returns:** _Promise_<[_Snapshot_](snapshot.md)\>

Defined in: [db.ts:358](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L358)

---

### getTicketCounter

▸ `Private` **getTicketCounter**(): _Promise_<Uint8Array\>

**Returns:** _Promise_<Uint8Array\>

Defined in: [db.ts:307](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L307)

---

### getTickets

▸ **getTickets**(`filter?`: { `signer`: _Uint8Array_ }): _Promise_<[_Ticket_](ticket.md)[]\>

Get signed tickets, both unacknowledged and acknowledged

#### Parameters

| Name            | Type         | Description                 |
| :-------------- | :----------- | :-------------------------- |
| `filter?`       | _object_     | optionally filter by signer |
| `filter.signer` | _Uint8Array_ | -                           |

**Returns:** _Promise_<[_Ticket_](ticket.md)[]\>

an array of signed tickets

Defined in: [db.ts:236](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L236)

---

### getUnacknowledgedTickets

▸ **getUnacknowledgedTickets**(`filter?`: { `signer`: _Uint8Array_ }): _Promise_<[_UnacknowledgedTicket_](unacknowledgedticket.md)[]\>

Get all unacknowledged tickets

#### Parameters

| Name            | Type         | Description                 |
| :-------------- | :----------- | :-------------------------- |
| `filter?`       | _object_     | optionally filter by signer |
| `filter.signer` | _Uint8Array_ | -                           |

**Returns:** _Promise_<[_UnacknowledgedTicket_](unacknowledgedticket.md)[]\>

an array of all unacknowledged tickets

Defined in: [db.ts:145](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L145)

---

### getUnacknowledgedTicketsByKey

▸ **getUnacknowledgedTicketsByKey**(`key`: [_PublicKey_](publickey.md)): _Promise_<[_UnacknowledgedTicket_](unacknowledgedticket.md)\>

#### Parameters

| Name  | Type                        |
| :---- | :-------------------------- |
| `key` | [_PublicKey_](publickey.md) |

**Returns:** _Promise_<[_UnacknowledgedTicket_](unacknowledgedticket.md)\>

Defined in: [db.ts:270](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L270)

---

### has

▸ `Private` **has**(`key`: _Uint8Array_): _Promise_<boolean\>

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `key` | _Uint8Array_ |

**Returns:** _Promise_<boolean\>

Defined in: [db.ts:75](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L75)

---

### keyOf

▸ `Private` **keyOf**(...`segments`: _Uint8Array_[]): _Uint8Array_

#### Parameters

| Name          | Type           |
| :------------ | :------------- |
| `...segments` | _Uint8Array_[] |

**Returns:** _Uint8Array_

Defined in: [db.ts:71](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L71)

---

### maybeGet

▸ `Private` **maybeGet**(`key`: _Uint8Array_): _Promise_<Uint8Array\>

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `key` | _Uint8Array_ |

**Returns:** _Promise_<Uint8Array\>

Defined in: [db.ts:101](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L101)

---

### put

▸ `Private` **put**(`key`: _Uint8Array_, `value`: _Uint8Array_): _Promise_<void\>

#### Parameters

| Name    | Type         |
| :------ | :----------- |
| `key`   | _Uint8Array_ |
| `value` | _Uint8Array_ |

**Returns:** _Promise_<void\>

Defined in: [db.ts:89](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L89)

---

### replaceTicketWithAcknowledgement

▸ **replaceTicketWithAcknowledgement**(`key`: [_PublicKey_](publickey.md), `acknowledgment`: [_AcknowledgedTicket_](acknowledgedticket.md)): _Promise_<void\>

#### Parameters

| Name             | Type                                          |
| :--------------- | :-------------------------------------------- |
| `key`            | [_PublicKey_](publickey.md)                   |
| `acknowledgment` | [_AcknowledgedTicket_](acknowledgedticket.md) |

**Returns:** _Promise_<void\>

Defined in: [db.ts:291](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L291)

---

### setCurrentCommitment

▸ **setCurrentCommitment**(`channelId`: [_Hash_](hash.md), `commitment`: [_Hash_](hash.md)): _Promise_<void\>

#### Parameters

| Name         | Type              |
| :----------- | :---------------- |
| `channelId`  | [_Hash_](hash.md) |
| `commitment` | [_Hash_](hash.md) |

**Returns:** _Promise_<void\>

Defined in: [db.ts:345](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L345)

---

### storeHashIntermediaries

▸ **storeHashIntermediaries**(`channelId`: [_Hash_](hash.md), `intermediates`: [_Intermediate_](../interfaces/intermediate.md)[]): _Promise_<void\>

#### Parameters

| Name            | Type                                              |
| :-------------- | :------------------------------------------------ |
| `channelId`     | [_Hash_](hash.md)                                 |
| `intermediates` | [_Intermediate_](../interfaces/intermediate.md)[] |

**Returns:** _Promise_<void\>

Defined in: [db.ts:327](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L327)

---

### storeUnacknowledgedTicket

▸ **storeUnacknowledgedTicket**(`challenge`: [_PublicKey_](publickey.md)): _Promise_<Uint8Array\>

#### Parameters

| Name        | Type                        |
| :---------- | :-------------------------- |
| `challenge` | [_PublicKey_](publickey.md) |

**Returns:** _Promise_<Uint8Array\>

Defined in: [db.ts:317](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L317)

---

### storeUnacknowledgedTickets

▸ **storeUnacknowledgedTickets**(`key`: [_PublicKey_](publickey.md), `unacknowledged`: [_UnacknowledgedTicket_](unacknowledgedticket.md)): _Promise_<void\>

#### Parameters

| Name             | Type                                              |
| :--------------- | :------------------------------------------------ |
| `key`            | [_PublicKey_](publickey.md)                       |
| `unacknowledged` | [_UnacknowledgedTicket_](unacknowledgedticket.md) |

**Returns:** _Promise_<void\>

Defined in: [db.ts:256](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L256)

---

### touch

▸ `Private` **touch**(`key`: _Uint8Array_): _Promise_<void\>

#### Parameters

| Name  | Type         |
| :---- | :----------- |
| `key` | _Uint8Array_ |

**Returns:** _Promise_<void\>

Defined in: [db.ts:93](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L93)

---

### updateAccount

▸ **updateAccount**(`account`: [_AccountEntry_](accountentry.md)): _Promise_<void\>

#### Parameters

| Name      | Type                              |
| :-------- | :-------------------------------- |
| `account` | [_AccountEntry_](accountentry.md) |

**Returns:** _Promise_<void\>

Defined in: [db.ts:386](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L386)

---

### updateAcknowledgement

▸ **updateAcknowledgement**(`ackTicket`: [_AcknowledgedTicket_](acknowledgedticket.md), `index`: _Uint8Array_): _Promise_<void\>

Update acknowledged ticket in database

#### Parameters

| Name        | Type                                          | Description |
| :---------- | :-------------------------------------------- | :---------- |
| `ackTicket` | [_AcknowledgedTicket_](acknowledgedticket.md) | Uint8Array  |
| `index`     | _Uint8Array_                                  | Uint8Array  |

**Returns:** _Promise_<void\>

Defined in: [db.ts:218](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L218)

---

### updateChannel

▸ **updateChannel**(`channelId`: [_Hash_](hash.md), `channel`: [_ChannelEntry_](channelentry.md)): _Promise_<void\>

#### Parameters

| Name        | Type                              |
| :---------- | :-------------------------------- |
| `channelId` | [_Hash_](hash.md)                 |
| `channel`   | [_ChannelEntry_](channelentry.md) |

**Returns:** _Promise_<void\>

Defined in: [db.ts:377](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L377)

---

### updateLatestBlockNumber

▸ **updateLatestBlockNumber**(`blockNumber`: _BN_): _Promise_<void\>

#### Parameters

| Name          | Type |
| :------------ | :--- |
| `blockNumber` | _BN_ |

**Returns:** _Promise_<void\>

Defined in: [db.ts:354](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L354)

---

### updateLatestConfirmedSnapshot

▸ **updateLatestConfirmedSnapshot**(`snapshot`: [_Snapshot_](snapshot.md)): _Promise_<void\>

#### Parameters

| Name       | Type                      |
| :--------- | :------------------------ |
| `snapshot` | [_Snapshot_](snapshot.md) |

**Returns:** _Promise_<void\>

Defined in: [db.ts:363](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L363)

---

### createMock

▸ `Static` **createMock**(): [_HoprDB_](hoprdb.md)

**Returns:** [_HoprDB_](hoprdb.md)

Defined in: [db.ts:395](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L395)
