import assert from 'assert'
import { Mutex } from 'async-mutex'

/**
 *  @property opts.web3 - An ethereum provider
 *  @property opts.blockTracker - An instance of eth-block-tracker
 *  @property opts.getPendingTransactions - A function that returns an array of txMeta
 *  whose status is `submitted`
 *  @property opts.getConfirmedTransactions - A function that returns an array of txMeta
 *  whose status is `confirmed`
 */
export interface NonceTrackerOptions {
  getLatestBlockNumber: () => Promise<number>
  getTransactionCount: (address: string, blockNumber?: number) => Promise<number>
  getPendingTransactions: (address: string) => Transaction[]
  getConfirmedTransactions: (address: string) => Transaction[]
}

/**
 * @property highestLocallyConfirmed - A hex string of the highest nonce on a confirmed transaction.
 * @property nextNetworkNonce - The next nonce suggested by the eth_getTransactionCount method.
 * @property highestSuggested - The maximum between the other two, the number returned.
 * @property local - Nonce details derived from pending transactions and highestSuggested
 * @property network - Nonce details from the eth_getTransactionCount method
 */
export interface NonceDetails {
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
export interface NetworkNextNonce {
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
export interface HighestContinuousFrom {
  name: string
  nonce: number
  details: {
    startPoint: number
    highest: number
  }
}

export interface Transaction {
  status?: string
  from?: string
  hash?: string
  nonce: number
}

/**
 * A simple nonce-tracker that takes into account nonces which have failed,
 * this could happen because of network issues, low funds, node unavailability, etc.
 */
export default class NonceTracker {
  private getLatestBlockNumber: NonceTrackerOptions['getLatestBlockNumber']
  private getTransactionCount: NonceTrackerOptions['getTransactionCount']
  private getPendingTransactions: NonceTrackerOptions['getPendingTransactions']
  private getConfirmedTransactions: NonceTrackerOptions['getConfirmedTransactions']
  private lockMap: Record<string, Mutex>

  constructor(opts: NonceTrackerOptions) {
    this.getLatestBlockNumber = opts.getLatestBlockNumber
    this.getTransactionCount = opts.getTransactionCount
    this.getPendingTransactions = opts.getPendingTransactions
    this.getConfirmedTransactions = opts.getConfirmedTransactions
    this.lockMap = {}
  }

  /**
   * @returns Promise<{ releaseLock: () => void }> with the key releaseLock (the global mutex)
   */
  async getGlobalLock(): Promise<{ releaseLock: () => void }> {
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
  async getNonceLock(address: string): Promise<NonceLock> {
    // await global mutex free
    await this._globalMutexFree()
    // await lock free, then take lock
    const releaseLock = await this._takeMutex(address)
    try {
      // evaluate multiple nextNonce strategies
      const networkNonceResult = await this._getNetworkNextNonce(address)
      const highestLocallyConfirmed = this._getHighestLocallyConfirmed(address)
      const nextNetworkNonce = networkNonceResult.nonce
      const highestSuggested = Math.max(nextNetworkNonce, highestLocallyConfirmed)

      const pendingTxs: Transaction[] = this.getPendingTransactions(address)
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

  async _globalMutexFree(): Promise<void> {
    const globalMutex = this._lookupMutex('global')
    const releaseLock = await globalMutex.acquire()
    releaseLock()
  }

  async _takeMutex(lockId: string): Promise<() => void> {
    const mutex = this._lookupMutex(lockId)
    const releaseLock = await mutex.acquire()
    return releaseLock
  }

  _lookupMutex(lockId: string): Mutex {
    let mutex = this.lockMap[lockId]
    if (!mutex) {
      mutex = new Mutex()
      this.lockMap[lockId] = mutex
    }
    return mutex
  }

  /**
   * Function returns the nonce details from teh network based on the latest block
   * and eth_getTransactionCount method
   *
   * @param address the hex string for the address whose nonce we are calculating
   * @returns {Promise<NetworkNextNonce>}
   */
  async _getNetworkNextNonce(address: string): Promise<NetworkNextNonce> {
    // calculate next nonce
    // we need to make sure our base count
    // and pending count are from the same block
    // const blockNumber: string = await this.blockTracker.getLatestBlock()
    const blockNumber = await this.getLatestBlockNumber()
    // const baseCountBN = await this.web3.eth.getTransactionCount(address, blockNumber)
    const baseCount = await this.getTransactionCount(address, blockNumber)
    assert(
      Number.isInteger(baseCount),
      `nonce-tracker - baseCount is not an integer - got: (${typeof baseCount}) "${baseCount}"`
    )
    const nonceDetails = { blockNumber, baseCount }
    return { name: 'network', nonce: baseCount, details: nonceDetails }
  }

  /**
   * Function returns the highest of the confirmed transaction from the address.
   * @param address the hex string for the address whose nonce we are calculating
   */
  _getHighestLocallyConfirmed(address: string): number {
    const confirmedTransactions: Transaction[] = this.getConfirmedTransactions(address)
    const highest = this._getHighestNonce(confirmedTransactions)
    return Number.isInteger(highest) ? highest + 1 : 0
  }

  /**
   * Function returns highest nonce value from the transcation list provided
   * @param txList list of transactions
   */
  _getHighestNonce(txList: Transaction[]): number {
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
  _getHighestContinuousFrom(txList: Transaction[], startPoint: number): HighestContinuousFrom {
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
