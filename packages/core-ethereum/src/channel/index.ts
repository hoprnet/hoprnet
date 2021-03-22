import type { ChannelUpdate } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'
import {
  AccountId,
  Balance,
  ChannelState,
  Hash,
  Public,
  SignedTicket,
  Ticket,
  TicketEpoch,
  Moment,
  ChannelEntry
} from '../types'
//import TicketFactory from './ticket'
import { waitForConfirmation, getId, pubKeyToAccountId, sign, isPartyA, Log, hash } from '../utils'
import type { Channel as IChannel } from '@hoprnet/hopr-core-connector-interface'
import { Uint8ArrayE } from '../types/extended'
import { toU8a } from '@hoprnet/hopr-utils'
import { getChannel as getOnChainState, initiateChannelSettlement } from '../chainInteractions'

const log = Log(['channel-factory'])

const WIN_PROB = new BN(1)

class Channel implements IChannel {
  private _settlementWindow?: Moment
  //public ticket: TicketFactory

  constructor(
    //public coreConnector: HoprEthereum,
    readonly self: Public,
    readonly counterparty: Public,
    readonly state: ChannelState
  ) {
    //this.ticket = new TicketFactory(this)
  }

  async stateCounter(): Promise<TicketEpoch> {
    const channel = await getOnChainState(this.self, this.counterparty)
    return new TicketEpoch(toU8a(Number(channel.stateCounter)))
  }

  async channelId(): Promise<Hash> {
    return new Hash(await getId(await this.self, await pubKeyToAccountId(this.counterparty)))
  }

  get settlementWindow(): Promise<Moment> {
    if (this._settlementWindow != null) {
      return Promise.resolve(this._settlementWindow)
    }

    return new Promise<Moment>(async (resolve, reject) => {
      try {
        this._settlementWindow = new Moment((await getOnChainState(this.self, this.counterparty)).closureTime)
      } catch (error) {
        return reject(error)
      }

      return resolve(this._settlementWindow)
    })
  }

  async balance(): Promise<Balance> {
    return new Balance((await getOnChainState(this.self, this.counterparty)).deposit)
  }

  async balance_a(): Promise<Balance> {
    return new Balance((await getOnChainState(this.self, this.counterparty)).partyABalance)
  }

  async balance_b(): Promise<Balance> {
    const { deposit, partyABalance } = await getOnChainState(this.self, this.counterparty)
    return new Balance(new BN(deposit).sub(new BN(partyABalance)))
  }

  /*
  async currentBalanceOfCounterparty(): Promise<Balance> {
    return new Balance(
            await this.coreConnector.hoprToken.methods
              .balanceOf(u8aToHex(await pubKeyToAccountId(this.counterparty)))
              .call()
          )
        )
      } catch (error) {
        return reject(error)
      }
  }
  */

  async initiateSettlement(): Promise<string> {
    if (!(this.state.status === 'OPEN' || this.state.status === 'PENDING')) {
      throw Error("channel must be 'OPEN' or 'PENDING'")
    }
    return await initiateChannelSettlement()
  }

  async testAndSetNonce(db, dbKeys, signature: Uint8Array): Promise<void> {
    const key = new Hash(dbKeys.Nonce(await this.channelId, await hash(signature))).toHex()

    try {
      await db.get(Buffer.from(key))
    } catch (err) {
      if (err.notFound) {
        await db.put(Buffer.from(key), Buffer.from(''))
        return
      }

      throw err
    }

    throw Error('Nonces must not be used twice.')
  }
}

// TODO listenForChannels
export async function subscribeToChannels(indexer) {
  const self = new Public(this.coreConnector.account.keys.onChain.pubKey)
  const selfAccountId = await self.toAccountId()

  indexer.on('channelOpened', async ({ partyA: _partyA, partyB: _partyB, channelEntry }: ChannelUpdate) => {
    const partyA = new Public(_partyA)
    const partyAAccountId = await partyA.toAccountId()
    const partyB = new Public(_partyB)

    log('channelOpened', partyA.toHex(), partyB.toHex())
    const isOurs = partyA.eq(self) || partyB.eq(self)
    if (!isOurs) return

    await onOpen(isPartyA(selfAccountId, partyAAccountId) ? partyB : partyA, channelEntry as ChannelEntry)
  })

  indexer.on('channelClosed', async ({ partyA: _partyA, partyB: _partyB }: ChannelUpdate) => {
    const partyA = new Public(_partyA)
    const partyAAccountId = await partyA.toAccountId()
    const partyB = new Public(_partyB)

    log('channelClosed', partyA.toHex(), partyB.toHex())
    const isOurs = partyA.eq(self) || partyB.eq(self)
    if (!isOurs) return

    await onClose(isPartyA(selfAccountId, partyAAccountId) ? partyB : partyA)
  })
}

async function onOpen(counterparty: Public, channelEntry: ChannelEntry): Promise<void> {
  log('Received open event for channel with %s', counterparty.toHex())

  const state = new ChannelState(
    new Balance(new BN(channelEntry.deposit)),
    new Balance(new BN(channelEntry.partyABalance)),
    stateCounterToStatus(channelEntry.stateCounter.toNumber())
  )

  // we store it, if we have an previous signed channel
  // under this counterparty, we replace it
  await this.saveOffChainState(counterparty, state)
}

