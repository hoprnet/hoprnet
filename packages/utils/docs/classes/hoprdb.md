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

Defined in: [db.ts:49](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L49)

## Properties

### db

• `Private` **db**: *LevelUp*<AbstractLevelDOWN<any, any\>, AbstractIterator<any, any\>\>

Defined in: [db.ts:49](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L49)

## Methods

### checkAndSetPacketTag

▸ **checkAndSetPacketTag**(`packetTag`: *Uint8Array*): *Promise*<boolean\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `packetTag` | *Uint8Array* |

**Returns:** *Promise*<boolean\>

Defined in: [db.ts:260](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L260)

___

### close

▸ **close**(): *Promise*<void\>

**Returns:** *Promise*<void\>

Defined in: [db.ts:323](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L323)

___

### del

▸ `Private` **del**(`key`: *Uint8Array*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |

**Returns:** *Promise*<void\>

Defined in: [db.ts:136](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L136)

___

### deleteAcknowledgement

▸ **deleteAcknowledgement**(`acknowledgement`: [*AcknowledgedTicket*](acknowledgedticket.md)): *Promise*<void\>

Delete acknowledged ticket in database

#### Parameters

| Name | Type |
| :------ | :------ |
| `acknowledgement` | [*AcknowledgedTicket*](acknowledgedticket.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:226](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L226)

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

Defined in: [db.ts:199](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L199)

___

### deleteTicket

▸ **deleteTicket**(`key`: [*PublicKey*](publickey.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | [*PublicKey*](publickey.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:287](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L287)

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

Defined in: [db.ts:252](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L252)

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

Defined in: [db.ts:164](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L164)

___

### get

▸ `Private` **get**(`key`: *Uint8Array*): *Promise*<Uint8Array\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |

**Returns:** *Promise*<Uint8Array\>

Defined in: [db.ts:97](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L97)

___

### getAccount

▸ **getAccount**(`address`: [*Address*](address.md)): *Promise*<[*AccountEntry*](accountentry.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | [*Address*](address.md) |

**Returns:** *Promise*<[*AccountEntry*](accountentry.md)\>

Defined in: [db.ts:381](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L381)

___

### getAccounts

▸ **getAccounts**(`filter?`: (`account`: [*AccountEntry*](accountentry.md)) => *boolean*): *Promise*<[*AccountEntry*](accountentry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `filter?` | (`account`: [*AccountEntry*](accountentry.md)) => *boolean* |

**Returns:** *Promise*<[*AccountEntry*](accountentry.md)[]\>

Defined in: [db.ts:390](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L390)

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

Defined in: [db.ts:184](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L184)

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

Defined in: [db.ts:112](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L112)

___

### getChannel

▸ **getChannel**(`channelId`: [*Hash*](hash.md)): *Promise*<[*ChannelEntry*](channelentry.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [*Hash*](hash.md) |

**Returns:** *Promise*<[*ChannelEntry*](channelentry.md)\>

Defined in: [db.ts:367](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L367)

___

### getChannels

▸ **getChannels**(`filter?`: (`channel`: [*ChannelEntry*](channelentry.md)) => *boolean*): *Promise*<[*ChannelEntry*](channelentry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `filter?` | (`channel`: [*ChannelEntry*](channelentry.md)) => *boolean* |

**Returns:** *Promise*<[*ChannelEntry*](channelentry.md)[]\>

Defined in: [db.ts:372](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L372)

___

### getCommitment

▸ **getCommitment**(`channelId`: [*Hash*](hash.md), `iteration`: *number*): *Promise*<Uint8Array\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [*Hash*](hash.md) |
| `iteration` | *number* |

**Returns:** *Promise*<Uint8Array\>

Defined in: [db.ts:337](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L337)

___

### getCurrentCommitment

▸ **getCurrentCommitment**(`channelId`: [*Hash*](hash.md)): *Promise*<[*Hash*](hash.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [*Hash*](hash.md) |

**Returns:** *Promise*<[*Hash*](hash.md)\>

Defined in: [db.ts:341](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L341)

___

### getLatestBlockNumber

▸ **getLatestBlockNumber**(): *Promise*<number\>

**Returns:** *Promise*<number\>

Defined in: [db.ts:349](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L349)

___

### getLatestConfirmedSnapshot

▸ **getLatestConfirmedSnapshot**(): *Promise*<[*Snapshot*](snapshot.md)\>

**Returns:** *Promise*<[*Snapshot*](snapshot.md)\>

Defined in: [db.ts:358](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L358)

___

### getTicketCounter

▸ `Private` **getTicketCounter**(): *Promise*<Uint8Array\>

**Returns:** *Promise*<Uint8Array\>

Defined in: [db.ts:307](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L307)

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

Defined in: [db.ts:236](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L236)

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

Defined in: [db.ts:145](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L145)

___

### getUnacknowledgedTicketsByKey

▸ **getUnacknowledgedTicketsByKey**(`key`: [*PublicKey*](publickey.md)): *Promise*<[*UnacknowledgedTicket*](unacknowledgedticket.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | [*PublicKey*](publickey.md) |

**Returns:** *Promise*<[*UnacknowledgedTicket*](unacknowledgedticket.md)\>

Defined in: [db.ts:270](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L270)

___

### has

▸ `Private` **has**(`key`: *Uint8Array*): *Promise*<boolean\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |

**Returns:** *Promise*<boolean\>

Defined in: [db.ts:75](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L75)

___

### keyOf

▸ `Private` **keyOf**(...`segments`: *Uint8Array*[]): *Uint8Array*

#### Parameters

| Name | Type |
| :------ | :------ |
| `...segments` | *Uint8Array*[] |

**Returns:** *Uint8Array*

Defined in: [db.ts:71](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L71)

___

### maybeGet

▸ `Private` **maybeGet**(`key`: *Uint8Array*): *Promise*<Uint8Array\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |

**Returns:** *Promise*<Uint8Array\>

Defined in: [db.ts:101](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L101)

___

### put

▸ `Private` **put**(`key`: *Uint8Array*, `value`: *Uint8Array*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |
| `value` | *Uint8Array* |

**Returns:** *Promise*<void\>

Defined in: [db.ts:89](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L89)

___

### replaceTicketWithAcknowledgement

▸ **replaceTicketWithAcknowledgement**(`key`: [*PublicKey*](publickey.md), `acknowledgment`: [*AcknowledgedTicket*](acknowledgedticket.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | [*PublicKey*](publickey.md) |
| `acknowledgment` | [*AcknowledgedTicket*](acknowledgedticket.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:291](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L291)

___

### setCurrentCommitment

▸ **setCurrentCommitment**(`channelId`: [*Hash*](hash.md), `commitment`: [*Hash*](hash.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [*Hash*](hash.md) |
| `commitment` | [*Hash*](hash.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:345](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L345)

___

### storeHashIntermediaries

▸ **storeHashIntermediaries**(`channelId`: [*Hash*](hash.md), `intermediates`: [*Intermediate*](../interfaces/intermediate.md)[]): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [*Hash*](hash.md) |
| `intermediates` | [*Intermediate*](../interfaces/intermediate.md)[] |

**Returns:** *Promise*<void\>

Defined in: [db.ts:327](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L327)

___

### storeUnacknowledgedTicket

▸ **storeUnacknowledgedTicket**(`challenge`: [*PublicKey*](publickey.md)): *Promise*<Uint8Array\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `challenge` | [*PublicKey*](publickey.md) |

**Returns:** *Promise*<Uint8Array\>

Defined in: [db.ts:317](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L317)

___

### storeUnacknowledgedTickets

▸ **storeUnacknowledgedTickets**(`key`: [*PublicKey*](publickey.md), `unacknowledged`: [*UnacknowledgedTicket*](unacknowledgedticket.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | [*PublicKey*](publickey.md) |
| `unacknowledged` | [*UnacknowledgedTicket*](unacknowledgedticket.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:256](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L256)

___

### touch

▸ `Private` **touch**(`key`: *Uint8Array*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |

**Returns:** *Promise*<void\>

Defined in: [db.ts:93](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L93)

___

### updateAccount

▸ **updateAccount**(`account`: [*AccountEntry*](accountentry.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | [*AccountEntry*](accountentry.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:386](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L386)

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

Defined in: [db.ts:218](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L218)

___

### updateChannel

▸ **updateChannel**(`channelId`: [*Hash*](hash.md), `channel`: [*ChannelEntry*](channelentry.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [*Hash*](hash.md) |
| `channel` | [*ChannelEntry*](channelentry.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:377](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L377)

___

### updateLatestBlockNumber

▸ **updateLatestBlockNumber**(`blockNumber`: *BN*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `blockNumber` | *BN* |

**Returns:** *Promise*<void\>

Defined in: [db.ts:354](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L354)

___

### updateLatestConfirmedSnapshot

▸ **updateLatestConfirmedSnapshot**(`snapshot`: [*Snapshot*](snapshot.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `snapshot` | [*Snapshot*](snapshot.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:363](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L363)

___

### createMock

▸ `Static` **createMock**(): [*HoprDB*](hoprdb.md)

**Returns:** [*HoprDB*](hoprdb.md)

Defined in: [db.ts:395](https://github.com/jlherren/hoprnet/blob/master/packages/utils/src/db.ts#L395)
