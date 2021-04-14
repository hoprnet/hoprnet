import type { Wallet as IWallet, ContractTransaction } from 'ethers'
import type { Networks } from '@hoprnet/hopr-ethereum'
import BN from 'bn.js'
import { ethers, errors } from 'ethers'
import { durations, isExpired, u8aConcat } from '@hoprnet/hopr-utils'
import NonceTracker, { NonceLock } from './nonce-tracker'
import TransactionManager from './transaction-manager'
import {
  PublicKey,
  Address,
  Acknowledgement,
  Balance,
  Hash,
  NativeBalance,
  UINT256,
  UnacknowledgedTicket,
  AccountEntry
} from './types'
import { isWinningTicket, getNetworkGasPrice } from './utils'
import { PROVIDER_CACHE_TTL } from './constants'

import debug from 'debug'
const log = debug('hopr-core-ethereum:account')

export const EMPTY_HASHED_SECRET = new Hash(ethers.utils.arrayify(ethers.constants.HashZero))
const cache = new Map<'balance' | 'nativeBalance', { value: string; updatedAt: number }>()

class Account {
  private _onChainSecret?: Hash
  private _nonceTracker: NonceTracker
  private _transactions = new TransactionManager()
  private preimage: Hash

  constructor(
    private ops: {
      network: Networks
    },
    private api: {
      getLatestBlockNumber: () => Promise<number>
      getTransactionCount: (address: Address, blockNumber?: number) => Promise<number>
      getBalance: (address: Address) => Promise<Balance>
      getNativeBalance: (address: Address) => Promise<NativeBalance>
      getAccount: (address: Address) => Promise<AccountEntry>
      findPreImage: (hash: Hash) => Promise<Hash>
    },
    public wallet: IWallet
  ) {
    this._nonceTracker = new NonceTracker(
      {
        minPending: durations.minutes(15)
      },
      {
        getLatestBlockNumber: this.api.getLatestBlockNumber,
        getTransactionCount: this.api.getTransactionCount,
        getConfirmedTransactions: () => Array.from(this._transactions.confirmed.values()),
        getPendingTransactions: () => Array.from(this._transactions.pending.values())
      }
    )
  }

  public async getNonceLock(): Promise<NonceLock> {
    return this._nonceTracker.getNonceLock(this.address)
  }

  /**
   * Retrieves HOPR balance, optionally uses the cache.
   * @returns HOPR balance
   */
  public async getBalance(useCache: boolean = false): Promise<Balance> {
    return getBalance(this.api.getBalance, this.address, useCache)
  }

  /**
   * Retrieves ETH balance, optionally uses the cache.
   * @returns ETH balance
   */
  public async getNativeBalance(useCache: boolean = false): Promise<NativeBalance> {
    return getNativeBalance(this.api.getNativeBalance, this.address, useCache)
  }

  async getTicketEpoch(): Promise<UINT256> {
    const state = await this.api.getAccount(this.address)
    if (!state || !state.counter) return UINT256.fromString('0')
    return new UINT256(state.counter)
  }

  /**
   * Returns the current value of the onChainSecret
   */
  async getOnChainSecret(): Promise<Hash | undefined> {
    if (this._onChainSecret && !this._onChainSecret.eq(EMPTY_HASHED_SECRET)) return this._onChainSecret
    const state = await this.api.getAccount(this.address)
    if (!state || !state.secret) return undefined
    this.updateLocalState(state.secret)
    return state.secret
  }

  private async initPreimage() {
    if (!this.preimage) {
      const ocs = await this.getOnChainSecret()
      if (!ocs) {
        throw new Error('cannot reserve preimage when there is no on chain secret')
      }
      this.preimage = await this.api.findPreImage(ocs)
    }
  }

  /**
   * Reserve a preImage for the given ticket if it is a winning ticket.
   * @param ticket the acknowledged ticket
   */
  async acknowledge(
    unacknowledgedTicket: UnacknowledgedTicket,
    acknowledgementHash: Hash
  ): Promise<Acknowledgement | null> {
    await this.initPreimage()
    const response = Hash.create(u8aConcat(unacknowledgedTicket.secretA.serialize(), acknowledgementHash.serialize()))
    const ticket = unacknowledgedTicket.ticket
    if (await isWinningTicket(ticket.getHash(), response, this.preimage, ticket.winProb)) {
      const ack = new Acknowledgement(ticket, response, this.preimage)
      this.preimage = await this.api.findPreImage(this.preimage)
      return ack
    } else {
      return null
    }
  }