async function onClose(counterparty: Public): Promise<void> {
  log('Received close event for channel with %s', counterparty.toHex())
  // TODO -
  // we don't know which channel iteration this
  // this signed channel is from so we do nothing
}

// TODO rename
export async function getOffChainState(db, dbKeys, counterparty: Uint8Array): Promise<ChannelState> {
  return db.get(Buffer.from(dbKeys.Channel(counterparty)))
}

class ChannelFactory {
  /*
  async increaseFunds(counterparty: AccountId, amount: Balance): Promise<void> {
    try {
      const { account } = this.coreConnector

      const balance = await account.getBalance()
      if (balance.isZero()) {
        throw Error(ERRORS.OOF_HOPR)
      }

      await waitForConfirmation(
        (
          await account.signTransaction(
            {
              from: (await account.address).toHex(),
              to: this.coreConnector.hoprToken.options.address
            },
            this.coreConnector.hoprToken.methods.send(
              this.coreConnector.hoprChannels.options.address,
              amount.toString(),
              this.coreConnector.web3.eth.abi.encodeParameters(
                ['address', 'address'],
                [(await account.address).toHex(), counterparty.toHex()]
              )
            )
          )
        ).send()
      )
    } catch (error) {
      throw error
    }
  }
*/

  async isOpen(address, counterpartyPubKey: Uint8Array) {
    const counterparty = await pubKeyToAccountId(counterpartyPubKey)
    const channelId = new Hash(await getId(address, counterparty))

    const [onChain, offChain]: [boolean, boolean] = await Promise.all([
      getOnChainState(new Public(counterpartyPubKey)).then((channel) => {
        return channel.status === 'OPEN' || channel.status === 'PENDING'
      }),
      getOffChainState(counterpartyPubKey).then(
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
        log(`Channel ${channelId.toHex()} exists off-chain but not on-chain.`)
        // we don't know which channel iteration this
        // this signed channel is from so we do nothing
        // await this.coreConnector.channel.deleteOffChainState(counterpartyPubKey)
      } else {
        throw Error(`Channel ${channelId.toHex()} exists on-chain but not off-chain.`)
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

    const signature = await sign(await ticket.hash, account.keys.onChain.privKey)
    signedTicket.set(signature, signedTicket.signatureOffset - signedTicket.byteOffset)
    return signedTicket
  }

  async create(counterpartyPubKey: Uint8Array, balance: Balance, balance_a: Balance): Promise<ChannelState> {
    const { account } = this.coreConnector
    const counterparty = await pubKeyToAccountId(counterpartyPubKey)

    await this.coreConnector.initOnchainValues()

    if (await this.isOpen(counterpartyPubKey)) {
      const channelState = await this.getOffChainState(counterpartyPubKey)
      return // TODO
    }

    if (sign != null) {
      /*
      if (amountFunded.lt(amountToFund)) {
        await this.increaseFunds(counterparty, new Balance(amountToFund.sub(amountFunded)))
      }
      */

      const state = new ChannelState(balance, balance_a, stateCounterToStatus(0))

      try {
        await waitForConfirmation(
          (
            await account.signTransaction(
              {
                from: (await account.address).toHex(),
                to: hoprChannels.options.address
              },
              hoprChannels.methods.openChannel(counterparty.toHex())
            )
          ).send()
        )

        await db.put(Buffer.from(dbKeys.Channel(counterpartyPubKey)), Buffer.from(state.serialize()))
      } catch (e) {
        if (e.message.match(/counterparty must have called init/)) {
          throw new Error('Cannot open channel to an uninitialized counterparty')
        }
        throw e
      }

      return state
    }

    throw Error('Cannot open channel. Channel is not open and no sign function was given.')
  }

  getAll<T, R>(onData: (channel: Channel) => Promise<T>, onEnd: (promises: Promise<T>[]) => R): Promise<R> {
    const promises: Promise<T>[] = []
    return new Promise<R>((resolve, reject) => {
      db.createReadStream({
          gte: Buffer.from(dbKeys.Channel(new Uint8Array(Hash.SIZE).fill(0x00))),
          lte: Buffer.from(dbKeys.Channel(new Uint8Array(Hash.SIZE).fill(0xff)))
        })
        .on('error', (err) => reject(err))
        .on('data', ({ key, value }: { key: Buffer; value: Buffer }) => {
          const signedChannel = ChannelState.deserialize(value)
          promises.push(
            onData(new Channel(dbKeys.ChannelKeyParse(key), signedChannel))
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

  /*
  handleOpeningRequest(source: AsyncIterable<Uint8Array>) {
    return async function* (this: ChannelFactory) {
      for await (const _msg of source) {
        const msg = _msg.slice()
        const signedChannel = ChannelState.deserialize(msg)
        yield signedChannel.serialize()
      }
    }.call(this)
  }
  */

  saveOffChainState(counterparty: Uint8Array, state: ChannelState) {
    return db.put(Buffer.from(dbKeys.Channel(counterparty)), Buffer.from(state.serialize()))
  }

  deleteOffChainState(db, dbKeys, counterparty: Uint8Array) {
    return db.del(Buffer.from(dbKeys.Channel(counterparty)))
  }
}

export { Channel }
