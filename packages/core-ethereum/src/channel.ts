import BN from 'bn.js'
import {
  PublicKey,
  Balance,
  Hash,
  HalfKey,
  AcknowledgedTicket,
  ChannelEntry,
  UnacknowledgedTicket,
  generateChannelId,
  ChannelStatus
} from '@hoprnet/hopr-utils'
import { debug } from '@hoprnet/hopr-utils'
import type { RedeemTicketResponse } from '.'
import { findCommitmentPreImage, bumpCommitment } from './commitment'
import type { ChainWrapper } from './ethereum'
import type Indexer from './indexer'
import type { HoprDB } from '@hoprnet/hopr-utils'
import chalk from 'chalk'
import { EventEmitter } from 'events'

const log = debug('hopr-core-ethereum:channel')


// TODO - legacy, models bidirectional channel.
class Channel {
  constructor(
    private readonly self: PublicKey,
    private readonly counterparty: PublicKey,
    private readonly db: HoprDB,
    private readonly chain: ChainWrapper,
    private readonly indexer: Indexer,
    private readonly events: EventEmitter
  ) {}

  /**
   * Reserve a preImage for the given ticket if it is a winning ticket.
   * @param ticket the acknowledged ticket
   */
  async acknowledge(
    unacknowledgedTicket: UnacknowledgedTicket,
    acknowledgement: HalfKey
  ): Promise<AcknowledgedTicket | null> {
    if (!unacknowledgedTicket.verifyChallenge(acknowledgement)) {
      throw Error(`The acknowledgement is not sufficient to solve the embedded challenge.`)
    }

    const response = unacknowledgedTicket.getResponse(acknowledgement)

    const ticket = unacknowledgedTicket.ticket
    const channelId = this.getThemToUsId()
    const opening = await findCommitmentPreImage(this.db, channelId)

    if (ticket.isWinningTicket(opening, response, ticket.winProb)) {
      const ack = new AcknowledgedTicket(ticket, response, opening, unacknowledgedTicket.signer)

      log(
        `Acknowledging ticket. Using opening ${chalk.yellow(opening.toHex())} and response ${chalk.green(
          response.toHex()
        )}`
      )

      try {
        await bumpCommitment(this.db, channelId)
        this.events.emit('ticket:win', ack, this)
        return ack
      } catch (e) {
        log(`ERROR: commitment could not be bumped ${e}, thus dropping ticket`)
        return null
      }
    } else {
      log(`Got a ticket that is not a win. Dropping ticket.`)
      await this.db.markLosing(unacknowledgedTicket)
      return null
    }
  }

  async getChainCommitment(): Promise<Hash> {
    return (await this.themToUs()).commitment
  }

  getUsToThemId(): Hash {
    return generateChannelId(this.self.toAddress(), this.counterparty.toAddress())
  }

  async usToThem(): Promise<ChannelEntry> {
    return await this.indexer.getChannel(this.getUsToThemId())
  }

  getThemToUsId(): Hash {
    return generateChannelId(this.counterparty.toAddress(), this.self.toAddress())
  }

  async themToUs(): Promise<ChannelEntry> {
    return await this.indexer.getChannel(this.getThemToUsId())
  }

  async fund(myFund: Balance, counterpartyFund: Balance) {
    const myAddress = this.self.toAddress()
    const counterpartyAddress = this.counterparty.toAddress()
    const totalFund = myFund.toBN().add(counterpartyFund.toBN())
    const myBalance = await this.chain.getBalance(myAddress)
    if (totalFund.gt(new BN(myBalance.toBN().toString()))) {
      throw Error('We do not have enough balance to fund the channel')
    }
    const tx = await this.chain.fundChannel(myAddress, counterpartyAddress, myFund, counterpartyFund)
    return await this.indexer.resolvePendingTransaction('channel-updated', tx)
  }

  async open(fundAmount: Balance) {
    // channel may not exist, we can still open it
    let c: ChannelEntry
    try {
      c = await this.usToThem()
    } catch {}
    if (c && c.status !== ChannelStatus.Closed) {
      throw Error('Channel is already opened')
    }

    const myAddress = this.self.toAddress()
    const counterpartyAddress = this.counterparty.toAddress()
    const myBalance = await this.chain.getBalance(myAddress)
    if (new BN(myBalance.toBN().toString()).lt(fundAmount.toBN())) {
      throw Error('We do not have enough balance to open a channel')
    }
    const tx = await this.chain.openChannel(myAddress, counterpartyAddress, fundAmount)
    await this.indexer.resolvePendingTransaction('channel-updated', tx)
    return generateChannelId(myAddress, counterpartyAddress)
  }

}

export { Channel }
