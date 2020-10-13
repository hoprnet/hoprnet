import type { Channel as IChannel } from '@hoprnet/hopr-core-connector-interface'
import { u8aToHex } from '@hoprnet/hopr-utils'
import { Balance, Channel as ChannelType, Hash, Moment, Public, SignedChannel } from '../types'
import TicketFactory from './ticket'
import { ChannelStatus } from '../types/channel'
import { hash } from '../utils'

import type HoprEthereum from '..'

import { OnChainChannel } from './types'

class Channel implements IChannel {
  private _signedChannel: SignedChannel
  private _settlementWindow?: Moment
  private _channelId?: Hash

  public ticket: TicketFactory

  constructor(public coreConnector: HoprEthereum, public counterparty: Uint8Array, signedChannel: SignedChannel) {
    this._signedChannel = signedChannel

    // check if channel still exists
    this.status.then((status) => {
      if (status === 'UNINITIALISED') {
        this.coreConnector.log.log('found channel off-chain but its closed on-chain')
        this.onClose()
      }
    })

    // if channel is closed
    this.onceClosed().then(async () => {
      return this.onClose()
    })

    this.ticket = new TicketFactory(this)
  }

  private get onChainChannel(): Promise<OnChainChannel> {
    return new Promise<OnChainChannel>(async (resolve, reject) => {
      try {
        return resolve(await this.coreConnector.channel.getOnChainState(await this.channelId))
      } catch (error) {
        return reject(error)
      }
    })
  }

  get status() {
    return new Promise<'UNINITIALISED' | 'FUNDING' | 'OPEN' | 'PENDING'>(async (resolve, reject) => {
      try {
        const channel = await this.onChainChannel
        const status = Number(channel.stateCounter) % 10

        if (status >= Object.keys(ChannelStatus).length) {
          throw Error("status like this doesn't exist")
        }

        if (status === ChannelStatus.UNINITIALISED) return resolve('UNINITIALISED')
        else if (status === ChannelStatus.FUNDING) return resolve('FUNDING')
        else if (status === ChannelStatus.OPEN) return resolve('OPEN')
        return resolve('PENDING')
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
    return Promise.resolve(this._signedChannel.channel)
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
    const status = await this.status
    let receipt: string

    try {
      if (!(status === 'OPEN' || status === 'PENDING')) {
        throw Error("channel must be 'OPEN' or 'PENDING'")
      }

      if (status === 'OPEN') {
        const tx = await this.coreConnector.signTransaction(
          {
            from: (await this.coreConnector.account.address).toHex(),
            to: this.coreConnector.hoprChannels.options.address,
            nonce: await this.coreConnector.account.nonce
          },
          this.coreConnector.hoprChannels.methods.initiateChannelClosure(
            u8aToHex(await this.coreConnector.utils.pubKeyToAccountId(this.counterparty))
          )
        )

        receipt = tx.transactionHash
        tx.send()
      } else if (status === 'PENDING') {
        const tx = await this.coreConnector.signTransaction(
          {
            from: (await this.coreConnector.account.address).toHex(),
            to: this.coreConnector.hoprChannels.options.address,
            nonce: await this.coreConnector.account.nonce
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

  private async onceClosed() {
    return this.coreConnector.channel.onceClosed(
      new Public(this.coreConnector.account.keys.onChain.pubKey),
      new Public(this.counterparty)
    )
  }

  // private async onOpen(): Promise<void> {
  //   return onOpen(this.coreConnector, this.counterparty, this._signedChannel)
  // }

  private async onClose(): Promise<void> {
    return this.coreConnector.channel.deleteOffChainState(this.counterparty)
  }
}

export default Channel
