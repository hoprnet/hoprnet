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
 * Convert the unconfirmed block number to the block number with acceptable finality
 * @param blockNumber block number of the unconfirmed block
 * @param confirmedBlockNumber block number of the known confirmed block
 * @param maxConfirmations block finality
 */
export const getConfirmedBlockNumberOrUndefined = (
  blockNumber: number,
  confirmedBlockNumber: number | undefined,
  maxConfirmations: number
): number | undefined => {
  if (blockNumber < maxConfirmations) {
    return undefined
  }

  const block = blockNumber - maxConfirmations

  if (block < confirmedBlockNumber) {
    return confirmedBlockNumber ?? 0
  } else {
    return block
  }
}
