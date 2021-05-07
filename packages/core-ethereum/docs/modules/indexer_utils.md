[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / indexer/utils

# Module: indexer/utils

## Table of contents

### Functions

- [isConfirmedBlock](indexer_utils.md#isconfirmedblock)
- [snapshotComparator](indexer_utils.md#snapshotcomparator)

## Functions

### isConfirmedBlock

▸ `Const` **isConfirmedBlock**(`blockNumber`: _number_, `onChainBlockNumber`: _number_, `maxConfirmations`: _number_): _boolean_

Compares blockNumber and onChainBlockNumber and returns `true`
if blockNumber is considered confirmed.

#### Parameters

| Name                 | Type     |
| :------------------- | :------- |
| `blockNumber`        | _number_ |
| `onChainBlockNumber` | _number_ |
| `maxConfirmations`   | _number_ |

**Returns:** _boolean_

boolean

Defined in: [packages/core-ethereum/src/indexer/utils.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/utils.ts#L29)

---

### snapshotComparator

▸ `Const` **snapshotComparator**(`snapA`: { `blockNumber`: _number_ ; `logIndex`: _number_ ; `transactionIndex`: _number_ }, `snapB`: { `blockNumber`: _number_ ; `logIndex`: _number_ ; `transactionIndex`: _number_ }): _number_

Compares the two snapshots provided.

#### Parameters

| Name                     | Type     |
| :----------------------- | :------- |
| `snapA`                  | _object_ |
| `snapA.blockNumber`      | _number_ |
| `snapA.logIndex`         | _number_ |
| `snapA.transactionIndex` | _number_ |
| `snapB`                  | _object_ |
| `snapB.blockNumber`      | _number_ |
| `snapB.logIndex`         | _number_ |
| `snapB.transactionIndex` | _number_ |

**Returns:** _number_

0 if they're equal, negative if `a` goes up, positive if `b` goes up

Defined in: [packages/core-ethereum/src/indexer/utils.ts:7](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/utils.ts#L7)
