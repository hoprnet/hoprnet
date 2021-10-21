import {
  PublicKey,
  Hash,
  HalfKey,
  AcknowledgedTicket,
  ChannelEntry,
  UnacknowledgedTicket,
  generateChannelId,
} from '@hoprnet/hopr-utils'
import { debug } from '@hoprnet/hopr-utils'
import { findCommitmentPreImage, bumpCommitment } from './commitment'
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

}

export { Channel }