  get privateKey(): Uint8Array {
    return ethers.utils.arrayify(this.wallet.privateKey)
  }

  get publicKey(): PublicKey {
    // convert to a compressed public key
    return PublicKey.fromString(ethers.utils.computePublicKey(this.wallet.publicKey, true))
  }

  get address(): Address {
    return Address.fromString(this.wallet.address)
  }

  updateLocalState(onChainSecret: Hash) {
    this._onChainSecret = onChainSecret
  }

  public async sendTransaction<T extends (...args: any) => Promise<ContractTransaction>>(
    method: T,
    ...rest: Parameters<T>
  ): Promise<ContractTransaction> {
    const gasLimit = 300e3
    const gasPrice = getNetworkGasPrice(this.ops.network)
    const nonceLock = await this._nonceTracker.getNonceLock(this.address)
    const nonce = nonceLock.nextNonce
    let transaction: ContractTransaction

    log('Sending transaction %o', {
      gasLimit,
      gasPrice,
      nonce
    })

    try {
      // send transaction to our ethereum provider
      // TODO: better type this, make it less hacky
      transaction = await method(
        ...[
          ...rest,
          {
            gasLimit,
            gasPrice,
            nonce
          }
        ]
      )
    } catch (error) {
      log('Transaction with nonce %d failed to sent: %s', nonce, error)
      nonceLock.releaseLock()
      throw Error('Could not send transaction')
    }

    log('Transaction with nonce %d successfully sent %s, waiting for confimation', nonce, transaction.hash)
    this._transactions.addToPending(transaction.hash, { nonce })
    nonceLock.releaseLock()

    // monitor transaction, this is done asynchronously
    transaction
      .wait()
      .then(() => {
        log('Transaction with nonce %d and hash %s confirmed', nonce, transaction.hash)
        this._transactions.moveToConfirmed(transaction.hash)
      })
      .catch((error) => {
        const reverted = ([errors.CALL_EXCEPTION] as string[]).includes(error)

        if (reverted) {
          log('Transaction with nonce %d and hash %s reverted: %s', nonce, transaction.hash, error)

          // this transaction failed but was confirmed as reverted
          this._transactions.moveToConfirmed(transaction.hash)
        } else {
          log('Transaction with nonce %d failed to sent: %s', nonce, error)

          const alreadyKnown = ([errors.NONCE_EXPIRED, errors.REPLACEMENT_UNDERPRICED] as string[]).includes(error)
          // if this hash is already known and we already have it included in
          // pending we can safely ignore this
          if (alreadyKnown && this._transactions.pending.has(transaction.hash)) return

          // this transaction was not confirmed so we just remove it
          this._transactions.remove(transaction.hash)
        }
      })

    return transaction
  }
}

/**
 * Retrieves HOPR balance, optionally uses the cache.
 * TODO: use indexer to track HOPR balance
 * @returns HOPR balance
 */
export const getBalance = async (
  getBalance: (account: Address) => Promise<Balance>,
  account: Address,
  useCache: boolean = false
): Promise<Balance> => {
  if (useCache) {
    const cached = cache.get('balance')
    const notExpired = cached && !isExpired(cached.updatedAt, new Date().getTime(), PROVIDER_CACHE_TTL)
    if (notExpired) return new Balance(new BN(cached.value))
  }

  const value = await getBalance(account)
  cache.set('balance', { value: value.toBN().toString(), updatedAt: new Date().getTime() })

  return value
}

/**
 * Retrieves ETH balance, optionally uses the cache.
 * @returns ETH balance
 */
export const getNativeBalance = async (
  getNativeBalance: (account: Address) => Promise<NativeBalance>,
  account: Address,
  useCache: boolean = false
): Promise<NativeBalance> => {
  if (useCache) {
    const cached = cache.get('nativeBalance')
    const notExpired = cached && !isExpired(cached.updatedAt, new Date().getTime(), PROVIDER_CACHE_TTL)
    if (notExpired) return new NativeBalance(new BN(cached.value))
  }

  const value = await getNativeBalance(account)
  cache.set('nativeBalance', { value: value.toBN().toString(), updatedAt: new Date().getTime() })

  return value
}

export default Account
