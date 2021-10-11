import BN from 'bn.js'
import {
  PublicKey,
  Balance,
  Hash,
  HalfKey,
  UINT256,
  Ticket,
  AcknowledgedTicket,
  ChannelEntry,
  UnacknowledgedTicket,
  Challenge,
  generateChannelId,
  ChannelStatus,
  PRICE_PER_PACKET,
  INVERSE_TICKET_WIN_PROB
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
    private readonly privateKey: Uint8Array,
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
        bumpCommitment(this.db, channelId)
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
    await this.chain.fundChannel(myAddress, counterpartyAddress, myFund, counterpartyFund)
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
    await this.chain.openChannel(myAddress, counterpartyAddress, fundAmount)
    return generateChannelId(myAddress, counterpartyAddress)
  }

  async initializeClosure() {
    const c = await this.usToThem()
    const counterpartyAddress = this.counterparty.toAddress()
    if (c.status !== ChannelStatus.Open && c.status !== ChannelStatus.WaitingForCommitment) {
      throw Error('Channel status is not OPEN or WAITING FOR COMMITMENT')
    }
    return await this.chain.initiateChannelClosure(counterpartyAddress)
  }

  async finalizeClosure() {
    const c = await this.usToThem()
    const counterpartyAddress = this.counterparty.toAddress()

    if (c.status !== ChannelStatus.PendingToClose) {
      throw Error('Channel status is not PENDING_TO_CLOSE')
    }
    return await this.chain.finalizeChannelClosure(counterpartyAddress)
  }

  private async bumpTicketIndex(channelId: Hash): Promise<UINT256> {
    let currentTicketIndex = await this.db.getCurrentTicketIndex(channelId)

    if (currentTicketIndex == undefined) {
      currentTicketIndex = new UINT256(new BN(1))
    }

    await this.db.setCurrentTicketIndex(channelId, new UINT256(currentTicketIndex.toBN().addn(1)))

    return currentTicketIndex
  }

  /**
   * Creates a signed ticket that includes the given amount of
   * tokens
   * @dev Due to a missing feature, namely ECMUL, in Ethereum, the
   * challenge is given as an Ethereum address because the signature
   * recovery algorithm is used to perform an EC-point multiplication.
   * @param amount value of the ticket
   * @param challenge challenge to solve in order to redeem the ticket
   * @param winProb the winning probability to use
   * @returns a signed ticket
   */
  async createTicket(pathLength: number, challenge: Challenge) {
    const counterpartyAddress = this.counterparty.toAddress()
    const channelState = await this.usToThem()
    const id = generateChannelId(this.self.toAddress(), this.counterparty.toAddress())
    const currentTicketIndex = await this.bumpTicketIndex(id)
    const amount = new Balance(PRICE_PER_PACKET.mul(INVERSE_TICKET_WIN_PROB).muln(pathLength - 1))
    const winProb = new BN(INVERSE_TICKET_WIN_PROB)

    const ticket = Ticket.create(
      counterpartyAddress,
      challenge,
      channelState.ticketEpoch,
      currentTicketIndex,
      amount,
      UINT256.fromInverseProbability(winProb),
      channelState.channelEpoch,
      this.privateKey
    )
    await this.db.markPending(ticket)

    log(`Creating ticket in channel ${chalk.yellow(channelState.getId().toHex())}. Ticket data: \n${ticket.toString()}`)

    return ticket
  }

  // @TODO Replace this with (truely) random data
  /**
   * Creates a ticket that is sent next to the packet to the last node.
   * @param challenge dummy challenge, potential no valid response known
   * @returns a ticket without any value
   */
  createDummyTicket(challenge: Challenge): Ticket {
    // TODO: document how dummy ticket works
    return Ticket.create(
      this.counterparty.toAddress(),
      challenge,
      UINT256.fromString('0'),
      UINT256.fromString('0'),
      new Balance(new BN(0)),
      UINT256.DUMMY_INVERSE_PROBABILITY,
      UINT256.fromString('0'),
      this.privateKey
    )
  }

  /*
   * As we issue probabilistic tickets, we can't be sure of the exact balance
   * of our channels, but we can see the bounds based on how many tickets are
   * outstanding.
   */
  async balanceToThem(): Promise<{ maximum: BN; minimum: BN }> {
    const stake = (await this.usToThem()).balance
    const outstandingTicketBalance = await this.db.getPendingBalanceTo(this.counterparty.toAddress())

    return {
      minimum: stake.toBN().sub(outstandingTicketBalance.toBN()),
      maximum: stake.toBN()
    }
  }

  async redeemAllTickets(): Promise<void> {
    // Because tickets are ordered and require the previous redemption to
    // have succeeded before we can redeem the next, we need to do this
    // sequentially.
    const tickets = await this.db.getAcknowledgedTickets({ signer: this.counterparty }) 
    for (const ticket of tickets) {
      log('redeeming ticket', ticket)
      this.redeemTicket(ticket)
      log('ticket was redeemed')
      // TODO handle failures due to foreseeable chain issues, gas etc.
    }
  }

  async redeemTicket(ackTicket: AcknowledgedTicket): Promise<RedeemTicketResponse> {
    if (!ackTicket.verify(this.counterparty)) {
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

      const receipt = await this.chain.redeemTicket(this.counterparty.toAddress(), ackTicket, ticket)

      //this.commitment.updateChainState(ackTicket.preImage)
      log('Successfully submitted ticket', ackTicket.response.toHex())
      await this.db.markRedeemeed(ackTicket)
      this.events.emit('ticket:redeemed', ackTicket)
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
}

export { Channel }
