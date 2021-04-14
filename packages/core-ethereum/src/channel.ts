import { ethers } from 'ethers'
import BN from 'bn.js'
import { PublicKey, Balance, Hash, UINT256, Ticket, Acknowledgement, Channel, Address } from './types'
import { computeWinningProbability, checkChallenge, isWinningTicket, getSignatureParameters } from './utils'
import Debug from 'debug'
import type { SubmitTicketResponse } from '.'
import type Indexer from './indexer'

const log = Debug('hopr-core-ethereum:channel')
const abiCoder = new ethers.utils.AbiCoder()

export class ChannelManager {
  constructor(
    private indexer: Indexer,
    private getBalance: (a: Address) => Promise<Balance>
  ) {}

  async getChannel(self: PublicKey, counterparty: PublicKey): Promise<Channel> {
    const id = Channel.generateId(self.toAddress(), counterparty.toAddress())
    const channel = await this.indexer.getChannel(id)
    if (!channel) {
      throw Error(`Channel for ${id.toHex()} not found`)
    }
    return channel
  }

  async channelExists(self: PublicKey, counterparty: PublicKey): Promise<boolean> {
    const id = Channel.generateId(self.toAddress(), counterparty.toAddress())
    const channel = await this.indexer.getChannel(id)
    return channel && channel.getStatus() !== 'CLOSED'
  }

  async open(self: PublicKey, counterparty: PublicKey, fundAmount: Balance) {
    if (this.channelExists(self, counterparty)) {
      throw Error('Channel is already opened')
    }
    const myBalance = await this.getBalance(self.toAddress())
    if (new BN(myBalance.toString()).lt(fundAmount.toBN())) {
      throw Error('We do not have enough balance to open a channel')
    }

    try {
      const transaction = await account.sendTransaction(
        hoprToken.send,
        hoprChannels.address,
        fundAmount.toBN().toString(),
        abiCoder.encode(['bool', 'address', 'address'], [true, self.toAddress().toHex(), counterparty.toAddress().toHex()])
      )
      await transaction.wait()

      return transaction.hash
    } catch (err) {
      // TODO: catch race-condition
      console.log(err)
      throw Error(`Failed to open channel`)
    }
  }

  async initializeClosure(self: PublicKey, counterparty: PublicKey) {
    const channel = await this.getChannel(self, counterparty)
    if (channel.getStatus() !== 'OPEN') {
      throw Error('Channel status is not OPEN')
    }

    try {
      const transaction = await account.sendTransaction(
        hoprChannels.initiateChannelClosure,
        counterparty.toAddress().toHex()
      )
      await transaction.wait()

      return transaction.hash
    } catch (err) {
      // TODO: catch race-condition
      console.log(err)
      throw Error(`Failed to initialize channel closure`)
    }
  }

  async finalizeClosure(self: PublicKey, counterparty: PublicKey) {
    const channel = await this.getChannel(self, counterparty)
    if (channel.getStatus() !== 'PENDING_TO_CLOSE') {
      throw Error('Channel status is not PENDING_TO_CLOSE')
    }

    try {
      const transaction = await account.sendTransaction(
        hoprChannels.finalizeChannelClosure,
        counterparty.toAddress().toHex()
      )
      await transaction.wait()

      return transaction.hash
    } catch (err) {
      // TODO: catch race-condition
      console.log(err)
      throw Error(`Failed to finilize channel closure`)
    }
  }


  async redeemTicket(ackTicket: Acknowledgement): Promise<SubmitTicketResponse> {
    try {
      const ticket = ackTicket.ticket

      log('Redeeming ticket', ackTicket.response.toHex())
      const { r, s, v } = getSignatureParameters(ticket.signature)

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
        ackTicket.response.toHex(),
        ticket.amount.toBN().toString(),
        ticket.winProb.toHex(),
        r.toHex(),
        s.toHex(),
        v + 27
      )
      await transaction.wait()
      // TODO delete ackTicket
      //this.connector.account.updateLocalState(ackTicket.preImage)

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

  /*
  private async initPreimage() {
    if (!this.preimage) {
      const ocs = await this.getOnChainSecret()
      if (!ocs) {
        throw new Error('cannot reserve preimage when there is no on chain secret')
      }
      this.preimage = await this.coreConnector.hashedSecret.findPreImage(ocs)
    }
  }

  /**
   * Reserve a preImage for the given ticket if it is a winning ticket.
   * @param ticket the acknowledged ticket
  async acknowledge(
    unacknowledgedTicket: UnacknowledgedTicket,
    acknowledgementHash: Hash
  ): Promise<Acknowledgement | null> {
    await this.initPreimage()
    const response = Hash.create(u8aConcat(unacknowledgedTicket.secretA.serialize(), acknowledgementHash.serialize()))
    const ticket = unacknowledgedTicket.ticket
    if (await isWinningTicket(ticket.getHash(), response, this.preimage, ticket.winProb)) {
      const ack = new Acknowledgement(ticket, response, this.preimage)
      this.preimage = await this.coreConnector.hashedSecret.findPreImage(this.preimage)
      return ack
    } else {
      return null
    }
  }
  */

  async createTicket(amount: Balance, challenge: Hash, winProb: number) {
    const counterpartyAddress = this.counterparty
    return Ticket.create(
      counterpartyAddress,
      challenge,
      new UINT256(counterpartyState.counter),
      amount,
      computeWinningProbability(winProb),
      new UINT256((await this.getState()).getIteration()),
      this.connector.account.privateKey
    )
  }

  async createDummyTicket(challenge: Hash): Promise<Ticket> {
    // TODO: document how dummy ticket works
    return Ticket.create(
      this.counterparty.toAddress(),
      challenge,
      UINT256.fromString('0'),
      new Balance(new BN(0)),
      computeWinningProbability(1),
      UINT256.fromString('0'),
      this.connector.account.privateKey
    )
  }
}
