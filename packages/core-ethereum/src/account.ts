import type HoprEthereum from '.'
import type { TransactionObject } from './tsc/web3/types'
import type { TransactionConfig } from 'web3-core'
import { getRpcOptions } from '@hoprnet/hopr-ethereum'
import { Intermediate, stringToU8a, u8aEquals, u8aToHex } from '@hoprnet/hopr-utils'
import NonceTracker, { Transaction } from './nonce-tracker'
import { AccountId, AcknowledgedTicket, Balance, Hash, NativeBalance, TicketEpoch } from './types'
import { isWinningTicket, pubKeyToAccountId } from './utils'
import { ContractEventEmitter } from './tsc/web3/types'
import { HASHED_SECRET_WIDTH } from './hashedSecret'

import debug from 'debug'
const log = debug('hopr-core-ethereum:account')

export const EMPTY_HASHED_SECRET = new Uint8Array(HASHED_SECRET_WIDTH).fill(0x00)
const rpcOps = getRpcOptions()

class Account {
  private _address?: AccountId
  private _preImageIterator: AsyncGenerator<boolean, boolean, AcknowledgedTicket>
  private _ticketEpoch?: TicketEpoch
  private _ticketEpochListener?: ContractEventEmitter<any>
  private _onChainSecret?: Hash
  private _nonceTracker: NonceTracker
  private _confirmed_transactions = new Map<string, Transaction>()
  private _pending_transactions = new Map<string, Transaction>()

  /**
   * The accounts keys:
   */
  public keys: {
    onChain: {
      privKey: Uint8Array
      pubKey: Uint8Array
    }
    offChain: {
      privKey: Uint8Array
      pubKey: Uint8Array
    }
  }

  constructor(public coreConnector: HoprEthereum, privKey: Uint8Array, pubKey: Uint8Array) {
    this.keys = {
      onChain: {
        privKey,
        pubKey
      },
      offChain: {
        privKey,
        pubKey
      }
    }

    this._nonceTracker = new NonceTracker({
      getLatestBlockNumber: () => coreConnector.web3.eth.getBlockNumber(),
      getTransactionCount: async (address: string, blockNumber?: number) =>
        coreConnector.web3.eth.getTransactionCount(address, blockNumber),
      getConfirmedTransactions: () => Array.from(this._confirmed_transactions.values()),
      getPendingTransactions: () => Array.from(this._pending_transactions.values())
    })

    this._preImageIterator = async function* (this: Account) {
      let ticket: AcknowledgedTicket = yield

      let currentPreImage: Promise<Intermediate> = this.coreConnector.hashedSecret.findPreImage(
        await this.onChainSecret
      )

      let tmp: Intermediate = await currentPreImage

      while (true) {
        if (
          await isWinningTicket(
            await (await ticket.signedTicket).ticket.hash,
            ticket.response,
            new Hash(tmp.preImage),
            (await ticket.signedTicket).ticket.winProb
          )
        ) {
          currentPreImage = this.coreConnector.hashedSecret.findPreImage(tmp.preImage)

          ticket.preImage = new Hash(tmp.preImage)

          if (tmp.iteration == 0) {
            // @TODO dispatch call of next hashedSecret submit
            return true
          } else {
            yield true
          }

          tmp = await currentPreImage
        } else {
          yield false
        }

        ticket = yield
      }
    }.call(this)
  }

  async stop() {
    if (this._ticketEpochListener) {
      this._ticketEpochListener.removeAllListeners()
    }
  }

  get nonce(): Promise<number> {
    return this._nonceTracker
      .getNonceLock(this._address.toHex())
      .then((res) => res.nonceDetails.params.highestSuggested)
  }

  /**
   * Returns the current balances of the account associated with this node (HOPR)
   * @returns a promise resolved to Balance
   */
  get balance(): Promise<Balance> {
    return new Promise<Balance>(async (resolve, reject) => {
      try {
        resolve(new Balance(await this.coreConnector.hoprToken.methods.balanceOf((await this.address).toHex()).call()))
      } catch (err) {
        reject(err)
      }
    })
  }

  /**
   * Returns the current native balance (ETH)
   * @returns a promise resolved to Balance
   */
  get nativeBalance(): Promise<NativeBalance> {
    return new Promise(async (resolve, reject) => {
      try {
        resolve(new NativeBalance(await this.coreConnector.web3.eth.getBalance((await this.address).toHex())))
      } catch (err) {
        reject(err)
      }
    })
  }

  get ticketEpoch(): Promise<TicketEpoch> {
    if (this._ticketEpoch != null) {
      return Promise.resolve(this._ticketEpoch)
    }

    this.attachAccountDataListener()

    return this.address.then((address) => {
      return this.coreConnector.hoprChannels.methods
        .accounts(address.toHex())
        .call()
        .then((res) => {
          this._ticketEpoch = new TicketEpoch(res.counter)

          return this._ticketEpoch
        })
    })
  }

