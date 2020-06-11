import type HoprEthereum from '.'
import { stringToU8a } from '@hoprnet/hopr-utils'
import { AccountId, Balance, Hash, NativeBalance, TicketEpoch } from './types'
import { pubKeyToAccountId } from './utils'
import { ContractEventEmitter } from './tsc/web3/types'

class Account {
  private _address?: AccountId
  private _nonceIterator: AsyncIterator<number>
  private _ticketEpoch: TicketEpoch
  private _ticketEpochListener: ContractEventEmitter<any>

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
        pubKey,
      },
      offChain: {
        privKey,
        pubKey,
      },
    }

    this._nonceIterator = async function* () {
      let nonce = await this.coreConnector.web3.eth.getTransactionCount((await this.address).toHex())

      while (true) {
        yield nonce++
      }
    }.call(this)
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
    return new Promise(async (resolve, reject) => {
      try {
        if (typeof this._ticketEpoch !== 'undefined') {
          return resolve(this._ticketEpoch)
        }

        // listen for 'SecretHashSet' events and update 'ticketEpoch'
        // on error, safely reset 'ticketEpoch' & event listener
        this._ticketEpochListener = this.coreConnector.hoprChannels.events
          .SecretHashSet({
            fromBlock: 'latest',
            filter: {
              account: (await this.address).toHex(),
            },
          })
          .on('data', (event) => {
            this.coreConnector.log('new ticketEpoch', event.returnValues.counter)

            this._ticketEpoch = new TicketEpoch(event.returnValues.counter)
          })
          .on('error', (error) => {
            this.coreConnector.log('error listening to SecretHashSet events', error.message)

            this._ticketEpochListener.removeAllListeners()
            this._ticketEpoch = undefined
          })

        this._ticketEpoch = new TicketEpoch(
          (await this.coreConnector.hoprChannels.methods.accounts((await this.address).toHex()).call()).counter
        )

        resolve(this._ticketEpoch)
      } catch (err) {
        // reset everything on unexpected error
        this._ticketEpochListener.removeAllListeners()
        this._ticketEpoch = undefined

        reject(err)
      }
    })
  }

  /**
   * Returns the current value of the onChainSecret
   */
  get onChainSecret(): Promise<Hash> {
    return new Promise<Hash>(async (resolve, reject) => {
      try {
        resolve(
          new Hash(
            stringToU8a(
              (await this.coreConnector.hoprChannels.methods.accounts((await this.address).toHex()).call()).hashedSecret
            )
          )
        )
      } catch (err) {
        reject(err)
      }
    })
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
}

export default Account
