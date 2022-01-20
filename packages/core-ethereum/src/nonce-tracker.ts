import type { Transaction as ITransaction } from './transaction-manager'
import { debug } from '@hoprnet/hopr-utils'
import assert from 'assert'
import { Mutex } from 'async-mutex'
import { Address } from '@hoprnet/hopr-utils'

const log = debug('hopr-core-ethereum:nonce-tracker')

/**
 * @property highestLocallyConfirmed - A hex string of the highest nonce on a confirmed transaction.
 * @property nextNetworkNonce - The next nonce suggested by the eth_getTransactionCount method.
 * @property highestSuggested - The maximum between the other two, the number returned.
 * @property local - Nonce details derived from pending transactions and highestSuggested
 * @property network - Nonce details from the eth_getTransactionCount method
 */
export type NonceDetails = {
  params: {
    highestLocallyConfirmed: number
    nextNetworkNonce: number
    highestSuggested: number
  }
  local: HighestContinuousFrom
  network: NetworkNextNonce
}

/**
 * @property nextNonce - The highest of the nonce values derived based on confirmed and pending transactions and eth_getTransactionCount method
 * @property nonceDetails - details of nonce value derivation.
 * @property releaseLock
 */
export interface NonceLock {
  nextNonce: number
  nonceDetails: NonceDetails
  releaseLock: () => void
}

/**
 * @property name - The name for how the nonce was calculated based on the data used
 * @property nonce - The next nonce value suggested by the eth_getTransactionCount method.
 * @property blockNumber - The latest block from the network
 * @property baseCount - Transaction count from the network suggested by eth_getTransactionCount method
 */
export type NetworkNextNonce = {
  name: string
  nonce: number
  details: {
    blockNumber: number
    baseCount: number
  }
}

/**
 * @property name - The name for how the nonce was calculated based on the data used
 * @property nonce - The next suggested nonce
 * @property details{startPoint, highest} - the provided starting nonce that was used and highest derived from it (for debugging)
 */
export type HighestContinuousFrom = {
  name: string
  nonce: number
  details: {
    startPoint: number
    highest: number
  }
}

export type Transaction = ITransaction & {
  status?: string
  from?: string
  hash?: string
}

/**
 * A simple nonce-tracker that takes into account nonces which have failed,
 * this could happen because of network issues, low funds, node unavailability, etc.
 */
export default class NonceTracker {
  private lockMap: Record<string, Mutex>

  constructor(
    private api: {
      getLatestBlockNumber: () => Promise<number>
      getTransactionCount: (address: Address, blockNumber?: number) => Promise<number>
      getPendingTransactions: (address: Address) => Transaction[]
      getConfirmedTransactions: (address: Address) => Transaction[]
    },
    private minPending?: number
  ) {
    this.lockMap = {}
  }

  /**
   * @returns Promise<{ releaseLock: () => void }> with the key releaseLock (the global mutex)
   */
  public async getGlobalLock(): Promise<{ releaseLock: () => void }> {
    const globalMutex = this._lookupMutex('global')
    // await global mutex free
    const releaseLock = await globalMutex.acquire()
    return { releaseLock }
  }

  /**
   * this will return an object with the `nextNonce` `nonceDetails`, and the releaseLock
   * Note: releaseLock must be called after adding a signed tx to pending transactions (or discarding).
   *
   * @param address the hex string for the address whose nonce we are calculating
   * @returns {Promise<NonceLock>}
   */
  public async getNonceLock(address: Address): Promise<NonceLock> {
    // await global mutex free
    await this._globalMutexFree()
    // await lock free, then take lock
    const releaseLock = await this._takeMutex(address.toHex())
    try {
      // evaluate multiple nextNonce strategies
      const networkNonceResult = await this._getNetworkNextNonce(address)
      const highestLocallyConfirmed = this._getHighestLocallyConfirmed(address)
      const nextNetworkNonce = networkNonceResult.nonce
      const highestSuggested = Math.max(nextNetworkNonce, highestLocallyConfirmed)

      const allPendingTxs = this.api.getPendingTransactions(address)
      const hasStuckTx = this._containsStuckTx(allPendingTxs)
      if (hasStuckTx) {
        log('Found stuck txs')
      }

      // if a struck tx is found, we overwrite pending txs
      const pendingTxs = hasStuckTx ? [] : allPendingTxs
      const localNonceResult = this._getHighestContinuousFrom(pendingTxs, highestSuggested)

      const nonceDetails: NonceDetails = {
        params: {
          highestLocallyConfirmed,
          nextNetworkNonce,
          highestSuggested
        },
        local: localNonceResult,
        network: networkNonceResult
      }

      const nextNonce = Math.max(networkNonceResult.nonce, localNonceResult.nonce)
      assert(
        Number.isInteger(nextNonce),
        `nonce-tracker - nextNonce is not an integer - got: (${typeof nextNonce}) "${nextNonce}"`
      )

      // return nonce and release cb
      return { nextNonce, nonceDetails, releaseLock }
    } catch (err) {
      // release lock if we encounter an error
      releaseLock()
      throw err
    }
  }

