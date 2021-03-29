/**
 * Compares the two snapshots provided.
 * @param snapA
 * @param snapB
 * @returns 0 if they're equal, negative if `a` goes up, positive if `b` goes up
 */
export const snapshotComparator = (
  snapA: { blockNumber: number; transactionIndex: number; logIndex: number },
  snapB: {
    blockNumber: number
    transactionIndex: number
    logIndex: number
  }
): number => {
  if (snapA.blockNumber !== snapB.blockNumber) {
    return snapA.blockNumber - snapB.blockNumber
  } else if (snapA.transactionIndex !== snapB.transactionIndex) {
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
export const isConfirmedBlock = (
  blockNumber: number,
  onChainBlockNumber: number,
  maxConfirmations: number
): boolean => {
  return blockNumber + maxConfirmations <= onChainBlockNumber
}

/**
 * We consider the indexer up to date if it's only lacking by 4 blocks behind.
 * @param onChainBlock
 * @param lastKnownBlock
 * @returns returns true if it's syncing
 */
export const isSyncing = (onChainBlock: number, lastKnownBlock: number): boolean => {
  return lastKnownBlock + 4 >= onChainBlock
}
