import { u8aToHex, u8aEquals } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import {
  AccountId,
  Balance,
  ChannelBalance,
  Channel as ChannelType,
  Hash,
  Public,
  Signature,
  SignedChannel,
  SignedTicket,
  Ticket,
  TicketEpoch
} from '../types'
import { ChannelStatus } from '../types/channel'
import { waitForConfirmation, getId, events, pubKeyToAccountId, sign, isPartyA } from '../utils'
import { ERRORS } from '../constants'

import type HoprEthereum from '..'
import Channel from './channel'

import { Uint8ArrayE } from '../types/extended'

import { CHANNEL_STATES } from './constants'
import { Log } from 'web3-core'
import { TicketStatic } from './ticket'
import debug from 'debug'
const log = debug('hopr-core-ethereum:channel')

const EMPTY_SIGNATURE = new Uint8Array(Signature.SIZE).fill(0x00)
const WIN_PROB = new BN(1)

class ChannelFactory {
  public tickets: TicketStatic

  constructor(private coreConnector: HoprEthereum) {
    this.tickets = new TicketStatic(coreConnector)
  }

  async increaseFunds(counterparty: AccountId, amount: Balance): Promise<void> {
    try {
      const balance = await this.coreConnector.account.balance
      if (balance.isZero()) {
        throw Error(ERRORS.OOF_HOPR)
      }

      await waitForConfirmation(
        (
          await this.coreConnector.signTransaction(
            {
              from: (await this.coreConnector.account.address).toHex(),
              to: this.coreConnector.hoprToken.options.address,
              nonce: await this.coreConnector.account.nonce
            },
            this.coreConnector.hoprToken.methods.send(
              this.coreConnector.hoprChannels.options.address,
              amount.toString(),
              this.coreConnector.web3.eth.abi.encodeParameters(
                ['address', 'address'],
                [(await this.coreConnector.account.address).toHex(), counterparty.toHex()]
              )
            )
          )
        ).send()
      )
    } catch (error) {
      throw error
    }
  }

  async isOpen(counterpartyPubKey: Uint8Array) {
    const counterparty = await pubKeyToAccountId(counterpartyPubKey)
    const channelId = new Hash(await getId(await this.coreConnector.account.address, counterparty))

    const [onChain, offChain]: [boolean, boolean] = await Promise.all([
      this.coreConnector.channel.getOnChainState(channelId).then((channel) => {
        const state = Number(channel.stateCounter) % CHANNEL_STATES
        return state === ChannelStatus.OPEN || state === ChannelStatus.PENDING
      }),
      this.coreConnector.db.get(Buffer.from(this.coreConnector.dbKeys.Channel(counterpartyPubKey))).then(
        () => true,
        (err) => {
          if (err.notFound) {
            return false
          } else {
            throw err
          }
        }
      )
    ])

    if (onChain != offChain) {
      if (!onChain && offChain) {
        log(`Channel ${u8aToHex(channelId)} exists off-chain but not on-chain, deleting data.`)
        await this.coreConnector.channel.deleteOffChainState(counterpartyPubKey)
      } else {
        throw Error(`Channel ${u8aToHex(channelId)} exists on-chain but not off-chain.`)
      }
    }

    return onChain && offChain
  }

