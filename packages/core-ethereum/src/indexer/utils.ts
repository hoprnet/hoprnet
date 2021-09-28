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
  if (snapA.blockNumber < snapB.blockNumber) return -1
  else if (snapA.blockNumber > snapB.blockNumber) return 1
  else if (snapA.transactionIndex < snapB.transactionIndex) return -1
  else if (snapA.transactionIndex > snapB.transactionIndex) return 1
  else if (snapA.logIndex < snapB.logIndex) return -1
  else if (snapA.logIndex > snapB.logIndex) return 1
  else return 0
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
