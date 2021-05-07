[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / indexer/utils

# Module: indexer/utils

## Table of contents

### Functions

- [isConfirmedBlock](indexer_utils.md#isconfirmedblock)
- [snapshotComparator](indexer_utils.md#snapshotcomparator)

## Functions

### isConfirmedBlock

▸ `Const` **isConfirmedBlock**(`blockNumber`: *number*, `onChainBlockNumber`: *number*, `maxConfirmations`: *number*): *boolean*

Compares blockNumber and onChainBlockNumber and returns `true`
if blockNumber is considered confirmed.

#### Parameters

| Name | Type |
| :------ | :------ |
| `blockNumber` | *number* |
| `onChainBlockNumber` | *number* |
| `maxConfirmations` | *number* |

**Returns:** *boolean*

boolean

Defined in: [packages/core-ethereum/src/indexer/utils.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/utils.ts#L29)

___

### snapshotComparator

▸ `Const` **snapshotComparator**(`snapA`: { `blockNumber`: *number* ; `logIndex`: *number* ; `transactionIndex`: *number*  }, `snapB`: { `blockNumber`: *number* ; `logIndex`: *number* ; `transactionIndex`: *number*  }): *number*

Compares the two snapshots provided.

#### Parameters

| Name | Type |
| :------ | :------ |
| `snapA` | *object* |
| `snapA.blockNumber` | *number* |
| `snapA.logIndex` | *number* |
| `snapA.transactionIndex` | *number* |
| `snapB` | *object* |
| `snapB.blockNumber` | *number* |
| `snapB.logIndex` | *number* |
| `snapB.transactionIndex` | *number* |

**Returns:** *number*

0 if they're equal, negative if `a` goes up, positive if `b` goes up

Defined in: [packages/core-ethereum/src/indexer/utils.ts:7](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/utils.ts#L7)
