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

\+ **new HoprDB**(`id`: [*Address*](address.md), `initialize`: *boolean*, `version`: *string*, `dbPath?`: *string*): [*HoprDB*](hoprdb.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `id` | [*Address*](address.md) |
| `initialize` | *boolean* |
| `version` | *string* |
| `dbPath?` | *string* |

**Returns:** [*HoprDB*](hoprdb.md)

Defined in: [db.ts:44](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L44)

## Properties

### db

• `Private` **db**: *LevelUp*<AbstractLevelDOWN<any, any\>, AbstractIterator<any, any\>\>

Defined in: [db.ts:44](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L44)

## Methods

### checkAndSetPacketTag

▸ **checkAndSetPacketTag**(`packetTag`: *Uint8Array*): *Promise*<boolean\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `packetTag` | *Uint8Array* |

**Returns:** *Promise*<boolean\>

Defined in: [db.ts:223](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L223)

___

### close

▸ **close**(): *Promise*<void\>

**Returns:** *Promise*<void\>

Defined in: [db.ts:233](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L233)

___

### del

▸ `Private` **del**(`key`: *Uint8Array*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |

**Returns:** *Promise*<void\>

Defined in: [db.ts:131](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L131)

___

### delAcknowledgedTicket

▸ **delAcknowledgedTicket**(`challenge`: [*EthereumChallenge*](ethereumchallenge.md)): *Promise*<void\>

Delete acknowledged ticket in database

#### Parameters

| Name | Type |
| :------ | :------ |
| `challenge` | [*EthereumChallenge*](ethereumchallenge.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:188](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L188)

___

### get

▸ `Private` **get**(`key`: *Uint8Array*): *Promise*<Uint8Array\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |

**Returns:** *Promise*<Uint8Array\>

Defined in: [db.ts:92](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L92)

___

### getAccount

▸ **getAccount**(`address`: [*Address*](address.md)): *Promise*<[*AccountEntry*](accountentry.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | [*Address*](address.md) |

**Returns:** *Promise*<[*AccountEntry*](accountentry.md)\>

Defined in: [db.ts:291](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L291)

___

### getAccounts

▸ **getAccounts**(`filter?`: (`account`: [*AccountEntry*](accountentry.md)) => *boolean*): *Promise*<[*AccountEntry*](accountentry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `filter?` | (`account`: [*AccountEntry*](accountentry.md)) => *boolean* |

**Returns:** *Promise*<[*AccountEntry*](accountentry.md)[]\>

Defined in: [db.ts:300](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L300)

___

### getAcknowledgedTickets

▸ **getAcknowledgedTickets**(`filter?`: { `signer`: [*PublicKey*](publickey.md)  }): *Promise*<[*AcknowledgedTicket*](acknowledgedticket.md)[]\>

Get acknowledged tickets

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `filter?` | *object* | optionally filter by signer |
| `filter.signer` | [*PublicKey*](publickey.md) | - |

**Returns:** *Promise*<[*AcknowledgedTicket*](acknowledgedticket.md)[]\>

an array of all acknowledged tickets

Defined in: [db.ts:172](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L172)

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

Defined in: [db.ts:107](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L107)

___

### getChannel

▸ **getChannel**(`channelId`: [*Hash*](hash.md)): *Promise*<[*ChannelEntry*](channelentry.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [*Hash*](hash.md) |

**Returns:** *Promise*<[*ChannelEntry*](channelentry.md)\>

Defined in: [db.ts:277](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L277)

___

### getChannels

▸ **getChannels**(`filter?`: (`channel`: [*ChannelEntry*](channelentry.md)) => *boolean*): *Promise*<[*ChannelEntry*](channelentry.md)[]\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `filter?` | (`channel`: [*ChannelEntry*](channelentry.md)) => *boolean* |

**Returns:** *Promise*<[*ChannelEntry*](channelentry.md)[]\>

Defined in: [db.ts:282](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L282)

___

### getCommitment

▸ **getCommitment**(`channelId`: [*Hash*](hash.md), `iteration`: *number*): *Promise*<Uint8Array\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [*Hash*](hash.md) |
| `iteration` | *number* |

**Returns:** *Promise*<Uint8Array\>

Defined in: [db.ts:247](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L247)

___

### getCurrentCommitment

▸ **getCurrentCommitment**(`channelId`: [*Hash*](hash.md)): *Promise*<[*Hash*](hash.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [*Hash*](hash.md) |

**Returns:** *Promise*<[*Hash*](hash.md)\>

Defined in: [db.ts:251](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L251)

___

### getLatestBlockNumber

▸ **getLatestBlockNumber**(): *Promise*<number\>

**Returns:** *Promise*<number\>

Defined in: [db.ts:259](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L259)

___

### getLatestConfirmedSnapshot

▸ **getLatestConfirmedSnapshot**(): *Promise*<[*Snapshot*](snapshot.md)\>

**Returns:** *Promise*<[*Snapshot*](snapshot.md)\>

Defined in: [db.ts:268](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L268)

___

### getTickets

▸ **getTickets**(`filter?`: { `signer`: [*PublicKey*](publickey.md)  }): *Promise*<[*Ticket*](ticket.md)[]\>

Get tickets, both unacknowledged and acknowledged

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `filter?` | *object* | optionally filter by signer |
| `filter.signer` | [*PublicKey*](publickey.md) | - |

**Returns:** *Promise*<[*Ticket*](ticket.md)[]\>

an array of signed tickets

Defined in: [db.ts:213](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L213)

___

### getUnacknowledgedTicket

▸ **getUnacknowledgedTicket**(`halfKeyChallenge`: [*HalfKeyChallenge*](halfkeychallenge.md)): *Promise*<[*UnacknowledgedTicket*](unacknowledgedticket.md)\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `halfKeyChallenge` | [*HalfKeyChallenge*](halfkeychallenge.md) |

**Returns:** *Promise*<[*UnacknowledgedTicket*](unacknowledgedticket.md)\>

Defined in: [db.ts:156](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L156)

___

### getUnacknowledgedTickets

▸ **getUnacknowledgedTickets**(`filter?`: { `signer`: [*PublicKey*](publickey.md)  }): *Promise*<[*UnacknowledgedTicket*](unacknowledgedticket.md)[]\>

Get unacknowledged tickets.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `filter?` | *object* | optionally filter by signer |
| `filter.signer` | [*PublicKey*](publickey.md) | - |

**Returns:** *Promise*<[*UnacknowledgedTicket*](unacknowledgedticket.md)[]\>

an array of all unacknowledged tickets

Defined in: [db.ts:140](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L140)

___

### has

▸ `Private` **has**(`key`: *Uint8Array*): *Promise*<boolean\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |

**Returns:** *Promise*<boolean\>

Defined in: [db.ts:70](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L70)

___

### keyOf

▸ `Private` **keyOf**(...`segments`: *Uint8Array*[]): *Uint8Array*

#### Parameters

| Name | Type |
| :------ | :------ |
| `...segments` | *Uint8Array*[] |

**Returns:** *Uint8Array*

Defined in: [db.ts:66](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L66)

___

### maybeGet

▸ `Private` **maybeGet**(`key`: *Uint8Array*): *Promise*<Uint8Array\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |

**Returns:** *Promise*<Uint8Array\>

Defined in: [db.ts:96](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L96)

___

### put

▸ `Private` **put**(`key`: *Uint8Array*, `value`: *Uint8Array*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |
| `value` | *Uint8Array* |

**Returns:** *Promise*<void\>

Defined in: [db.ts:84](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L84)

___

### replaceUnAckWithAck

▸ **replaceUnAckWithAck**(`halfKeyChallenge`: [*HalfKeyChallenge*](halfkeychallenge.md), `ackTicket`: [*AcknowledgedTicket*](acknowledgedticket.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `halfKeyChallenge` | [*HalfKeyChallenge*](halfkeychallenge.md) |
| `ackTicket` | [*AcknowledgedTicket*](acknowledgedticket.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:192](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L192)

___

### setCurrentCommitment

▸ **setCurrentCommitment**(`channelId`: [*Hash*](hash.md), `commitment`: [*Hash*](hash.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [*Hash*](hash.md) |
| `commitment` | [*Hash*](hash.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:255](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L255)

___

### storeHashIntermediaries

▸ **storeHashIntermediaries**(`channelId`: [*Hash*](hash.md), `intermediates`: [*Intermediate*](../interfaces/intermediate.md)[]): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [*Hash*](hash.md) |
| `intermediates` | [*Intermediate*](../interfaces/intermediate.md)[] |

**Returns:** *Promise*<void\>

Defined in: [db.ts:237](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L237)

___

### storeUnacknowledgedTicket

▸ **storeUnacknowledgedTicket**(`halfKeyChallenge`: [*HalfKeyChallenge*](halfkeychallenge.md), `unackTicket`: [*UnacknowledgedTicket*](unacknowledgedticket.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `halfKeyChallenge` | [*HalfKeyChallenge*](halfkeychallenge.md) |
| `unackTicket` | [*UnacknowledgedTicket*](unacknowledgedticket.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:160](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L160)

___

### touch

▸ `Private` **touch**(`key`: *Uint8Array*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `key` | *Uint8Array* |

**Returns:** *Promise*<void\>

Defined in: [db.ts:88](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L88)

___

### updateAccount

▸ **updateAccount**(`account`: [*AccountEntry*](accountentry.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account` | [*AccountEntry*](accountentry.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:296](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L296)

___

### updateChannel

▸ **updateChannel**(`channelId`: [*Hash*](hash.md), `channel`: [*ChannelEntry*](channelentry.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `channelId` | [*Hash*](hash.md) |
| `channel` | [*ChannelEntry*](channelentry.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:287](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L287)

___

### updateLatestBlockNumber

▸ **updateLatestBlockNumber**(`blockNumber`: *BN*): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `blockNumber` | *BN* |

**Returns:** *Promise*<void\>

Defined in: [db.ts:264](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L264)

___

### updateLatestConfirmedSnapshot

▸ **updateLatestConfirmedSnapshot**(`snapshot`: [*Snapshot*](snapshot.md)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `snapshot` | [*Snapshot*](snapshot.md) |

**Returns:** *Promise*<void\>

Defined in: [db.ts:273](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L273)

___

### createMock

▸ `Static` **createMock**(): [*HoprDB*](hoprdb.md)

**Returns:** [*HoprDB*](hoprdb.md)

Defined in: [db.ts:305](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L305)
