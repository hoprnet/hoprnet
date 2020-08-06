import type { Channel as IChannel } from '@hoprnet/hopr-core-connector-interface'
import { u8aToHex } from '@hoprnet/hopr-utils'
import { Balance, ChannelId, Channel as ChannelType, Hash, Moment, SignedChannel } from '../types'
import TicketFactory from './ticket'
import { ChannelStatus } from '../types/channel'
import { waitForConfirmation, waitFor, hash } from '../utils'

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
      if (status === ChannelStatus.UNINITIALISED) {
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

  private async onceClosed() {
    return this.coreConnector.channel.onceClosed(
      await this.coreConnector.account.address,
      await this.coreConnector.utils.pubKeyToAccountId(this.counterparty)
    )
  }

  // private async onOpen(): Promise<void> {
  //   return onOpen(this.coreConnector, this.counterparty, this._signedChannel)
  // }

  private async onClose(): Promise<void> {
    return this.coreConnector.channel.deleteOffChainState(this.counterparty)
  }

  private get channel(): Promise<OnChainChannel> {
    return new Promise<OnChainChannel>(async (resolve, reject) => {
      try {
        return resolve(await this.coreConnector.channel.getOnChainState(await this.channelId))
      } catch (error) {
        return reject(error)
      }
    })
  }

  private get status(): Promise<ChannelStatus> {
    return new Promise<ChannelStatus>(async (resolve, reject) => {
      try {
        const channel = await this.channel
        const status = Number(channel.stateCounter) % 10

        if (status >= Object.keys(ChannelStatus).length) {
          throw Error("status like this doesn't exist")
        }

        return resolve(status)
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
        this._channelId = new ChannelId(
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
        this._settlementWindow = new Moment((await this.channel).closureTime)
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
        return resolve(new Balance((await this.channel).deposit))
      } catch (error) {
        return reject(error)
      }
    })
  }

  get balance_a(): Promise<Balance> {
    return new Promise<Balance>(async (resolve, reject) => {
      try {
        return resolve(new Balance((await this.channel).partyABalance))
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

  async initiateSettlement(): Promise<void> {
    // @TODO check out whether we can cache this.channel is some way
    let channel = await this.channel
    const status = await this.status

    try {
      if (!(status === ChannelStatus.OPEN || status === ChannelStatus.PENDING)) {
        throw Error("channel must be 'OPEN' or 'PENDING'")
      }

      if (status === ChannelStatus.OPEN) {
        await waitForConfirmation(
          (
            await this.coreConnector.signTransaction(
              this.coreConnector.hoprChannels.methods.initiateChannelClosure(
                u8aToHex(await this.coreConnector.utils.pubKeyToAccountId(this.counterparty))
              ),
              {
                from: (await this.coreConnector.account.address).toHex(),
                to: this.coreConnector.hoprChannels.options.address,
                nonce: await this.coreConnector.account.nonce,
              }
            )
          ).send()
        )

        channel = await this.coreConnector.channel.getOnChainState(await this.channelId)

        await waitFor({
          web3: this.coreConnector.web3,
          network: this.coreConnector.network,
          getCurrentBlock: async () => {
            return this.coreConnector.web3.eth.getBlockNumber().then((blockNumber) => {
              return this.coreConnector.web3.eth.getBlock(blockNumber)
            })
          },
          timestamp: Number(channel.closureTime) * 1e3,
        })

        await waitForConfirmation(
          (
            await this.coreConnector.signTransaction(
              this.coreConnector.hoprChannels.methods.claimChannelClosure(
                u8aToHex(await this.coreConnector.utils.pubKeyToAccountId(this.counterparty))
              ),
              {
                from: (await this.coreConnector.account.address).toHex(),
                to: this.coreConnector.hoprChannels.options.address,
                nonce: await this.coreConnector.account.nonce,
              }
            )
          ).send()
        )
      } else {
        await this.onceClosed()
      }

      await this.onClose()
    } catch (error) {
      throw error
    }
  }

  // @TODO: remove this, no longer needed
  async getPreviousChallenges(): Promise<Hash> {
    return new Hash()
  }

  async testAndSetNonce(signature: Uint8Array): Promise<void> {
    const key = new Hash(this.coreConnector.dbKeys.Nonce(await this.channelId, await hash(signature))).toHex()

    try {
      await this.coreConnector.db.get(key)
    } catch (err) {
      if (err.notFound) {
        await this.coreConnector.db.put(key, new Uint8Array())
        return
      }

      throw err
    }

    throw Error('Nonces must not be used twice.')
  }
}

export default Channel
