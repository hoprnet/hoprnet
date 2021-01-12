import { Log } from './utils'

const log = Log(['transcation-manager'])

export type Transaction = {
  nonce: number
  createdAt: number
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
   * pending transactions
   */
  public readonly pending = new Map<string, Transaction>()
  /**
   * confirmed transactions
   */
  public readonly confirmed = new Map<string, Transaction>()

  /**
   * Adds transaction in pending
   * @param hash transaction hash
   * @param transaction object
   */
  public addToPending(hash: string, transaction: Pick<Transaction, 'nonce'>): void {
    if (this.pending.has(hash)) return

    log('Adding pending transaction %s %i', hash, transaction.nonce)
    this.pending.set(hash, { nonce: transaction.nonce, createdAt: this._getTime() })
  }

  /**
   * Moves transcation from pending to confirmed
   * @param hash transaction hash
   */
  public moveToConfirmed(hash: string): void {
    if (!this.pending.has(hash)) return

    log('Moving transaction to confirmed %s', hash)
    this.confirmed.set(hash, this.pending.get(hash))
    this.pending.delete(hash)
  }

  /**
   * Removed transcation from pending and confirmed
   * @param hash transaction hash
   */
  public remove(hash: string): void {
    log('Removing transaction %s', hash)
    this.pending.delete(hash)
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