  async createDummyChannelTicket(
    counterparty: AccountId,
    challenge: Hash,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    }
  ): Promise<SignedTicket> {
    if (!challenge) {
      throw Error(`Challenge is not set`)
    }

    const winProb = new Uint8ArrayE(new BN(new Uint8Array(Hash.SIZE).fill(0xff)).div(WIN_PROB).toArray('le', Hash.SIZE))

    const signedTicket = new SignedTicket(arr)

    const ticket = new Ticket(
      {
        bytes: signedTicket.buffer,
        offset: signedTicket.ticketOffset
      },
      {
        counterparty,
        challenge,
        epoch: new TicketEpoch(0),
        amount: new Balance(0),
        winProb,
        channelIteration: new TicketEpoch(0)
      }
    )

    await sign(await ticket.hash, this.coreConnector.account.keys.onChain.privKey, undefined, {
      bytes: signedTicket.buffer,
      offset: signedTicket.signatureOffset
    })

    return signedTicket
  }

  async createSignedChannel(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      channel: ChannelType
      signature?: Signature
    }
  ): Promise<SignedChannel> {
    const signedChannel = new SignedChannel(arr, struct)

    if (signedChannel.signature.eq(EMPTY_SIGNATURE)) {
      await signedChannel.channel.sign(this.coreConnector.account.keys.onChain.privKey, undefined, {
        bytes: signedChannel.buffer,
        offset: signedChannel.signatureOffset
      })
    }

    return signedChannel
  }

  async create(
    counterpartyPubKey: Uint8Array,
    _getOnChainPublicKey: (counterparty: Uint8Array) => Promise<Uint8Array>,
    channelBalance?: ChannelBalance,
    sign?: (channelBalance: ChannelBalance) => Promise<SignedChannel>
  ): Promise<Channel> {
    const counterparty = await pubKeyToAccountId(counterpartyPubKey)
    const amPartyA = isPartyA(await this.coreConnector.account.address, counterparty)
    let signedChannel: SignedChannel

    await this.coreConnector.initOnchainValues()

    if (await this.isOpen(counterpartyPubKey)) {
      const record = (await this.coreConnector.db.get(
        Buffer.from(this.coreConnector.dbKeys.Channel(counterpartyPubKey))
      )) as Uint8Array
      signedChannel = new SignedChannel({
        bytes: record.buffer,
        offset: record.byteOffset
      })
      return new Channel(this.coreConnector, counterpartyPubKey, signedChannel)
    }

    if (sign != null && channelBalance != null) {
      const channel = new Channel(this.coreConnector, counterpartyPubKey, signedChannel)

      const amountToFund = new Balance(
        amPartyA ? channelBalance.balance_a : channelBalance.balance.sub(channelBalance.balance_a)
      )
      const amountFunded = await (amPartyA ? channel.balance_a : channel.balance_b)

      if (amountFunded.lt(amountToFund)) {
        await this.increaseFunds(counterparty, new Balance(amountToFund.sub(amountFunded)))
      }

      signedChannel = await sign(channelBalance)

      try {
        await waitForConfirmation(
          (
            await this.coreConnector.signTransaction(
              {
                from: (await this.coreConnector.account.address).toHex(),
                to: this.coreConnector.hoprChannels.options.address,
                nonce: await this.coreConnector.account.nonce
              },
              this.coreConnector.hoprChannels.methods.openChannel(counterparty.toHex())
            )
          ).send()
        )

        await this.coreConnector.db.put(
          Buffer.from(this.coreConnector.dbKeys.Channel(counterpartyPubKey)),
          Buffer.from(signedChannel)
        )
      } catch (e) {
        if (e.message.match(/counterparty must have called init/)) {
          throw new Error('Cannot open channel to an uninitialized counterparty')
        }
        throw e
      }

      return channel
    }
    throw Error('Cannot open channel. Channel is not open and no sign function was given.')
  }

  getAll<T, R>(onData: (channel: Channel) => Promise<T>, onEnd: (promises: Promise<T>[]) => R): Promise<R> {
    const promises: Promise<T>[] = []
    return new Promise<R>((resolve, reject) => {
      this.coreConnector.db
        .createReadStream({
          gte: Buffer.from(this.coreConnector.dbKeys.Channel(new Uint8Array(Hash.SIZE).fill(0x00))),
          lte: Buffer.from(this.coreConnector.dbKeys.Channel(new Uint8Array(Hash.SIZE).fill(0xff)))
        })
        .on('error', (err) => reject(err))
        .on('data', ({ key, value }: { key: Buffer; value: Buffer }) => {
          const signedChannel = new SignedChannel({
            bytes: value.buffer,
            offset: value.byteOffset
          })

          promises.push(
            onData(new Channel(this.coreConnector, this.coreConnector.dbKeys.ChannelKeyParse(key), signedChannel))
          )
        })
        .on('end', () => resolve(onEnd(promises)))
    })
  }

  async closeChannels(): Promise<Balance> {
    const result = new BN(0)

    return this.getAll(
      (channel: Channel) =>
        channel.initiateSettlement().then(() => {
          // @TODO: add balance
          result.iaddn(0)
        }),
      async (promises: Promise<void>[]) => {
        await Promise.all(promises)

        return new Balance(result)
      }
    )
  }

  handleOpeningRequest(source: AsyncIterable<Uint8Array>) {
    return async function* (this: ChannelFactory) {
      for await (const _msg of source) {
        const msg = _msg.slice()
        const signedChannel = new SignedChannel({
          bytes: msg.buffer,
          offset: msg.byteOffset
        })

        const counterpartyPubKey = await signedChannel.signer

        /*
        // Fund both ways
        const counterparty = await pubKeyToAccountId(counterpartyPubKey)
        const channelBalance = signedChannel.channel.balance

        if (isPartyA(await this.coreConnector.account.address, counterparty)) {
          if (channelBalance.balance.sub(channelBalance.balance_a).gtn(0)) {
            if (
              !(await this.coreConnector.account.balance).lt(
                new Balance(channelBalance.balance.sub(channelBalance.balance_a))
              )
            ) {
              await this.increaseFunds(counterparty, new Balance(channelBalance.balance.sub(channelBalance.balance_a)))
            }
          }
        } else {
          if (channelBalance.balance_a.gtn(0)) {
            if (!(await this.coreConnector.account.balance).lt(channelBalance.balance_a)) {
              await this.increaseFunds(counterparty, channelBalance.balance_a)
            }
          }
        }
        */

        // listen for opening event and update DB
        this.coreConnector.channel
          .onceOpen(new Public(this.coreConnector.account.keys.onChain.pubKey), new Public(counterpartyPubKey))
          .then(() => this.coreConnector.channel.saveOffChainState(counterpartyPubKey, signedChannel))

        yield signedChannel.toU8a()
      }
    }.call(this)
  }

  getOffChainState(counterparty: Uint8Array): Promise<SignedChannel> {
    return this.coreConnector.db.get(Buffer.from(this.coreConnector.dbKeys.Channel(counterparty))) as any
  }

  saveOffChainState(counterparty: Uint8Array, signedChannel: SignedChannel) {
    return this.coreConnector.db.put(
      Buffer.from(this.coreConnector.dbKeys.Channel(counterparty)),
      Buffer.from(signedChannel)
    )
  }

  deleteOffChainState(counterparty: Uint8Array) {
    return this.coreConnector.db.del(Buffer.from(this.coreConnector.dbKeys.Channel(counterparty)))
  }

  getOnChainState(channelId: Hash) {
    return this.coreConnector.hoprChannels.methods.channels(channelId.toHex()).call()
  }

  async onceOpen(self: Public, counterparty: Public) {
    const channelId = await getId(await self.toAccountId(), await counterparty.toAccountId())

    return new Promise((resolve, reject) => {
      const subscription = this.coreConnector.web3.eth.subscribe('logs', {
        address: this.coreConnector.hoprChannels.options.address,
        topics: events.OpenedChannelTopics(self, counterparty, true)
      })

      subscription
        .on('data', async (data: Log) => {
          const event = events.decodeOpenedChannelEvent(data)

          const { opener, counterparty } = event.returnValues
          const _channelId = await getId(await opener.toAccountId(), await counterparty.toAccountId())

          if (!u8aEquals(_channelId, channelId)) {
            return
          }

          await subscription.unsubscribe()
          return resolve(event.returnValues)
        })
        .on('error', reject)
    })
  }

  async onceClosed(self: Public, counterparty: Public) {
    const channelId = await getId(await self.toAccountId(), await counterparty.toAccountId())

    return new Promise((resolve, reject) => {
      const subscription = this.coreConnector.web3.eth.subscribe('logs', {
        address: this.coreConnector.hoprChannels.options.address,
        topics: events.ClosedChannelTopics(self, counterparty, true)
      })

      subscription
        .on('data', async (data: Log) => {
          const event = events.decodeClosedChannelEvent(data)

          const { closer, counterparty } = event.returnValues
          const _channelId = await getId(await closer.toAccountId(), await counterparty.toAccountId())

          if (!u8aEquals(_channelId, channelId)) {
            return
          }

          await subscription.unsubscribe()
          return resolve(event.returnValues)
        })
        .on('error', reject)
    })
  }
}

export { ChannelFactory }

export default Channel
