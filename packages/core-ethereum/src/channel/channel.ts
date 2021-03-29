import type { Channel as IChannel, Types as Interfaces } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'
import { Balance, Channel as ChannelType, Hash, Public, SignedChannel, ChannelEntry, UINT256 } from '../types'
import TicketFactory from './ticket'
import { hash } from '../utils'

import type HoprEthereum from '..'

class Channel implements IChannel {
  private _settlementWindow?: UINT256
  private _channelId?: Hash

  public ticket: TicketFactory

  constructor(
    public coreConnector: HoprEthereum,
    public counterparty: Uint8Array,
    private signedChannel: SignedChannel
  ) {
    this.ticket = new TicketFactory(this)
  }

  private get onChainChannel(): Promise<ChannelEntry> {
    return new Promise(async (resolve, reject) => {
      try {
        const channel = await this.coreConnector.channel.getOnChainState(new Public(this.counterparty))
        return resolve(channel)
      } catch (error) {
        return reject(error)
      }
    })
  }

  get stateCounter(): Promise<UINT256> {
    return new Promise<UINT256>(async (resolve, reject) => {
      try {
        const channel = await this.onChainChannel
        return resolve(new UINT256(channel.stateCounter))
      } catch (error) {
        return reject(error)
      }
    })
  }

  get status() {
    return new Promise<ReturnType<Interfaces.ChannelEntry['getStatus']>>(async (resolve, reject) => {
      try {
        const channel = await this.onChainChannel
        return resolve(channel.getStatus())
      } catch (error) {
        return reject(error)
      }
    })
  }

  get offChainCounterparty(): Promise<Uint8Array> {
    return Promise.resolve(this.counterparty)
  }

  get channelId(): Promise<Hash> {
    if (this._channelId != null) {
      return Promise.resolve<Hash>(this._channelId)
    }

    return new Promise<Hash>(async (resolve, reject) => {
      try {
        this._channelId = new Hash(
          await this.coreConnector.utils.getId(
            await this.coreConnector.account.address,
            await this.coreConnector.utils.pubKeyToAddress(this.counterparty)
          )
        )
      } catch (error) {
        return reject(error)
      }

      return resolve(this._channelId)
    })
  }

  get settlementWindow(): Promise<UINT256> {
    if (this._settlementWindow != null) {
      return Promise.resolve(this._settlementWindow)
    }

    return new Promise<UINT256>(async (resolve, reject) => {
      try {
        this._settlementWindow = new UINT256((await this.onChainChannel).closureTime)
      } catch (error) {
        return reject(error)
      }

      return resolve(this._settlementWindow)
    })
  }

  get state(): Promise<ChannelType> {
    return Promise.resolve(this.signedChannel.channel)
  }

  get balance(): Promise<Balance> {
    return new Promise<Balance>(async (resolve, reject) => {
      try {
        return resolve(new Balance((await this.onChainChannel).deposit))
      } catch (error) {
        return reject(error)
      }
    })
  }

  get balance_a(): Promise<Balance> {
    return new Promise<Balance>(async (resolve, reject) => {
      try {
        return resolve(new Balance((await this.onChainChannel).partyABalance))
      } catch (error) {
        return reject(error)
      }
    })
  }

  get balance_b(): Promise<Balance> {
    return new Promise<Balance>(async (resolve, reject) => {
      try {
        const { deposit, partyABalance } = await this.onChainChannel
        return resolve(new Balance(new BN(deposit).sub(new BN(partyABalance))))
      } catch (error) {
        return reject(error)
      }
    })
  }

  get currentBalance(): Promise<Balance> {
    return this.coreConnector.account.getBalance()
  }

  get currentBalanceOfCounterparty(): Promise<Balance> {
    return new Promise<Balance>(async (resolve, reject) => {
      try {
        return resolve(
          new Balance(
            new BN(
              await this.coreConnector.hoprToken.methods
                .balanceOf((await this.coreConnector.utils.pubKeyToAddress(this.counterparty)).toHex())
                .call()
            )
          )
        )
      } catch (error) {
        return reject(error)
      }
    })
  }

  async initiateSettlement(): Promise<string> {
    const { account } = this.coreConnector
    const status = await this.status
    let receipt: string

    try {
      if (!(status === 'OPEN' || status === 'PENDING_TO_CLOSE')) {
        throw Error("channel must be 'OPEN' or 'PENDING_TO_CLOSE'")
      }

      if (status === 'OPEN') {
        const tx = await account.signTransaction(
          {
            from: (await account.address).toHex(),
            to: this.coreConnector.hoprChannels.options.address
          },
          this.coreConnector.hoprChannels.methods.initiateChannelClosure(
            (await this.coreConnector.utils.pubKeyToAddress(this.counterparty)).toHex()
          )
        )

        receipt = tx.transactionHash
        tx.send()
      } else if (status === 'PENDING_TO_CLOSE') {
        const tx = await account.signTransaction(
          {
            from: (await account.address).toHex(),
            to: this.coreConnector.hoprChannels.options.address
          },
          this.coreConnector.hoprChannels.methods.finalizeChannelClosure(
            (await this.coreConnector.utils.pubKeyToAddress(this.counterparty)).toHex()
          )
        )

        receipt = tx.transactionHash
        tx.send()
      }

      return receipt
    } catch (error) {
      throw error
    }
  }

  async testAndSetNonce(signature: Uint8Array): Promise<void> {
    const key = new Hash(this.coreConnector.dbKeys.Nonce(await this.channelId, await hash(signature))).toHex()

    try {
      await this.coreConnector.db.get(Buffer.from(key))
    } catch (err) {
      if (err.notFound) {
        await this.coreConnector.db.put(Buffer.from(key), Buffer.from(''))
        return
      }

      throw err
    }

    throw Error('Nonces must not be used twice.')
  }
}

export default Channel
