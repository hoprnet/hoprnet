[@hoprnet/hopr-ethereum](../README.md) / [Exports](../modules.md) / TypedEvent

# Interface: TypedEvent<EventArgs\>

## Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgs` | extends `Result` |

## Hierarchy

- `Event`

  ↳ **`TypedEvent`**

## Table of contents

### Properties

- [address](TypedEvent.md#address)
- [args](TypedEvent.md#args)
- [blockHash](TypedEvent.md#blockhash)
- [blockNumber](TypedEvent.md#blocknumber)
- [data](TypedEvent.md#data)
- [decodeError](TypedEvent.md#decodeerror)
- [event](TypedEvent.md#event)
- [eventSignature](TypedEvent.md#eventsignature)
- [logIndex](TypedEvent.md#logindex)
- [removed](TypedEvent.md#removed)
- [topics](TypedEvent.md#topics)
- [transactionHash](TypedEvent.md#transactionhash)
- [transactionIndex](TypedEvent.md#transactionindex)

### Methods

- [decode](TypedEvent.md#decode)
- [getBlock](TypedEvent.md#getblock)
- [getTransaction](TypedEvent.md#gettransaction)
- [getTransactionReceipt](TypedEvent.md#gettransactionreceipt)
- [removeListener](TypedEvent.md#removelistener)

## Properties

### address

• **address**: `string`

#### Inherited from

Event.address

#### Defined in

node_modules/@ethersproject/abstract-provider/lib/index.d.ts:58

___

### args

• **args**: `EventArgs`

#### Overrides

Event.args

#### Defined in

packages/ethereum/src/types/common.d.ts:11

___

### blockHash

• **blockHash**: `string`

#### Inherited from

Event.blockHash

#### Defined in

node_modules/@ethersproject/abstract-provider/lib/index.d.ts:55

___

### blockNumber

• **blockNumber**: `number`

#### Inherited from

Event.blockNumber

#### Defined in

node_modules/@ethersproject/abstract-provider/lib/index.d.ts:54

___

### data

• **data**: `string`

#### Inherited from

Event.data

#### Defined in

node_modules/@ethersproject/abstract-provider/lib/index.d.ts:59

___

### decodeError

• `Optional` **decodeError**: `Error`

#### Inherited from

Event.decodeError

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:48

___

### event

• `Optional` **event**: `string`

#### Inherited from

Event.event

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:45

___

### eventSignature

• `Optional` **eventSignature**: `string`

#### Inherited from

Event.eventSignature

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:46

___

### logIndex

• **logIndex**: `number`

#### Inherited from

Event.logIndex

#### Defined in

node_modules/@ethersproject/abstract-provider/lib/index.d.ts:62

___

### removed

• **removed**: `boolean`

#### Inherited from

Event.removed

#### Defined in

node_modules/@ethersproject/abstract-provider/lib/index.d.ts:57

___

### topics

• **topics**: `string`[]

#### Inherited from

Event.topics

#### Defined in

node_modules/@ethersproject/abstract-provider/lib/index.d.ts:60

___

### transactionHash

• **transactionHash**: `string`

#### Inherited from

Event.transactionHash

#### Defined in

node_modules/@ethersproject/abstract-provider/lib/index.d.ts:61

___

### transactionIndex

• **transactionIndex**: `number`

#### Inherited from

Event.transactionIndex

#### Defined in

node_modules/@ethersproject/abstract-provider/lib/index.d.ts:56

## Methods

### decode

▸ `Optional` **decode**(`data`, `topics?`): `any`

#### Parameters

| Name | Type |
| :------ | :------ |
| `data` | `string` |
| `topics?` | `string`[] |

#### Returns

`any`

#### Inherited from

Event.decode

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:49

___

### getBlock

▸ **getBlock**(): `Promise`<`Block`\>

#### Returns

`Promise`<`Block`\>

#### Inherited from

Event.getBlock

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:51

___

### getTransaction

▸ **getTransaction**(): `Promise`<`TransactionResponse`\>

#### Returns

`Promise`<`TransactionResponse`\>

#### Inherited from

Event.getTransaction

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:52

___

### getTransactionReceipt

▸ **getTransactionReceipt**(): `Promise`<`TransactionReceipt`\>

#### Returns

`Promise`<`TransactionReceipt`\>

#### Inherited from

Event.getTransactionReceipt

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:53

___

### removeListener

▸ **removeListener**(): `void`

#### Returns

`void`

#### Inherited from

Event.removeListener

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:50
