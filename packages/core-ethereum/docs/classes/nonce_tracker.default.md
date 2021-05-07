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

\+ **new default**(`api`: { `getConfirmedTransactions`: (`address`: *Address*) => [*Transaction*](../modules/nonce_tracker.md#transaction)[] ; `getLatestBlockNumber`: () => *Promise*<number\> ; `getPendingTransactions`: (`address`: *Address*) => [*Transaction*](../modules/nonce_tracker.md#transaction)[] ; `getTransactionCount`: (`address`: *Address*, `blockNumber?`: *number*) => *Promise*<number\>  }, `minPending?`: *number*): [*default*](nonce_tracker.default.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `api` | *object* |
| `api.getConfirmedTransactions` | (`address`: *Address*) => [*Transaction*](../modules/nonce_tracker.md#transaction)[] |
| `api.getLatestBlockNumber` | () => *Promise*<number\> |
| `api.getPendingTransactions` | (`address`: *Address*) => [*Transaction*](../modules/nonce_tracker.md#transaction)[] |
| `api.getTransactionCount` | (`address`: *Address*, `blockNumber?`: *number*) => *Promise*<number\> |
| `minPending?` | *number* |

**Returns:** [*default*](nonce_tracker.default.md)

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:77](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L77)

## Properties

### lockMap

• `Private` **lockMap**: *Record*<string, Mutex\>

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:77](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L77)

## Methods

### \_containsStuckTx

▸ `Private` **_containsStuckTx**(`txs`: [*Transaction*](../modules/nonce_tracker.md#transaction)[]): *boolean*

It's possible we encounter transactions that are pending for a very long time,
this can happen if a transaction is under-funded.
This function will return `true` if it finds a pending transaction that has
been pending for more than {minPending} ms.

#### Parameters

| Name | Type |
| :------ | :------ |
| `txs` | [*Transaction*](../modules/nonce_tracker.md#transaction)[] |

**Returns:** *boolean*

true if it contains a stuck transaction

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:163](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L163)

___

### \_getHighestContinuousFrom

▸ `Private` **_getHighestContinuousFrom**(`txList`: [*Transaction*](../modules/nonce_tracker.md#transaction)[], `startPoint`: *number*): [*HighestContinuousFrom*](../modules/nonce_tracker.md#highestcontinuousfrom)

Function return the nonce value higher than the highest nonce value from the transaction list
starting from startPoint

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `txList` | [*Transaction*](../modules/nonce_tracker.md#transaction)[] | list of txMeta's |
| `startPoint` | *number* | the highest known locally confirmed nonce |

**Returns:** [*HighestContinuousFrom*](../modules/nonce_tracker.md#highestcontinuousfrom)

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:246](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L246)

___

### \_getHighestLocallyConfirmed

▸ `Private` **_getHighestLocallyConfirmed**(`address`: *Address*): *number*

Function returns the highest of the confirmed transaction from the address.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `address` | *Address* | the hex string for the address whose nonce we are calculating |

**Returns:** *number*

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:220](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L220)

___

### \_getHighestNonce

▸ `Private` **_getHighestNonce**(`txList`: [*Transaction*](../modules/nonce_tracker.md#transaction)[]): *number*

Function returns highest nonce value from the transcation list provided

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `txList` | [*Transaction*](../modules/nonce_tracker.md#transaction)[] | list of transactions |

**Returns:** *number*

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:230](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L230)

___

### \_getNetworkNextNonce

▸ `Private` **_getNetworkNextNonce**(`address`: *Address*): *Promise*<[*NetworkNextNonce*](../modules/nonce_tracker.md#networknextnonce)\>

Function returns the nonce details from teh network based on the latest block
and eth_getTransactionCount method

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `address` | *Address* | the hex string for the address whose nonce we are calculating |

**Returns:** *Promise*<[*NetworkNextNonce*](../modules/nonce_tracker.md#networknextnonce)\>

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:193](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L193)

___

### \_globalMutexFree

▸ `Private` **_globalMutexFree**(): *Promise*<void\>

**Returns:** *Promise*<void\>

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:174](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L174)

___

### \_lookupMutex

▸ `Private` **_lookupMutex**(`lockId`: *string*): *Mutex*

#### Parameters

| Name | Type |
| :------ | :------ |
| `lockId` | *string* |

**Returns:** *Mutex*

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:207](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L207)

___

### \_takeMutex

▸ `Private` **_takeMutex**(`lockId`: *string*): *Promise*<() => *void*\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `lockId` | *string* |

**Returns:** *Promise*<() => *void*\>

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:180](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L180)

___

### getGlobalLock

▸ **getGlobalLock**(): *Promise*<{ `releaseLock`: () => *void*  }\>

**Returns:** *Promise*<{ `releaseLock`: () => *void*  }\>

Promise<{ releaseLock: () => void }> with the key releaseLock (the global mutex)

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:94](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L94)

___

### getNonceLock

▸ **getNonceLock**(`address`: *Address*): *Promise*<[*NonceLock*](../interfaces/nonce_tracker.noncelock.md)\>

this will return an object with the `nextNonce` `nonceDetails`, and the releaseLock
Note: releaseLock must be called after adding a signed tx to pending transactions (or discarding).

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `address` | *Address* | the hex string for the address whose nonce we are calculating |

**Returns:** *Promise*<[*NonceLock*](../interfaces/nonce_tracker.noncelock.md)\>

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:108](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L108)
