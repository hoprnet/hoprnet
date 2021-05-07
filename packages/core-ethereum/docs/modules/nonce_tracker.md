[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / nonce-tracker

# Module: nonce-tracker

## Table of contents

### Classes

- [default](../classes/nonce_tracker.default.md)

### Interfaces

- [NonceLock](../interfaces/nonce_tracker.noncelock.md)

### Type aliases

- [HighestContinuousFrom](nonce_tracker.md#highestcontinuousfrom)
- [NetworkNextNonce](nonce_tracker.md#networknextnonce)
- [NonceDetails](nonce_tracker.md#noncedetails)
- [Transaction](nonce_tracker.md#transaction)

## Type aliases

### HighestContinuousFrom

Ƭ **HighestContinuousFrom**: *object*

**`property`** name - The name for how the nonce was calculated based on the data used

**`property`** nonce - The next suggested nonce

**`property`** details{startPoint, highest} - the provided starting nonce that was used and highest derived from it (for debugging)

#### Type declaration

| Name | Type |
| :------ | :------ |
| `details` | *object* |
| `details.highest` | *number* |
| `details.startPoint` | *number* |
| `name` | *string* |
| `nonce` | *number* |

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:57](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L57)

___

### NetworkNextNonce

Ƭ **NetworkNextNonce**: *object*

**`property`** name - The name for how the nonce was calculated based on the data used

**`property`** nonce - The next nonce value suggested by the eth_getTransactionCount method.

**`property`** blockNumber - The latest block from the network

**`property`** baseCount - Transaction count from the network suggested by eth_getTransactionCount method

#### Type declaration

| Name | Type |
| :------ | :------ |
| `details` | *object* |
| `details.baseCount` | *number* |
| `details.blockNumber` | *number* |
| `name` | *string* |
| `nonce` | *number* |

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:43](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L43)

___

### NonceDetails

Ƭ **NonceDetails**: *object*

**`property`** highestLocallyConfirmed - A hex string of the highest nonce on a confirmed transaction.

**`property`** nextNetworkNonce - The next nonce suggested by the eth_getTransactionCount method.

**`property`** highestSuggested - The maximum between the other two, the number returned.

**`property`** local - Nonce details derived from pending transactions and highestSuggested

**`property`** network - Nonce details from the eth_getTransactionCount method

#### Type declaration

| Name | Type |
| :------ | :------ |
| `local` | [*HighestContinuousFrom*](nonce_tracker.md#highestcontinuousfrom) |
| `network` | [*NetworkNextNonce*](nonce_tracker.md#networknextnonce) |
| `params` | *object* |
| `params.highestLocallyConfirmed` | *number* |
| `params.highestSuggested` | *number* |
| `params.nextNetworkNonce` | *number* |

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:16](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L16)

___

### Transaction

Ƭ **Transaction**: [*Transaction*](transaction_manager.md#transaction) & { `from?`: *string* ; `hash?`: *string* ; `status?`: *string*  }

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:66](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L66)
