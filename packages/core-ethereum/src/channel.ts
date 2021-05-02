import BN from 'bn.js'
import {
  PublicKey,
  Address,
  Balance,
  Hash,
  UINT256,
  Ticket,
  Acknowledgement,
  ChannelEntry,
  UnacknowledgedTicket
} from '@hoprnet/hopr-utils'
import Debug from 'debug'
import type { SubmitTicketResponse } from '.'
import { Commitment } from './commitment'
import type { ChainWrapper } from './ethereum'
import type Indexer from './indexer'
import type { HoprDB } from '@hoprnet/hopr-utils'

const log = Debug('hopr-core-ethereum:channel')

class Channel {
  private index: number
  private commitment: Commitment

  constructor(
    private readonly self: PublicKey,
    private readonly counterparty: PublicKey,
    private readonly db: HoprDB,
    private readonly chain: ChainWrapper,
    private readonly indexer: Indexer,
    private readonly privateKey: Uint8Array
  ) {
    this.index = 0 // TODO - bump channel epoch to make sure..
    this.commitment = new Commitment(
      (commitment: Hash) => this.chain.setCommitment(commitment),
      () => this.getChainCommitment(),
      this.db,
      this.getId()
    )
  }

  static generateId(self: Address, counterparty: Address) {
    let parties = self.sortPair(counterparty)
    return Hash.create(Buffer.concat(parties.map((x) => x.serialize())))
  }

  /**
   * Reserve a preImage for the given ticket if it is a winning ticket.
   * @param ticket the acknowledged ticket
   */
  async acknowledge(
    unacknowledgedTicket: UnacknowledgedTicket,
    acknowledgement: Hash
  ): Promise<Acknowledgement | null> {
    const validateResult = unacknowledgedTicket.verify(this.counterparty, acknowledgement)

    if (validateResult.valid == false) {
      throw Error(`Ticket invalid`)
    }

    const response = new Hash(validateResult.response)

    const ticket = unacknowledgedTicket.ticket
    if (ticket.isWinningTicket(response, await this.commitment.getCurrentCommitment(), ticket.winProb)) {
      const ack = new Acknowledgement(ticket, response, await this.commitment.getCurrentCommitment())
      await this.commitment.bumpCommitment()
      return ack
    } else {
      return null
    }
  }

  getId() {
    return Channel.generateId(this.self.toAddress(), this.counterparty.toAddress())
  }

  async getChainCommitment(): Promise<Hash> {
    return (await this.getState()).commitmentFor(this.self.toAddress())
  }

  async getState(): Promise<ChannelEntry> {
    const channelId = this.getId()
    const state = await this.indexer.getChannel(channelId)
    if (state) return state

    throw Error(`Channel state for ${channelId.toHex()} not found`)
  }

  // TODO kill
  async getBalances() {
    const state = await this.getState()
    const a = state.partyABalance
    const b = state.partyBBalance
    const [self, counterparty] = state.partyA.eq(this.self.toAddress()) ? [a, b] : [b, a]

    return {
      self,
      counterparty
    }
  }

  async fund(myFund: Balance, counterpartyFund: Balance) {
    const myAddress = this.self.toAddress()
    const counterpartyAddress = this.counterparty.toAddress()
    const totalFund = myFund.toBN().add(counterpartyFund.toBN())
    const myBalance = await this.chain.getBalance(myAddress)
    if (totalFund.gt(new BN(myBalance.toString()))) {
      throw Error('We do not have enough balance to fund the channel')
    }
    await this.chain.fundChannel(myAddress, counterpartyAddress, myFund, counterpartyFund)
  }

  async open(fundAmount: Balance) {
    // channel may not exist, we can still open it
    let c: ChannelEntry
    try {
      c = await this.getState()
    } catch {}
    if (c && c.status !== 'CLOSED') {
      throw Error('Channel is already opened')
    }

    const myAddress = this.self.toAddress()
    const counterpartyAddress = this.counterparty.toAddress()
    const myBalance = await this.chain.getBalance(myAddress)
    if (new BN(myBalance.toString()).lt(fundAmount.toBN())) {
      throw Error('We do not have enough balance to open a channel')
    }
    await this.chain.openChannel(myAddress, counterpartyAddress, fundAmount)
  }

  async initializeClosure() {
    const c = await this.getState()
    const counterpartyAddress = this.counterparty.toAddress()
    if (c.status !== 'OPEN') {
      throw Error('Channel status is not OPEN')
    }
    return await this.chain.initiateChannelClosure(counterpartyAddress)
  }

  async finalizeClosure() {
    const c = await this.getState()
    const counterpartyAddress = this.counterparty.toAddress()

    if (c.status !== 'PENDING_TO_CLOSE') {
      throw Error('Channel status is not PENDING_TO_CLOSE')
    }
    return await this.chain.finalizeChannelClosure(counterpartyAddress)
  }

  async createTicket(amount: Balance, challenge: Address, winProb: number) {
    const counterpartyAddress = this.counterparty.toAddress()
    const c = await this.getState()
    return Ticket.create(
      counterpartyAddress,
      challenge,
      c.ticketEpochFor(this.counterparty.toAddress()),
      new UINT256(new BN(this.index++)),
      amount,
      Ticket.fromProbability(winProb),
      (await this.getState()).channelEpoch,
      this.privateKey
    )
  }

  createDummyTicket(challenge: Address): Ticket {
    // TODO: document how dummy ticket works
    return Ticket.create(
      this.counterparty.toAddress(),
      challenge,
      UINT256.fromString('0'),
      new UINT256(new BN(this.index++)),
      new Balance(new BN(0)),
      Ticket.fromProbability(1),
      UINT256.fromString('0'),
      this.privateKey
    )
  }

  async submitTicket(ackTicket: Acknowledgement): Promise<SubmitTicketResponse> {
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

      const isWinning = ticket.isWinningTicket(ackTicket.response, ackTicket.preImage, ticket.winProb)

      if (!isWinning) {
        log(`Failed to submit ticket ${ackTicket.response.toHex()}:  'Not a winning ticket.'`)
        return {
          status: 'FAILURE',
          message: 'Not a winning ticket.'
        }
      }

      const receipt = await this.chain.redeemTicket(this.counterparty.toAddress().toHex(), ackTicket, ticket)

      // TODO delete ackTicket
      //this.commitment.updateChainState(ackTicket.preImage)

      log('Successfully submitted ticket', ackTicket.response.toHex())
      return {
        status: 'SUCCESS',
        receipt,
        ackTicket
      }
    } catch (err) {
      log('Unexpected error when submitting ticket', ackTicket.response.toHex(), err)
      return {
        status: 'ERROR',
        error: err
      }
    }
  }
}

export { Channel }
