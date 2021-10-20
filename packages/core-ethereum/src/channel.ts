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
  ChannelStatus,
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

// Lock
let _redeemingAll: Promise<void> | undefined = undefined

export async function redeemTickets(
  source: PublicKey,
  db: HoprDB,
  chain: ChainWrapper,
  indexer: Indexer,
  events: EventEmitter
): Promise<void> {
  if (_redeemingAll) {
    return _redeemingAll
  }
  // Because tickets are ordered and require the previous redemption to
  // have succeeded before we can redeem the next, we need to do this
  // sequentially.
  const tickets = await db.getAcknowledgedTickets({ signer: source })
  log(`redeeming ${tickets.length} tickets from ${source.toB58String()}`)
  const _redeemAll = async () => {
    try {
      for (const ticket of tickets) {
        log('redeeming ticket', ticket)
        const result = await redeemTicket(source, ticket, db, chain, indexer, events)
        if (result.status !== 'SUCCESS') {
          log('Error redeeming ticket', result)
          // We need to abort as tickets require ordered redemption.
          return
        }
        log('ticket was redeemed')
      }
    } catch (e) {
      // We are going to swallow the error here, as more than one consumer may
      // be inspecting this same promise.
      log('Error when redeeming tickets, aborting', e)
    }
    log(`redemption of tickets from ${source.toB58String()} is complete`)
    _redeemingAll = undefined
  }
  _redeemingAll = _redeemAll()
  return _redeemingAll
}

// Private as out of order redemption will break things - redeem all at once.
async function redeemTicket(
  counterparty: PublicKey,
  ackTicket: AcknowledgedTicket,
  db: HoprDB,
  chain: ChainWrapper,
  indexer: Indexer,
  events: EventEmitter
): Promise<RedeemTicketResponse> {
  if (!ackTicket.verify(counterparty)) {
    return {
      status: 'FAILURE',
      message: 'Invalid response to acknowledgement'
    }
  }

  try {
    const ticket = ackTicket.ticket

    log('Submitting ticket', ackTicket.response.toHex())
    const emptyPreImage = new Hash(new Uint8Array(Hash.SIZE).fill(0x00))
    const hasPreImage = !ackTicket.preImage.eq(emptyPreImage)
    if (!hasPreImage) {
      log(`Failed to submit ticket ${ackTicket.response.toHex()}: 'PreImage is empty.'`)
      return {
        status: 'FAILURE',
        message: 'PreImage is empty.'
      }
    }

    const isWinning = ticket.isWinningTicket(ackTicket.preImage, ackTicket.response, ticket.winProb)

    if (!isWinning) {
      log(`Failed to submit ticket ${ackTicket.response.toHex()}:  'Not a winning ticket.'`)
      return {
        status: 'FAILURE',
        message: 'Not a winning ticket.'
      }
    }

    const receipt = await chain.redeemTicket(counterparty.toAddress(), ackTicket, ticket)
    await indexer.resolvePendingTransaction('channel-updated', receipt)

    log('Successfully submitted ticket', ackTicket.response.toHex())
    await db.markRedeemeed(ackTicket)
    events.emit('ticket:redeemed', ackTicket)
    return {
      status: 'SUCCESS',
      receipt,
      ackTicket
    }
  } catch (err) {
    // TODO delete ackTicket -- check if it's due to gas!
    log('Unexpected error when redeeming ticket', ackTicket.response.toHex(), err)
    return {
      status: 'ERROR',
      error: err
    }
  }
}

export const _redeemTicket = redeemTicket // For tests

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

  async initializeClosure() {
    const c = await this.usToThem()
    const counterpartyAddress = this.counterparty.toAddress()
    if (c.status !== ChannelStatus.Open && c.status !== ChannelStatus.WaitingForCommitment) {
      throw Error('Channel status is not OPEN or WAITING FOR COMMITMENT')
    }
    const tx = await this.chain.initiateChannelClosure(counterpartyAddress)
    return await this.indexer.resolvePendingTransaction('channel-updated', tx)
  }

  async finalizeClosure() {
    const c = await this.usToThem()
    const counterpartyAddress = this.counterparty.toAddress()

    if (c.status !== ChannelStatus.PendingToClose) {
      throw Error('Channel status is not PENDING_TO_CLOSE')
    }
    const tx = await this.chain.finalizeChannelClosure(counterpartyAddress)
    return await this.indexer.resolvePendingTransaction('channel-updated', tx)
  }


}

export { Channel }
