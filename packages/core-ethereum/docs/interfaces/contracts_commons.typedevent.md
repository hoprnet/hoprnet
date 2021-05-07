[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [contracts/commons](../modules/contracts_commons.md) / TypedEvent

# Interface: TypedEvent<EventArgs\>

[contracts/commons](../modules/contracts_commons.md).TypedEvent

## Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgs` | Result |

## Hierarchy

- *Event*

  ↳ **TypedEvent**

## Table of contents

### Properties

- [address](contracts_commons.typedevent.md#address)
- [args](contracts_commons.typedevent.md#args)
- [blockHash](contracts_commons.typedevent.md#blockhash)
- [blockNumber](contracts_commons.typedevent.md#blocknumber)
- [data](contracts_commons.typedevent.md#data)
- [decode](contracts_commons.typedevent.md#decode)
- [decodeError](contracts_commons.typedevent.md#decodeerror)
- [event](contracts_commons.typedevent.md#event)
- [eventSignature](contracts_commons.typedevent.md#eventsignature)
- [getBlock](contracts_commons.typedevent.md#getblock)
- [getTransaction](contracts_commons.typedevent.md#gettransaction)
- [getTransactionReceipt](contracts_commons.typedevent.md#gettransactionreceipt)
- [logIndex](contracts_commons.typedevent.md#logindex)
- [removeListener](contracts_commons.typedevent.md#removelistener)
- [removed](contracts_commons.typedevent.md#removed)
- [topics](contracts_commons.typedevent.md#topics)
- [transactionHash](contracts_commons.typedevent.md#transactionhash)
- [transactionIndex](contracts_commons.typedevent.md#transactionindex)

## Properties

### address

• **address**: *string*

Inherited from: Event.address

Defined in: node_modules/@ethersproject/abstract-provider/lib/index.d.ts:53

___

### args

• **args**: EventArgs

Overrides: Event.args

Defined in: packages/core-ethereum/src/contracts/commons.ts:12

___

### blockHash

• **blockHash**: *string*

Inherited from: Event.blockHash

Defined in: node_modules/@ethersproject/abstract-provider/lib/index.d.ts:50

___

### blockNumber

• **blockNumber**: *number*

Inherited from: Event.blockNumber

Defined in: node_modules/@ethersproject/abstract-provider/lib/index.d.ts:49

___

### data

• **data**: *string*

Inherited from: Event.data

Defined in: node_modules/@ethersproject/abstract-provider/lib/index.d.ts:54

___

### decode

• `Optional` **decode**: (`data`: *string*, `topics?`: *string*[]) => *any*

#### Type declaration

▸ (`data`: *string*, `topics?`: *string*[]): *any*

#### Parameters

| Name | Type |
| :------ | :------ |
| `data` | *string* |
| `topics?` | *string*[] |

**Returns:** *any*

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:43

Inherited from: Event.decode

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:43

___

### decodeError

• `Optional` **decodeError**: Error

Inherited from: Event.decodeError

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:42

___

### event

• `Optional` **event**: *string*

Inherited from: Event.event

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:39

___

### eventSignature

• `Optional` **eventSignature**: *string*

Inherited from: Event.eventSignature

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:40

___

### getBlock

• **getBlock**: () => *Promise*<Block\>

#### Type declaration

▸ (): *Promise*<Block\>

**Returns:** *Promise*<Block\>

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:45

Inherited from: Event.getBlock

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:45

___

### getTransaction

• **getTransaction**: () => *Promise*<TransactionResponse\>

#### Type declaration

▸ (): *Promise*<TransactionResponse\>

**Returns:** *Promise*<TransactionResponse\>

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:46

Inherited from: Event.getTransaction

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:46

___

### getTransactionReceipt

• **getTransactionReceipt**: () => *Promise*<TransactionReceipt\>

#### Type declaration

▸ (): *Promise*<TransactionReceipt\>

**Returns:** *Promise*<TransactionReceipt\>

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:47

Inherited from: Event.getTransactionReceipt

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:47

___

### logIndex

• **logIndex**: *number*

Inherited from: Event.logIndex

Defined in: node_modules/@ethersproject/abstract-provider/lib/index.d.ts:57

___

### removeListener

• **removeListener**: () => *void*

#### Type declaration

▸ (): *void*

**Returns:** *void*

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:44

Inherited from: Event.removeListener

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:44

___

### removed

• **removed**: *boolean*

Inherited from: Event.removed

Defined in: node_modules/@ethersproject/abstract-provider/lib/index.d.ts:52

___

### topics

• **topics**: *string*[]

Inherited from: Event.topics

Defined in: node_modules/@ethersproject/abstract-provider/lib/index.d.ts:55

___

### transactionHash

• **transactionHash**: *string*

Inherited from: Event.transactionHash

Defined in: node_modules/@ethersproject/abstract-provider/lib/index.d.ts:56

___

### transactionIndex

• **transactionIndex**: *number*

Inherited from: Event.transactionIndex

Defined in: node_modules/@ethersproject/abstract-provider/lib/index.d.ts:51