  /**
   * Returns the current value of the onChainSecret
   */
  get onChainSecret(): Promise<Hash> {
    if (this._onChainSecret != null) {
      return Promise.resolve(this._onChainSecret)
    }

    this.attachAccountDataListener()

    return this.address.then((address) => {
      return this.coreConnector.hoprChannels.methods
        .accounts(address.toHex())
        .call()
        .then((res) => {
          const hashedSecret = stringToU8a(res.hashedSecret)

          // true if this string is an empty bytes32
          if (u8aEquals(hashedSecret, EMPTY_HASHED_SECRET)) {
            return undefined
          }

          this._onChainSecret = new Hash(hashedSecret)

          return this._onChainSecret
        })
    })
  }

  /**
   * Reserve a preImage for the given ticket if it is a winning ticket.
   * @param ticket the acknowledged ticket
   */
  async reservePreImageIfIsWinning(ticket: AcknowledgedTicket) {
    await this._preImageIterator.next()

    return (await this._preImageIterator.next(ticket)).value
  }

  get address(): Promise<AccountId> {
    if (this._address) {
      return Promise.resolve(this._address)
    }

    return pubKeyToAccountId(this.keys.onChain.pubKey).then((accountId: AccountId) => {
      this._address = accountId
      return this._address
    })
  }

  updateLocalState(onChainSecret: Hash) {
    this._onChainSecret = onChainSecret
  }

  // @TODO: switch to web3js-accounts
  public async signTransaction<T>(
    // config put in .send
    txConfig: Omit<TransactionConfig, 'nonce'>,
    // return of our contract method in web3.Contract instance
    txObject?: TransactionObject<T>
  ) {
    const { web3, network } = this.coreConnector

    const abi = txObject ? txObject.encodeABI() : undefined
    const gas = 200e3

    // set gasPrice
    let gasPrice: number = 1e9
    // specified in network settings
    if (rpcOps[network]?.gasPrice) gasPrice = rpcOps[network]?.gasPrice
    // let's web3 pick gas price
    if (network === 'mainnet') return undefined

    // @TODO: potential deadlock, needs to be improved
    const nonceLock = await this._nonceTracker.getNonceLock(this._address.toHex())

    // @TODO: provide some of the values to avoid multiple calls
    const options = {
      gas,
      gasPrice,
      ...txConfig,
      nonce: nonceLock.nextNonce,
      data: abi
    }

    const signedTransaction = await web3.eth.accounts.signTransaction(options, u8aToHex(this.keys.onChain.privKey))

    const send = () => {
      if (signedTransaction.rawTransaction == null) {
        throw Error('Cannot process transaction because Web3.js did not give us the raw transaction.')
      }

      log('Sending transaction %o', {
        gas: options.gas,
        gasPrice: options.gasPrice,
        nonce: options.nonce,
        hash: signedTransaction.transactionHash
      })

      const event = web3.eth.sendSignedTransaction(signedTransaction.rawTransaction)
      this._pending_transactions.set(signedTransaction.transactionHash, {
        hash: signedTransaction.transactionHash,
        nonce: options.nonce
      })
      log('Added pending transaction %s %i', signedTransaction.transactionHash, options.nonce)
      nonceLock.releaseLock()

      // @TODO: cleanup old txs
      event.once('receipt', () => {
        log('Moving transaction to confirmed %s %i', signedTransaction.transactionHash, options.nonce)
        this._pending_transactions.delete(signedTransaction.transactionHash)
        this._confirmed_transactions.set(signedTransaction.transactionHash, {
          hash: signedTransaction.transactionHash,
          nonce: options.nonce
        })
      })
      event.once('error', (error) => {
        log(
          'Removing failed transaction %s %i with error %s',
          signedTransaction.transactionHash,
          options.nonce,
          error.message
        )
        this._pending_transactions.delete(signedTransaction.transactionHash)
        this._confirmed_transactions.delete(signedTransaction.transactionHash)
      })

      return event
    }

    return {
      send,
      transactionHash: signedTransaction.transactionHash
    }
  }

  private async attachAccountDataListener() {
    if (this._ticketEpochListener == null) {
      // listen for 'SecretHashSet' events and update 'ticketEpoch'
      // on error, safely reset 'ticketEpoch' & event listener
      try {
        this._ticketEpochListener = this.coreConnector.hoprChannels.events
          .SecretHashSet({
            fromBlock: 'latest',
            filter: {
              account: (await this.address).toHex()
            }
          })
          .on('data', (event) => {
            log('new ticketEpoch', event.returnValues.counter)

            this._ticketEpoch = new TicketEpoch(event.returnValues.counter)
            this._onChainSecret = new Hash(stringToU8a(event.returnValues.secretHash), Hash.SIZE)
          })
          .on('error', (error) => {
            log('error listening to SecretHashSet events', error.message)

            this._ticketEpochListener?.removeAllListeners()
            this._ticketEpoch = undefined
          })
      } catch (err) {
        log(err)
        this._ticketEpochListener?.removeAllListeners()
      }
    }
  }
}

export default Account
