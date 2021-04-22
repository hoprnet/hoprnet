import type Connector from '.'
import { ethers } from 'ethers'
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
} from './types'
import { computeWinningProbability, checkChallenge, isWinningTicket } from './utils'
import Debug from 'debug'
import type { SubmitTicketResponse } from '.'
import { Commitment } from './commitment'

const log = Debug('hopr-core-ethereum:channel')
const abiCoder = new ethers.utils.AbiCoder()

class Channel {
  private index: number
  private commitment: Commitment

  constructor(
    private readonly connector: Connector, // TODO: replace with ethereum global context?
    private readonly self: PublicKey,
    public readonly counterparty: PublicKey
  ) {
    this.index = 0 // TODO - bump channel epoch to make sure..
    this.commitment = new Commitment(() => {}, () => {}, this.connector.db, this.getId())
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
    acknowledgementHash: Hash
  ): Promise<Acknowledgement | null> {
    const response = Hash.create(unacknowledgedTicket.secretA.serialize(), acknowledgementHash.serialize())
    const ticket = unacknowledgedTicket.ticket
    if (
      await isWinningTicket(ticket.getHash(), response, await this.commitment.getCurrentCommitment(), ticket.winProb)
    ) {
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

  async getState(): Promise<ChannelEntry> {
    const channelId = this.getId()
    const state = await this.connector.indexer.getChannel(channelId)
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
    const { account, hoprToken, hoprChannels } = this.connector
    const myAddress = this.self.toAddress()
    const counterpartyAddress = this.counterparty.toAddress()
    const totalFund = myFund.toBN().add(counterpartyFund.toBN())
    const myBalance = await this.connector.hoprToken.balanceOf(myAddress.toHex())
    if (totalFund.gt(new BN(myBalance.toString()))) {
      throw Error('We do not have enough balance to fund the channel')
    }

    try {
      const transaction = await account.sendTransaction(
        hoprToken.send,
        hoprChannels.address,
        totalFund.toString(),
        abiCoder.encode(
          ['bool', 'address', 'address', 'uint256', 'uint256'],
          [
            false,
            myAddress.toHex(),
            counterpartyAddress.toHex(),
            myFund.toBN().toString(),
            counterpartyFund.toBN().toString()
          ]
        )
      )
      await transaction.wait()

      return transaction.hash
    } catch (err) {
      // TODO: catch race-condition
      console.log(err)
      throw Error(`Failed to fund channel`)
    }
  }

  async open(fundAmount: Balance) {
    const { account, hoprToken, hoprChannels } = this.connector
    // channel may not exist, we can still open it
    let c: ChannelEntry
    try {
      c = await this.getState()
    } catch {}
    if (c.status !== 'CLOSED') {
      throw Error('Channel is already opened')
    }

    const myAddress = this.self.toAddress()
    const counterpartyAddress = this.counterparty.toAddress()
    const myBalance = await this.connector.hoprToken.balanceOf(myAddress.toHex())
    if (new BN(myBalance.toString()).lt(fundAmount.toBN())) {
      throw Error('We do not have enough balance to open a channel')
    }

    try {
      const transaction = await account.sendTransaction(
        hoprToken.send,
        hoprChannels.address,
        fundAmount.toBN().toString(),
        abiCoder.encode(['bool', 'address', 'address'], [true, myAddress.toHex(), counterpartyAddress.toHex()])
      )
      await transaction.wait()

      return transaction.hash
    } catch (err) {
      // TODO: catch race-condition
      console.log(err)
      throw Error(`Failed to open channel`)
    }
  }

  async initializeClosure() {
    const { account, hoprChannels } = this.connector

    const c = await this.getState()
    const counterpartyAddress = this.counterparty.toAddress()

    if (c.status !== 'OPEN') {
      throw Error('Channel status is not OPEN')
    }

    try {
      const transaction = await account.sendTransaction(
        hoprChannels.initiateChannelClosure,
        counterpartyAddress.toHex()
      )
      await transaction.wait()

      return transaction.hash
    } catch (err) {
      // TODO: catch race-condition
      console.log(err)
      throw Error(`Failed to initialize channel closure`)
    }
  }

  async finalizeClosure() {
    const { account, hoprChannels } = this.connector

    const c = await this.getState()
    const counterpartyAddress = this.counterparty.toAddress()

    if (c.status !== 'PENDING_TO_CLOSE') {
      throw Error('Channel status is not PENDING_TO_CLOSE')
    }

    try {
      const transaction = await account.sendTransaction(
        hoprChannels.finalizeChannelClosure,
        counterpartyAddress.toHex()
      )
      await transaction.wait()

      return transaction.hash
    } catch (err) {
      // TODO: catch race-condition
      console.log(err)
      throw Error(`Failed to finilize channel closure`)
    }
  }

  async createTicket(amount: Balance, challenge: Hash, winProb: number) {
    const counterpartyAddress = this.counterparty.toAddress()
    const counterpartyState = await this.connector.indexer.getAccount(counterpartyAddress)
    return Ticket.create(
      counterpartyAddress,
      challenge,
      new UINT256(counterpartyState.counter),
      new UINT256(new BN(this.index++)),
      amount,
      computeWinningProbability(winProb),
      (await this.getState()).channelEpoch,
      this.connector.account.privateKey
    )
  }

  async createDummyTicket(challenge: Hash): Promise<Ticket> {
    // TODO: document how dummy ticket works
    return Ticket.create(
      this.counterparty.toAddress(),
      challenge,
      UINT256.fromString('0'),
      new UINT256(new BN(this.index++)),
      new Balance(new BN(0)),
      computeWinningProbability(1),
      UINT256.fromString('0'),
      this.connector.account.privateKey
    )
  }

  async submitTicket(ackTicket: Acknowledgement): Promise<SubmitTicketResponse> {
    try {
      const ticket = ackTicket.ticket

      log('Submitting ticket', ackTicket.response.toHex())
      const { hoprChannels, account } = this.connector

      const emptyPreImage = new Hash(new Uint8Array(Hash.SIZE).fill(0x00))
      const hasPreImage = !ackTicket.preImage.eq(emptyPreImage)
      if (!hasPreImage) {
        log(`Failed to submit ticket ${ackTicket.response.toHex()}: 'PreImage is empty.'`)
        return {
          status: 'FAILURE',
          message: 'PreImage is empty.'
        }
      }

      const validChallenge = await checkChallenge(ticket.challenge, ackTicket.response)
      if (!validChallenge) {
        log(`Failed to submit ticket ${ackTicket.response.toHex()}: 'Invalid challenge.'`)
        return {
          status: 'FAILURE',
          message: 'Invalid challenge.'
        }
      }

      const isWinning = await isWinningTicket(ticket.getHash(), ackTicket.response, ackTicket.preImage, ticket.winProb)
      if (!isWinning) {
        log(`Failed to submit ticket ${ackTicket.response.toHex()}:  'Not a winning ticket.'`)
        return {
          status: 'FAILURE',
          message: 'Not a winning ticket.'
        }
      }

      const counterparty = ticket.getSigner().toAddress()
      const transaction = await account.sendTransaction(
        hoprChannels.redeemTicket,
        counterparty.toHex(),
        ackTicket.preImage.toHex(),
        ackTicket.ticket.epoch.serialize(),
        ackTicket.ticket.index.serialize(),
        ackTicket.response.toHex(),
        ticket.amount.toBN().toString(),
        ticket.winProb.toHex(),
        ticket.signature.serialize()
      )
      await transaction.wait()
      // TODO delete ackTicket
      this.connector.account.updateLocalState(ackTicket.preImage)

      log('Successfully submitted ticket', ackTicket.response.toHex())
      return {
        status: 'SUCCESS',
        receipt: transaction.hash,
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

export default Channel
