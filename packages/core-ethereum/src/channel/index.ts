import type { ChannelUpdate, AcknowledgedTicket } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'
import {
  AccountId,
  Balance,
  ChannelBalance,
  Channel as ChannelType,
  ChannelState,
  Hash,
  Public,
  Signature,
  SignedChannel,
  SignedTicket,
  Ticket,
  TicketEpoch,
  ChannelEntry
} from '../types'
import {
  waitForConfirmation,
  getId,
  pubKeyToAccountId,
  sign,
  isPartyA,
  getParties,
  Log,
  stateCounterToStatus
} from '../utils'
import { ERRORS } from '../constants'
import type HoprEthereum from '..'
import Channel from './channel'
import { Uint8ArrayE } from '../types/extended'
import { u8aIsEmpty, u8aToHex } from '@hoprnet/hopr-utils'
import { checkChallenge } from '../utils'

const log = Log(['channel-factory'])

const EMPTY_SIGNATURE = new Uint8Array(Signature.SIZE).fill(0x00)
const WIN_PROB = new BN(1)
const INVALID_MESSAGES = {
  NO_PRE_IMAGE: 'PreImage is empty.',
  INVALID_CHALLENGE: 'Invalid challenge.',
  NOT_WINNING: 'Not a winning ticket.'
}

class ChannelFactory {
  constructor(private coreConnector: HoprEthereum) {
    this.listenForChannels()
  }

  async listenForChannels(): Promise<void> {
    const { indexer } = this.coreConnector
    const self = new Public(this.coreConnector.account.keys.onChain.pubKey)
    const selfAccountId = await self.toAccountId()

    indexer.on('channelOpened', async ({ partyA: _partyA, partyB: _partyB, channelEntry }: ChannelUpdate) => {
      const partyA = new Public(_partyA)
      const partyAAccountId = await partyA.toAccountId()
      const partyB = new Public(_partyB)

      log('channelOpened', partyA.toHex(), partyB.toHex())
      const isOurs = partyA.eq(self) || partyB.eq(self)
      if (!isOurs) return

      await this.onOpen(isPartyA(selfAccountId, partyAAccountId) ? partyB : partyA, channelEntry as ChannelEntry)
    })

    indexer.on('channelClosed', async ({ partyA: _partyA, partyB: _partyB }: ChannelUpdate) => {
      const partyA = new Public(_partyA)
      const partyAAccountId = await partyA.toAccountId()
      const partyB = new Public(_partyB)

      log('channelClosed', partyA.toHex(), partyB.toHex())
      const isOurs = partyA.eq(self) || partyB.eq(self)
      if (!isOurs) return

      await this.onClose(isPartyA(selfAccountId, partyAAccountId) ? partyB : partyA)
    })
  }

  async onOpen(counterparty: Public, channelEntry: ChannelEntry): Promise<void> {
    log('Received open event for channel with %s', counterparty.toHex())

    const balance = new ChannelBalance(undefined, {
      balance: new BN(channelEntry.deposit),
      balance_a: new BN(channelEntry.partyABalance)
    })
    const state = new ChannelState(undefined, { state: stateCounterToStatus(channelEntry.stateCounter.toNumber()) })
    const newChannel = new ChannelType(undefined, {
      state,
      balance
    })

    // we store it, if we have an previous signed channel
    // under this counterparty, we replace it
    await this.saveOffChainState(
      counterparty,
      new SignedChannel(undefined, {
        channel: newChannel,
        counterparty
      })
    )
  }

  async onClose(counterparty: Public): Promise<void> {
    log('Received close event for channel with %s', counterparty.toHex())
    // we don't know which channel iteration this
    // this signed channel is from so we do nothing
    // await this.deleteOffChainState(counterparty)
  }

