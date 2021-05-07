[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [transaction-manager](../modules/transaction_manager.md) / default

# Class: default

[transaction-manager](../modules/transaction_manager.md).default

Keep track of pending and confirmed transactions,
and allows for pruning unnecessary data.
This class is mainly used by nonce-tracker which relies
on transcation-manager to keep an update to date view
on transactions.

## Table of contents

### Constructors

- [constructor](transaction_manager.default.md#constructor)

### Properties

- [confirmed](transaction_manager.default.md#confirmed)
- [pending](transaction_manager.default.md#pending)

### Methods

- [\_getTime](transaction_manager.default.md#_gettime)
- [addToPending](transaction_manager.default.md#addtopending)
- [moveToConfirmed](transaction_manager.default.md#movetoconfirmed)
- [prune](transaction_manager.default.md#prune)
- [remove](transaction_manager.default.md#remove)

## Constructors

### constructor

\+ **new default**(): [_default_](transaction_manager.default.md)

**Returns:** [_default_](transaction_manager.default.md)

## Properties

### confirmed

• `Readonly` **confirmed**: _Map_<string, [_Transaction_](../modules/transaction_manager.md#transaction)\>

confirmed transactions

Defined in: [packages/core-ethereum/src/transaction-manager.ts:24](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/transaction-manager.ts#L24)

---

### pending

• `Readonly` **pending**: _Map_<string, [_Transaction_](../modules/transaction_manager.md#transaction)\>

pending transactions

Defined in: [packages/core-ethereum/src/transaction-manager.ts:20](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/transaction-manager.ts#L20)

## Methods

### \_getTime

▸ `Private` **\_getTime**(): _number_

**Returns:** _number_

current timestamp

Defined in: [packages/core-ethereum/src/transaction-manager.ts:78](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/transaction-manager.ts#L78)

---

### addToPending

▸ **addToPending**(`hash`: _string_, `transaction`: _Pick_<[_Transaction_](../modules/transaction_manager.md#transaction), `"nonce"`\>): _void_

Adds transaction in pending

#### Parameters

| Name          | Type                                                                               | Description      |
| :------------ | :--------------------------------------------------------------------------------- | :--------------- |
| `hash`        | _string_                                                                           | transaction hash |
| `transaction` | _Pick_<[_Transaction_](../modules/transaction_manager.md#transaction), `"nonce"`\> | object           |

**Returns:** _void_

Defined in: [packages/core-ethereum/src/transaction-manager.ts:31](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/transaction-manager.ts#L31)

---

### moveToConfirmed

▸ **moveToConfirmed**(`hash`: _string_): _void_

Moves transcation from pending to confirmed

#### Parameters

| Name   | Type     | Description      |
| :----- | :------- | :--------------- |
| `hash` | _string_ | transaction hash |

**Returns:** _void_

Defined in: [packages/core-ethereum/src/transaction-manager.ts:42](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/transaction-manager.ts#L42)

---

### prune

▸ **prune**(): _void_

Removes confirmed blocks except last 5 nonces.
This is a way for us to clean up some memory which we know
we don't need anymore.

**Returns:** _void_

Defined in: [packages/core-ethereum/src/transaction-manager.ts:65](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/transaction-manager.ts#L65)

---

### remove

▸ **remove**(`hash`: _string_): _void_

Removed transcation from pending and confirmed

#### Parameters

| Name   | Type     | Description      |
| :----- | :------- | :--------------- |
| `hash` | _string_ | transaction hash |

**Returns:** _void_

Defined in: [packages/core-ethereum/src/transaction-manager.ts:54](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/transaction-manager.ts#L54)
