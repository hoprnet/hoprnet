import Debug from 'debug'
import { BigNumber } from 'ethers'
import { isDeepStrictEqual } from 'util'
const log = Debug('hopr-core-ethereum:transcation-manager')

export type TransactionPayload = {
  to: string
  data: string
  value: BigNumber
}
export type Transaction = {
  nonce: number
  createdAt: number
  gasPrice: number | BigNumber
}

/**
 * Keep track of pending and confirmed transactions,
 * and allows for pruning unnecessary data.
 * This class is mainly used by nonce-tracker which relies
 * on transcation-manager to keep an update to date view
 * on transactions.
 */
class TranscationManager {
  /**
   * transaction payloads
   */
  public readonly payloads = new Map<string, TransactionPayload>()
  /**
   * pending transactions
   */
  public readonly pending = new Map<string, Transaction>()
  /**
   * mined transactions
   */
  public readonly mined = new Map<string, Transaction>()
  /**
   * confirmed transactions
   */
  public readonly confirmed = new Map<string, Transaction>()

  /**
   * If a transaction payload exists in mined or pending with a higher/equal gas price
   * @param payload object
   * @param gasPrice gas price associated with the payload
   */
  public existInMinedOrPendingWithHigherFee(payload: TransactionPayload, gasPrice: number | BigNumber): Boolean {
    // Using isDeepStrictEqual to compare TransactionPayload objects, see
    // https://nodejs.org/api/util.html#util_util_isdeepstrictequal_val1_val2
    if (Array.from(this.payloads.values()).findIndex((pl) => isDeepStrictEqual(pl, payload)) >= 0) {
      return false
    }
    const hash = [...this.payloads].find(([_, val]) => val == payload)[0]
    if (!this.mined.get(hash) && BigNumber.from(this.pending.get(hash).gasPrice).lt(BigNumber.from(gasPrice))) {
      return false
    }
    return true
  }

  /**
   * Adds transaction in pending
   * @param hash transaction hash
   * @param transaction object
   */
  public addToPending(
    hash: string,
    transaction: Omit<Transaction, 'createdAt'>,
    transactionPayload: TransactionPayload
  ): void {
    if (this.pending.has(hash)) return

    log('Adding pending transaction %s %i', hash, transaction.nonce)
    this.payloads.set(hash, transactionPayload)
    this.pending.set(hash, { nonce: transaction.nonce, createdAt: this._getTime(), gasPrice: transaction.gasPrice })
  }

  /**
   * Moves transcation from pending to mined
   * @param hash transaction hash
   */
  public moveToMined(hash: string): void {
    if (!this.pending.has(hash)) return

    log('Moving transaction to confirmed %s', hash)
    this.mined.set(hash, this.pending.get(hash))
    this.pending.delete(hash)
  }

  /**
   * Moves transcation from pending to confirmed. Delete payload
   * @param hash transaction hash
   */
  public moveToConfirmed(hash: string): void {
    if (!this.mined.has(hash)) return

    log('Moving transaction to confirmed %s', hash)
    this.confirmed.set(hash, this.pending.get(hash))
    this.mined.delete(hash)
    this.payloads.delete(hash)
  }

  /**
   * Removed transcation from pending, mined and confirmed
   * @param hash transaction hash
   */
  public remove(hash: string): void {
    log('Removing transaction %s', hash)
    this.payloads.delete(hash)
    this.pending.delete(hash)
    this.mined.delete(hash)
    this.confirmed.delete(hash)
  }

  /**
   * Removes confirmed blocks except last 5 nonces.
   * This is a way for us to clean up some memory which we know
   * we don't need anymore.
   */
  public prune(): void {
    const descTxs = Array.from(this.confirmed.entries()).sort(([, a], [, b]) => {
      return b.nonce - a.nonce
    })

    for (const [hash] of descTxs.slice(5, descTxs.length)) {
      this.remove(hash)
    }
  }

  /**
   * @returns current timestamp
   */
  private _getTime(): number {
    return new Date().getTime()
  }
}

export default TranscationManager