  async increaseFunds(counterparty: AccountId, amount: Balance): Promise<void> {
    try {
      const { account } = this.coreConnector

      const balance = await account.balance
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

  async isOpen(counterpartyPubKey: Uint8Array) {
    const counterparty = await pubKeyToAccountId(counterpartyPubKey)
    const channelId = new Hash(await getId(await this.coreConnector.account.address, counterparty))

    const [onChain, offChain]: [boolean, boolean] = await Promise.all([
      this.coreConnector.channel.getOnChainState(new Public(counterpartyPubKey)).then((channel) => {
        return channel.status === 'OPEN' || channel.status === 'PENDING'
      }),
      this.getOffChainState(counterpartyPubKey).then(
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
    const { account } = this.coreConnector
    const counterparty = await pubKeyToAccountId(counterpartyPubKey)
    const amPartyA = isPartyA(await account.address, counterparty)
    let signedChannel: SignedChannel

    await this.coreConnector.initOnchainValues()

    if (await this.isOpen(counterpartyPubKey)) {
      const record = await this.getOffChainState(counterpartyPubKey)
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

      const state = new ChannelState(undefined, { state: stateCounterToStatus(0) })

      // signedChannel = await sign(channelBalance)
      signedChannel = new SignedChannel(undefined, {
        channel: new ChannelType(undefined, {
          state,
          balance: channelBalance
        }),
        counterparty: new Public(counterpartyPubKey)
      })

      try {
        await waitForConfirmation(
          (
            await account.signTransaction(
              {
                from: (await account.address).toHex(),
                to: this.coreConnector.hoprChannels.options.address
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

  async getOnChainState(counterparty: Public): Promise<ChannelEntry> {
    const inGanache = !this.coreConnector.network || this.coreConnector.network === 'localhost'
    const self = new Public(this.coreConnector.account.keys.onChain.pubKey)
    const selfAccountId = await self.toAccountId()
    const counterpartyAccountId = await counterparty.toAccountId()
    const [partyAAccountId] = getParties(selfAccountId, counterpartyAccountId)

    // HACK: when running our unit/intergration tests using ganache, the indexer doesn't have enough
    // time to pick up the events and reduce the data - here we are doing 2 things wrong:
    // 1. all our unit tests are actually intergration tests, nothing is mocked
    // 2. our actual intergration tests do not have any block mining time
    // this will be tackled in the upcoming refactor
    if (inGanache) {
      const channelId = await getId(selfAccountId, counterpartyAccountId)
      const response = await this.coreConnector.hoprChannels.methods.channels(channelId.toHex()).call()

      return new ChannelEntry(undefined, {
        blockNumber: new BN(0),
        transactionIndex: new BN(0),
        logIndex: new BN(0),
        deposit: new BN(response.deposit),
        partyABalance: new BN(response.partyABalance),
        closureTime: new BN(response.closureTime),
        stateCounter: new BN(response.stateCounter),
        closureByPartyA: response.closureByPartyA
      })
    } else {
      let channelEntry = await this.coreConnector.indexer.getChannelEntry(
        partyAAccountId.eq(selfAccountId) ? self : counterparty,
        partyAAccountId.eq(selfAccountId) ? counterparty : self
      )
      if (channelEntry) return channelEntry

      // when channelEntry is not found, the onchain data is all 0
      return new ChannelEntry(undefined, {
        blockNumber: new BN(0),
        transactionIndex: new BN(0),
        logIndex: new BN(0),
        deposit: new BN(0),
        partyABalance: new BN(0),
        closureTime: new BN(0),
        stateCounter: new BN(0),
        closureByPartyA: false
      })
    }
  }

  public async redeemTicket(
    ackTicket: AcknowledgedTicket
  ): Promise<
    | {
        status: 'SUCCESS'
        receipt: string
      }
    | {
        status: 'FAILURE'
        message: string
      }
    | {
        status: 'ERROR'
        error: Error | string
      }
  > {
    const ticketChallenge = ackTicket.response

    try {
      const signedTicket = await ackTicket.signedTicket
      const ticket = signedTicket.ticket

      log('Submitting ticket', u8aToHex(ticketChallenge))
      const { hoprChannels, account, utils } = this.coreConnector
      const { r, s, v } = utils.getSignatureParameters(signedTicket.signature)

      if (u8aIsEmpty(ackTicket.preImage)) {
        log(`Failed to submit ticket ${u8aToHex(ticketChallenge)}: ${INVALID_MESSAGES.NO_PRE_IMAGE}`)
        return {
          status: 'FAILURE',
          message: INVALID_MESSAGES.NO_PRE_IMAGE
        }
      }

      const validChallenge = await checkChallenge(ticket.challenge, ackTicket.response)
      if (!validChallenge) {
        log(`Failed to submit ticket ${u8aToHex(ticketChallenge)}: ${INVALID_MESSAGES.INVALID_CHALLENGE}`)
        return {
          status: 'FAILURE',
          message: INVALID_MESSAGES.INVALID_CHALLENGE
        }
      }

      const isWinning = await this.coreConnector.probabilisticPayments.validateTicket(ackTicket)
      if (!isWinning) {
        log(`Failed to submit ticket ${u8aToHex(ticketChallenge)}: ${INVALID_MESSAGES.NOT_WINNING}`)
        return {
          status: 'FAILURE',
          message: INVALID_MESSAGES.NOT_WINNING
        }
      }

      const counterparty = await this.coreConnector.utils.pubKeyToAccountId(await signedTicket.signer)

      const transaction = await account.signTransaction(
        {
          from: (await account.address).toHex(),
          to: hoprChannels.options.address
        },
        hoprChannels.methods.redeemTicket(
          u8aToHex(ackTicket.preImage),
          u8aToHex(ackTicket.response),
          ticket.amount.toString(),
          u8aToHex(ticket.winProb),
          u8aToHex(counterparty),
          u8aToHex(r),
          u8aToHex(s),
          v + 27
        )
      )

      await transaction.send()
      //ackTicket.redeemed = true
      this.coreConnector.probabilisticPayments.updateOnChainSecret(ackTicket.preImage) // redemption contract updates on chain

      log('Successfully submitted ticket', u8aToHex(ticketChallenge))
      return {
        status: 'SUCCESS',
        receipt: transaction.transactionHash
      }
    } catch (err) {
      log('Unexpected error when submitting ticket', u8aToHex(ticketChallenge), err)
      return {
        status: 'ERROR',
        error: err
      }
    }
  }
}

export { ChannelFactory }

export default Channel
