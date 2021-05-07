[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [contracts/commons](../modules/contracts_commons.md) / TypedEvent

# Interface: TypedEvent<EventArgs\>

[contracts/commons](../modules/contracts_commons.md).TypedEvent

## Type parameters

| Name        | Type   |
| :---------- | :----- |
| `EventArgs` | Result |

## Hierarchy

- _Event_

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

• **address**: _string_

Inherited from: Event.address

Defined in: node_modules/@ethersproject/abstract-provider/lib/index.d.ts:53

---

### args

• **args**: EventArgs

Overrides: Event.args

Defined in: packages/core-ethereum/src/contracts/commons.ts:12

---

### blockHash

• **blockHash**: _string_

Inherited from: Event.blockHash

Defined in: node_modules/@ethersproject/abstract-provider/lib/index.d.ts:50

---

### blockNumber

• **blockNumber**: _number_

Inherited from: Event.blockNumber

Defined in: node_modules/@ethersproject/abstract-provider/lib/index.d.ts:49

---

### data

• **data**: _string_

Inherited from: Event.data

Defined in: node_modules/@ethersproject/abstract-provider/lib/index.d.ts:54

---

### decode

• `Optional` **decode**: (`data`: _string_, `topics?`: _string_[]) => _any_

#### Type declaration

▸ (`data`: _string_, `topics?`: _string_[]): _any_

#### Parameters

| Name      | Type       |
| :-------- | :--------- |
| `data`    | _string_   |
| `topics?` | _string_[] |

**Returns:** _any_

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:43

Inherited from: Event.decode

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:43

---

### decodeError

• `Optional` **decodeError**: Error

Inherited from: Event.decodeError

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:42

---

### event

• `Optional` **event**: _string_

Inherited from: Event.event

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:39

---

### eventSignature

• `Optional` **eventSignature**: _string_

Inherited from: Event.eventSignature

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:40

---

### getBlock

• **getBlock**: () => _Promise_<Block\>

#### Type declaration

▸ (): _Promise_<Block\>

**Returns:** _Promise_<Block\>

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:45

Inherited from: Event.getBlock

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:45

---

### getTransaction

• **getTransaction**: () => _Promise_<TransactionResponse\>

#### Type declaration

▸ (): _Promise_<TransactionResponse\>

**Returns:** _Promise_<TransactionResponse\>

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:46

Inherited from: Event.getTransaction

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:46

---

### getTransactionReceipt

• **getTransactionReceipt**: () => _Promise_<TransactionReceipt\>

#### Type declaration

▸ (): _Promise_<TransactionReceipt\>

**Returns:** _Promise_<TransactionReceipt\>

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:47

Inherited from: Event.getTransactionReceipt

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:47

---

### logIndex

• **logIndex**: _number_

Inherited from: Event.logIndex

Defined in: node_modules/@ethersproject/abstract-provider/lib/index.d.ts:57

---

### removeListener

• **removeListener**: () => _void_

#### Type declaration

▸ (): _void_

**Returns:** _void_

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:44

Inherited from: Event.removeListener

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:44

---

### removed

• **removed**: _boolean_

Inherited from: Event.removed

Defined in: node_modules/@ethersproject/abstract-provider/lib/index.d.ts:52

---

### topics

• **topics**: _string_[]

Inherited from: Event.topics

Defined in: node_modules/@ethersproject/abstract-provider/lib/index.d.ts:55

---

### transactionHash

• **transactionHash**: _string_

Inherited from: Event.transactionHash

Defined in: node_modules/@ethersproject/abstract-provider/lib/index.d.ts:56

---

### transactionIndex

• **transactionIndex**: _number_

Inherited from: Event.transactionIndex

Defined in: node_modules/@ethersproject/abstract-provider/lib/index.d.ts:51
