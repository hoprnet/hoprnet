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

- [address](typedevent.md#address)
- [args](typedevent.md#args)
- [blockHash](typedevent.md#blockhash)
- [blockNumber](typedevent.md#blocknumber)
- [data](typedevent.md#data)
- [decode](typedevent.md#decode)
- [decodeError](typedevent.md#decodeerror)
- [event](typedevent.md#event)
- [eventSignature](typedevent.md#eventsignature)
- [getBlock](typedevent.md#getblock)
- [getTransaction](typedevent.md#gettransaction)
- [getTransactionReceipt](typedevent.md#gettransactionreceipt)
- [logIndex](typedevent.md#logindex)
- [removeListener](typedevent.md#removelistener)
- [removed](typedevent.md#removed)
- [topics](typedevent.md#topics)
- [transactionHash](typedevent.md#transactionhash)
- [transactionIndex](typedevent.md#transactionindex)

## Properties

### address

• **address**: `string`

#### Inherited from

Event.address

#### Defined in

node_modules/@ethersproject/abstract-provider/lib/index.d.ts:56

___

### args

• **args**: `EventArgs`

#### Overrides

Event.args

#### Defined in

packages/ethereum/types/commons.ts:12

___

### blockHash

• **blockHash**: `string`

#### Inherited from

Event.blockHash

#### Defined in

node_modules/@ethersproject/abstract-provider/lib/index.d.ts:53

___

### blockNumber

• **blockNumber**: `number`

#### Inherited from

Event.blockNumber

#### Defined in

node_modules/@ethersproject/abstract-provider/lib/index.d.ts:52

___

### data

• **data**: `string`

#### Inherited from

Event.data

#### Defined in

node_modules/@ethersproject/abstract-provider/lib/index.d.ts:57

___

### decode

• `Optional` **decode**: (`data`: `string`, `topics?`: `string`[]) => `any`

#### Type declaration

▸ (`data`, `topics?`): `any`

##### Parameters

| Name | Type |
| :------ | :------ |
| `data` | `string` |
| `topics?` | `string`[] |

##### Returns

`any`

#### Inherited from

Event.decode

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:47

___

### decodeError

• `Optional` **decodeError**: `Error`

#### Inherited from

Event.decodeError

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:46

___

### event

• `Optional` **event**: `string`

#### Inherited from

Event.event

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:43

___

### eventSignature

• `Optional` **eventSignature**: `string`

#### Inherited from

Event.eventSignature

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:44

___

### getBlock

• **getBlock**: () => `Promise`<`Block`\>

#### Type declaration

▸ (): `Promise`<`Block`\>

##### Returns

`Promise`<`Block`\>

#### Inherited from

Event.getBlock

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:49

___

### getTransaction

• **getTransaction**: () => `Promise`<`TransactionResponse`\>

#### Type declaration

▸ (): `Promise`<`TransactionResponse`\>

##### Returns

`Promise`<`TransactionResponse`\>

#### Inherited from

Event.getTransaction

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:50

___

### getTransactionReceipt

• **getTransactionReceipt**: () => `Promise`<`TransactionReceipt`\>

#### Type declaration

▸ (): `Promise`<`TransactionReceipt`\>

##### Returns

`Promise`<`TransactionReceipt`\>

#### Inherited from

Event.getTransactionReceipt

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:51

___

### logIndex

• **logIndex**: `number`

#### Inherited from

Event.logIndex

#### Defined in

node_modules/@ethersproject/abstract-provider/lib/index.d.ts:60

___

### removeListener

• **removeListener**: () => `void`

#### Type declaration

▸ (): `void`

##### Returns

`void`

#### Inherited from

Event.removeListener

#### Defined in

node_modules/@ethersproject/contracts/lib/index.d.ts:48

___

### removed

• **removed**: `boolean`

#### Inherited from

Event.removed

#### Defined in

node_modules/@ethersproject/abstract-provider/lib/index.d.ts:55

___

### topics

• **topics**: `string`[]

#### Inherited from

Event.topics

#### Defined in

node_modules/@ethersproject/abstract-provider/lib/index.d.ts:58

___

### transactionHash

• **transactionHash**: `string`

#### Inherited from

Event.transactionHash

#### Defined in

node_modules/@ethersproject/abstract-provider/lib/index.d.ts:59

___

### transactionIndex

• **transactionIndex**: `number`

#### Inherited from

Event.transactionIndex

#### Defined in

node_modules/@ethersproject/abstract-provider/lib/index.d.ts:54
