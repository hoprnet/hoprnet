import type { Channel as IChannel } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'
import { u8aToHex, toU8a } from '@hoprnet/hopr-utils'
import {
  Balance,
  Channel as ChannelType,
  Hash,
  Moment,
  Public,
  SignedChannel,
  TicketEpoch,
  ChannelEntry
} from '../types'
import TicketFactory from './ticket'
import { hash } from '../utils'

import type HoprEthereum from '..'

class Channel implements IChannel {
  private _settlementWindow?: Moment
  private _channelId?: Hash

  public ticket: TicketFactory

  constructor(
    public coreConnector: HoprEthereum,
    public counterparty: Uint8Array,
    private signedChannel: SignedChannel
  ) {
    this.ticket = new TicketFactory(this)
  }

  get onChainChannel(): Promise<ChannelEntry> {
    return new Promise(async (resolve, reject) => {
      try {
        const channel = await this.coreConnector.channel.getOnChainState(new Public(this.counterparty))
        return resolve(channel)
      } catch (error) {
        return reject(error)
      }
    })
  }

  get stateCounter(): Promise<TicketEpoch> {
    return new Promise<TicketEpoch>(async (resolve, reject) => {
      try {
        const channel = await this.onChainChannel
        return resolve(new TicketEpoch(toU8a(Number(channel.stateCounter))))
      } catch (error) {
        return reject(error)
      }
    })
  }

  get status() {
    return new Promise<'UNINITIALISED' | 'FUNDED' | 'OPEN' | 'PENDING'>(async (resolve, reject) => {
      try {
        const channel = await this.onChainChannel
        return resolve(channel.status)
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
            await this.coreConnector.utils.pubKeyToAccountId(this.counterparty)
          )
        )
      } catch (error) {
        return reject(error)
      }

      return resolve(this._channelId)
    })
  }

  get settlementWindow(): Promise<Moment> {
    if (this._settlementWindow != null) {
      return Promise.resolve(this._settlementWindow)
    }

    return new Promise<Moment>(async (resolve, reject) => {
      try {
        this._settlementWindow = new Moment((await this.onChainChannel).closureTime)
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
    return new Promise<Balance>(async (resolve, reject) => {
      try {
        return resolve(
          new Balance(
            await this.coreConnector.hoprToken.methods
              .balanceOf(u8aToHex(await this.coreConnector.account.address))
              .call()
          )
        )
      } catch (error) {
        return reject(error)
      }
    })
  }

  get currentBalanceOfCounterparty(): Promise<Balance> {
    return new Promise<Balance>(async (resolve, reject) => {
      try {
        return resolve(
          new Balance(
            await this.coreConnector.hoprToken.methods
              .balanceOf(u8aToHex(await this.coreConnector.utils.pubKeyToAccountId(this.counterparty)))
              .call()
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
      if (!(status === 'OPEN' || status === 'PENDING')) {
        throw Error("channel must be 'OPEN' or 'PENDING'")
      }

      if (status === 'OPEN') {
        const tx = await account.signTransaction(
          {
            from: (await account.address).toHex(),
            to: this.coreConnector.hoprChannels.options.address
          },
          this.coreConnector.hoprChannels.methods.initiateChannelClosure(
            u8aToHex(await this.coreConnector.utils.pubKeyToAccountId(this.counterparty))
          )
        )

        receipt = tx.transactionHash
        tx.send()
      } else if (status === 'PENDING') {
        const tx = await account.signTransaction(
          {
            from: (await account.address).toHex(),
            to: this.coreConnector.hoprChannels.options.address
          },
          this.coreConnector.hoprChannels.methods.claimChannelClosure(
            u8aToHex(await this.coreConnector.utils.pubKeyToAccountId(this.counterparty))
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
