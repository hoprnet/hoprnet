[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [nonce-tracker](../modules/nonce_tracker.md) / default

# Class: default

[nonce-tracker](../modules/nonce_tracker.md).default

A simple nonce-tracker that takes into account nonces which have failed,
this could happen because of network issues, low funds, node unavailability, etc.

## Table of contents

### Constructors

- [constructor](nonce_tracker.default.md#constructor)

### Properties

- [lockMap](nonce_tracker.default.md#lockmap)

### Methods

- [\_containsStuckTx](nonce_tracker.default.md#_containsstucktx)
- [\_getHighestContinuousFrom](nonce_tracker.default.md#_gethighestcontinuousfrom)
- [\_getHighestLocallyConfirmed](nonce_tracker.default.md#_gethighestlocallyconfirmed)
- [\_getHighestNonce](nonce_tracker.default.md#_gethighestnonce)
- [\_getNetworkNextNonce](nonce_tracker.default.md#_getnetworknextnonce)
- [\_globalMutexFree](nonce_tracker.default.md#_globalmutexfree)
- [\_lookupMutex](nonce_tracker.default.md#_lookupmutex)
- [\_takeMutex](nonce_tracker.default.md#_takemutex)
- [getGlobalLock](nonce_tracker.default.md#getgloballock)
- [getNonceLock](nonce_tracker.default.md#getnoncelock)

## Constructors

### constructor

\+ **new default**(`api`: { `getConfirmedTransactions`: (`address`: _Address_) => [_Transaction_](../modules/nonce_tracker.md#transaction)[] ; `getLatestBlockNumber`: () => _Promise_<number\> ; `getPendingTransactions`: (`address`: _Address_) => [_Transaction_](../modules/nonce_tracker.md#transaction)[] ; `getTransactionCount`: (`address`: _Address_, `blockNumber?`: _number_) => _Promise_<number\> }, `minPending?`: _number_): [_default_](nonce_tracker.default.md)

#### Parameters

| Name                           | Type                                                                                 |
| :----------------------------- | :----------------------------------------------------------------------------------- |
| `api`                          | _object_                                                                             |
| `api.getConfirmedTransactions` | (`address`: _Address_) => [_Transaction_](../modules/nonce_tracker.md#transaction)[] |
| `api.getLatestBlockNumber`     | () => _Promise_<number\>                                                             |
| `api.getPendingTransactions`   | (`address`: _Address_) => [_Transaction_](../modules/nonce_tracker.md#transaction)[] |
| `api.getTransactionCount`      | (`address`: _Address_, `blockNumber?`: _number_) => _Promise_<number\>               |
| `minPending?`                  | _number_                                                                             |

**Returns:** [_default_](nonce_tracker.default.md)

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:77](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L77)

## Properties

### lockMap

• `Private` **lockMap**: _Record_<string, Mutex\>

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:77](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L77)

## Methods

### \_containsStuckTx

▸ `Private` **\_containsStuckTx**(`txs`: [_Transaction_](../modules/nonce_tracker.md#transaction)[]): _boolean_

It's possible we encounter transactions that are pending for a very long time,
this can happen if a transaction is under-funded.
This function will return `true` if it finds a pending transaction that has
been pending for more than {minPending} ms.

#### Parameters

| Name  | Type                                                       |
| :---- | :--------------------------------------------------------- |
| `txs` | [_Transaction_](../modules/nonce_tracker.md#transaction)[] |

**Returns:** _boolean_

true if it contains a stuck transaction

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:163](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L163)

---

### \_getHighestContinuousFrom

▸ `Private` **\_getHighestContinuousFrom**(`txList`: [_Transaction_](../modules/nonce_tracker.md#transaction)[], `startPoint`: _number_): [_HighestContinuousFrom_](../modules/nonce_tracker.md#highestcontinuousfrom)

Function return the nonce value higher than the highest nonce value from the transaction list
starting from startPoint

#### Parameters

| Name         | Type                                                       | Description                               |
| :----------- | :--------------------------------------------------------- | :---------------------------------------- |
| `txList`     | [_Transaction_](../modules/nonce_tracker.md#transaction)[] | list of txMeta's                          |
| `startPoint` | _number_                                                   | the highest known locally confirmed nonce |

**Returns:** [_HighestContinuousFrom_](../modules/nonce_tracker.md#highestcontinuousfrom)

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:246](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L246)

---

### \_getHighestLocallyConfirmed

▸ `Private` **\_getHighestLocallyConfirmed**(`address`: _Address_): _number_

Function returns the highest of the confirmed transaction from the address.

#### Parameters

| Name      | Type      | Description                                                   |
| :-------- | :-------- | :------------------------------------------------------------ |
| `address` | _Address_ | the hex string for the address whose nonce we are calculating |

**Returns:** _number_

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:220](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L220)

---

### \_getHighestNonce

▸ `Private` **\_getHighestNonce**(`txList`: [_Transaction_](../modules/nonce_tracker.md#transaction)[]): _number_

Function returns highest nonce value from the transcation list provided

#### Parameters

| Name     | Type                                                       | Description          |
| :------- | :--------------------------------------------------------- | :------------------- |
| `txList` | [_Transaction_](../modules/nonce_tracker.md#transaction)[] | list of transactions |

**Returns:** _number_

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:230](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L230)

---

### \_getNetworkNextNonce

▸ `Private` **\_getNetworkNextNonce**(`address`: _Address_): _Promise_<[_NetworkNextNonce_](../modules/nonce_tracker.md#networknextnonce)\>

Function returns the nonce details from teh network based on the latest block
and eth_getTransactionCount method

#### Parameters

| Name      | Type      | Description                                                   |
| :-------- | :-------- | :------------------------------------------------------------ |
| `address` | _Address_ | the hex string for the address whose nonce we are calculating |

**Returns:** _Promise_<[_NetworkNextNonce_](../modules/nonce_tracker.md#networknextnonce)\>

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:193](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L193)

---

### \_globalMutexFree

▸ `Private` **\_globalMutexFree**(): _Promise_<void\>

**Returns:** _Promise_<void\>

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:174](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L174)

---

### \_lookupMutex

▸ `Private` **\_lookupMutex**(`lockId`: _string_): _Mutex_

#### Parameters

| Name     | Type     |
| :------- | :------- |
| `lockId` | _string_ |

**Returns:** _Mutex_

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:207](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L207)

---

### \_takeMutex

▸ `Private` **\_takeMutex**(`lockId`: _string_): _Promise_<() => _void_\>

#### Parameters

| Name     | Type     |
| :------- | :------- |
| `lockId` | _string_ |

**Returns:** _Promise_<() => _void_\>

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:180](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L180)

---

### getGlobalLock

▸ **getGlobalLock**(): _Promise_<{ `releaseLock`: () => _void_ }\>

**Returns:** _Promise_<{ `releaseLock`: () => _void_ }\>

Promise<{ releaseLock: () => void }> with the key releaseLock (the global mutex)

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:94](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L94)

---

### getNonceLock

▸ **getNonceLock**(`address`: _Address_): _Promise_<[_NonceLock_](../interfaces/nonce_tracker.noncelock.md)\>

this will return an object with the `nextNonce` `nonceDetails`, and the releaseLock
Note: releaseLock must be called after adding a signed tx to pending transactions (or discarding).

#### Parameters

| Name      | Type      | Description                                                   |
| :-------- | :-------- | :------------------------------------------------------------ |
| `address` | _Address_ | the hex string for the address whose nonce we are calculating |

**Returns:** _Promise_<[_NonceLock_](../interfaces/nonce_tracker.noncelock.md)\>

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:108](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L108)