  /**
   * It's possible we encounter transactions that are pending for a very long time,
   * this can happen if a transaction is under-funded.
   * This function will return `true` if it finds a pending transaction that has
   * been pending for more than {minPending} ms.
   * @param txs
   * @return true if it contains a stuck transaction
   */
  private _containsStuckTx(txs: Transaction[]): boolean {
    if (!this.minPending) return false

    const now = new Date().getTime()

    // checks if one of the txs is stuck
    return txs.some((tx: Transaction) => {
      return tx.createdAt + this.minPending < now
    })
  }

  private async _globalMutexFree(): Promise<void> {
    const globalMutex = this._lookupMutex('global')
    const releaseLock = await globalMutex.acquire()
    releaseLock()
  }

  private async _takeMutex(lockId: string): Promise<() => void> {
    const mutex = this._lookupMutex(lockId)
    const releaseLock = await mutex.acquire()
    return releaseLock
  }

  /**
   * Function returns the nonce details from teh network based on the latest block
   * and eth_getTransactionCount method
   *
   * @param address the hex string for the address whose nonce we are calculating
   * @returns {Promise<NetworkNextNonce>}
   */
  private async _getNetworkNextNonce(address: Address): Promise<NetworkNextNonce> {
    // calculate next nonce
    // we need to make sure our base count
    // and pending count are from the same block
    const blockNumber = await this.api.getLatestBlockNumber()
    const baseCount = await this.api.getTransactionCount(address, blockNumber)
    assert(
      Number.isInteger(baseCount),
      `nonce-tracker - baseCount is not an integer - got: (${typeof baseCount}) "${baseCount}"`
    )
    const nonceDetails = { blockNumber, baseCount }
    return { name: 'network', nonce: baseCount, details: nonceDetails }
  }

  private _lookupMutex(lockId: string): Mutex {
    let mutex = this.lockMap[lockId]
    if (!mutex) {
      mutex = new Mutex()
      this.lockMap[lockId] = mutex
    }
    return mutex
  }

  /**
   * Function returns the highest of the confirmed transaction from the address.
   * @param address the hex string for the address whose nonce we are calculating
   */
  private _getHighestLocallyConfirmed(address: Address): number {
    const confirmedTransactions: Transaction[] = this.api.getConfirmedTransactions(address)
    const highest = this._getHighestNonce(confirmedTransactions)
    return Number.isInteger(highest) ? highest + 1 : 0
  }

  /**
   * Function returns highest nonce value from the transcation list provided
   * @param txList list of transactions
   */
  private _getHighestNonce(txList: Transaction[]): number {
    const nonces = txList.map((txMeta) => {
      const { nonce } = txMeta
      assert(Number.isInteger(nonce), 'nonces should be intergers')
      return nonce
    })
    const highestNonce = Math.max.apply(null, nonces)
    return highestNonce
  }

  /**
   * Function return the nonce value higher than the highest nonce value from the transaction list
   * starting from startPoint
   * @param txList {array} - list of txMeta's
   * @param startPoint {number} - the highest known locally confirmed nonce
   */
  private _getHighestContinuousFrom(txList: Transaction[], startPoint: number): HighestContinuousFrom {
    const nonces = txList.map((txMeta) => {
      const { nonce } = txMeta
      assert(Number.isInteger(nonce), 'nonces should be intergers')
      return nonce
    })

    let highest = startPoint
    while (nonces.includes(highest)) {
      highest += 1
    }

    return { name: 'local', nonce: highest, details: { startPoint, highest } }
  }
}
