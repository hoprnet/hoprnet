import { providers as IProviders, Wallet as IWallet, ContractTransaction } from 'ethers'
import type HoprEthereum from '.'
import type { HoprToken } from './contracts'
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
  UnacknowledgedTicket
} from './types'
import { isWinningTicket, isGanache, getNetworkGasPrice } from './utils'
import { PROVIDER_CACHE_TTL } from './constants'
import * as ethereum from './ethereum'
import BN from 'bn.js'

import debug from 'debug'
const log = debug('hopr-core-ethereum:account')

// omits the last element in the list
// type OmitLastElement<T extends any[]> = T extends [...infer I, infer _L] ? I : never

export const EMPTY_HASHED_SECRET = new Hash(new Uint8Array(Hash.SIZE).fill(0x00))
const cache = new Map<'balance' | 'nativeBalance', { value: string; updatedAt: number }>()

class Account {
  private _onChainSecret?: Hash
  private _nonceTracker: NonceTracker
  private _transactions = new TransactionManager()
  private preimage: Hash

  constructor(public coreConnector: HoprEthereum, public wallet: IWallet) {
    this._nonceTracker = new NonceTracker({
      getLatestBlockNumber: async () => {
        // when running our unit/intergration tests using ganache,
        // the indexer doesn't have enough time to pick up the events and reduce the data
        return isGanache(coreConnector.network)
          ? coreConnector.provider.getBlockNumber()
          : coreConnector.indexer.latestBlock
      },
      getTransactionCount: async (address: string, blockNumber?: number) =>
        coreConnector.provider.getTransactionCount(address, blockNumber),
      getConfirmedTransactions: () => Array.from(this._transactions.confirmed.values()),
      getPendingTransactions: () => Array.from(this._transactions.pending.values()),
      minPending: durations.minutes(15)
    })
  }

  public async getNonceLock(): Promise<NonceLock> {
    return this._nonceTracker.getNonceLock(this.address.toHex())
  }

  /**
   * Retrieves HOPR balance, optionally uses the cache.
   * @returns HOPR balance
   */
  public async getBalance(useCache: boolean = false): Promise<Balance> {
    return getBalance(this.coreConnector.hoprToken, this.address, useCache)
  }

  /**
   * Retrieves ETH balance, optionally uses the cache.
   * @returns ETH balance
   */
  public async getNativeBalance(useCache: boolean = false): Promise<NativeBalance> {
    return getNativeBalance(this.coreConnector.provider, this.address, useCache)
  }

  async getTicketEpoch(): Promise<UINT256> {
    const state = await this.coreConnector.indexer.getAccount(this.address)
    if (!state || !state.counter) return UINT256.fromString('0')
    return new UINT256(state.counter)
  }

  /**
   * Returns the current value of the onChainSecret
   */
  async getOnChainSecret(): Promise<Hash | undefined> {
    if (this._onChainSecret && !this._onChainSecret.eq(EMPTY_HASHED_SECRET)) return this._onChainSecret
    const state = await this.coreConnector.indexer.getAccount(this.address)
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
      this.preimage = await this.coreConnector.hashedSecret.findPreImage(ocs)
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
      this.preimage = await this.coreConnector.hashedSecret.findPreImage(this.preimage)
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

  private handleTransactionError(hash: string, nonce: number, releaseLock: () => void, error: string) {
    releaseLock()
    const reverted = ([errors.CALL_EXCEPTION] as string[]).includes(error)

    if (reverted) {
      log('Transaction with nonce %d and hash %s reverted: %s', nonce, hash, error)

      // this transaction failed but was confirmed as reverted
      this._transactions.moveToConfirmed(hash)
    } else {
      log('Transaction with nonce %d failed to sent: %s', nonce, error)

      const alreadyKnown = ([errors.NONCE_EXPIRED, errors.REPLACEMENT_UNDERPRICED] as string[]).includes(error)
      // if this hash is already known and we already have it included in
      // pending we can safely ignore this
      if (alreadyKnown && this._transactions.pending.has(hash)) return

      // this transaction was not confirmed so we just remove it
      this._transactions.remove(hash)
    }
  }

  public async sendTransaction<T extends (...args: any) => Promise<ContractTransaction>>(
    method: T,
    ...rest: Parameters<T>
  ): Promise<ContractTransaction> {
    const gasLimit = 300e3
    const gasPrice = getNetworkGasPrice(this.coreConnector.network)
    const nonceLock = await this._nonceTracker.getNonceLock(this.address.toHex())
    const nonce = nonceLock.nextNonce
    let transaction: ContractTransaction

    try {
      log('Sending transaction %o', {
        gasLimit,
        gasPrice,
        nonce
      })
      // send transaction to our ethereum provider
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
          this.handleTransactionError(transaction.hash, nonce, nonceLock.releaseLock, error.message)
        })
    } catch (error) {
      this.handleTransactionError(transaction.hash, nonce, nonceLock.releaseLock, error.message)
    }

    if (!transaction) throw Error('Could not send transaction')
    return transaction
  }
}

/**
 * Retrieves HOPR balance, optionally uses the cache.
 * TODO: use indexer to track HOPR balance
 * @returns HOPR balance
 */
export const getBalance = async (
  hoprToken: HoprToken,
  account: Address,
  useCache: boolean = false
): Promise<Balance> => {
  if (useCache) {
    const cached = cache.get('balance')
    const notExpired = cached && !isExpired(cached.updatedAt, new Date().getTime(), PROVIDER_CACHE_TTL)
    if (notExpired) return new Balance(new BN(cached.value))
  }

  const value = await ethereum.getBalance(hoprToken, account)
  cache.set('balance', { value: value.toBN().toString(), updatedAt: new Date().getTime() })

  return value
}

/**
 * Retrieves ETH balance, optionally uses the cache.
 * @returns ETH balance
 */
export const getNativeBalance = async (
  provider: IProviders.WebSocketProvider,
  account: Address,
  useCache: boolean = false
): Promise<NativeBalance> => {
  if (useCache) {
    const cached = cache.get('nativeBalance')
    const notExpired = cached && !isExpired(cached.updatedAt, new Date().getTime(), PROVIDER_CACHE_TTL)
    if (notExpired) return new NativeBalance(new BN(cached.value))
  }

  const value = await ethereum.getNativeBalance(provider, account)
  cache.set('nativeBalance', { value: value.toBN().toString(), updatedAt: new Date().getTime() })

  return value
}

export default Account
