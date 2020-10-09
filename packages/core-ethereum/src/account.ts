import type HoprEthereum from '.'
import { stringToU8a, u8aEquals } from '@hoprnet/hopr-utils'
import { AccountId, AcknowledgedTicket, Balance, Hash, NativeBalance, TicketEpoch } from './types'
import { isWinningTicket, pubKeyToAccountId } from './utils'
import { ContractEventEmitter } from './tsc/web3/types'
import { PreImageResult } from './hashedSecret'

import { HASHED_SECRET_WIDTH } from './hashedSecret'
export const EMPTY_HASHED_SECRET = new Uint8Array(HASHED_SECRET_WIDTH).fill(0x00)

class Account {
  private _address?: AccountId
  private _nonceIterator: AsyncIterator<number>
  private _preImageIterator: AsyncGenerator<boolean, boolean, AcknowledgedTicket>

  private _ticketEpoch?: TicketEpoch
  private _ticketEpochListener?: ContractEventEmitter<any>
  private _onChainSecret?: Hash

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

    this._nonceIterator = async function* (this: Account) {
      let nonce = await this.coreConnector.web3.eth.getTransactionCount((await this.address).toHex())

      while (true) {
        yield nonce++
      }
    }.call(this)

    this._preImageIterator = async function* (this: Account) {
      let ticket: AcknowledgedTicket = yield

      let currentPreImage: Promise<PreImageResult> = this.coreConnector.hashedSecret.findPreImage(
        await this.onChainSecret
      )

      let tmp: PreImageResult = await currentPreImage

      while (true) {
        if (
          await isWinningTicket(
            await (await ticket.signedTicket).ticket.hash,
            ticket.response,
            tmp.preImage,
            (await ticket.signedTicket).ticket.winProb
          )
        ) {
          currentPreImage = this.coreConnector.hashedSecret.findPreImage(tmp.preImage)

          ticket.preImage = tmp.preImage

          if (tmp.index == 0) {
            // @TODO dispatch call of next hashedSecret submit
            return true
          } else {
            yield true
          }

          tmp = await currentPreImage
        } else {
          yield false
        }

        ticket = yield false
      }
    }.call(this)
  }

  async stop() {
    this._ticketEpochListener.removeAllListeners()
  }

  get nonce(): Promise<number> {
    return this._nonceIterator.next().then((res) => res.value)
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
            this.coreConnector.log('new ticketEpoch', event.returnValues.counter)

            this._ticketEpoch = new TicketEpoch(event.returnValues.counter)
            this._onChainSecret = new Hash(stringToU8a(event.returnValues.secretHash), Hash.SIZE)
          })
          .on('error', (error) => {
            this.coreConnector.log('error listening to SecretHashSet events', error.message)

            this._ticketEpochListener?.removeAllListeners()
            this._ticketEpoch = undefined
          })
      } catch (err) {
        this.coreConnector.log(err)
        this._ticketEpochListener?.removeAllListeners()
      }
    }
  }
}

export default Account
