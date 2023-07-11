export type IndexerSnapshot = { blockNumber: number; transactionIndex: number; logIndex: number }

/**
 * Compares the two snapshots provided.
 * @param snapA
 * @param snapB
 * @returns 0 if they're equal, negative if `a` goes up, positive if `b` goes up
 */
export function snapshotComparator(snapA: IndexerSnapshot, snapB: IndexerSnapshot): number {
  if (snapA.blockNumber != snapB.blockNumber) {
    return snapA.blockNumber - snapB.blockNumber
  } else if (snapA.transactionIndex != snapB.transactionIndex) {
    return snapA.transactionIndex - snapB.transactionIndex
  } else {
    return snapA.logIndex - snapB.logIndex
  }
}

/**
 * Compares blockNumber and onChainBlockNumber and returns `true`
 * if blockNumber is considered confirmed.
 * @returns boolean
 */
export function isConfirmedBlock(blockNumber: number, onChainBlockNumber: number, maxConfirmations: number): boolean {
  return blockNumber + maxConfirmations <= onChainBlockNumber
}
