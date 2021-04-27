import type Connector from '.'
import { ethers } from 'ethers'
import BN from 'bn.js'
import { PublicKey, Address, Balance, Hash, UINT256, Ticket, Acknowledgement, ChannelEntry } from './types'
import { computeWinningProbability, isWinningTicket, getSignatureParameters } from './utils'
import Debug from 'debug'
import type { SubmitTicketResponse } from '.'

const log = Debug('hopr-core-ethereum:channel')
const abiCoder = new ethers.utils.AbiCoder()

class Channel {
  constructor(
    private readonly connector: Connector, // TODO: replace with ethereum global context?
    private readonly self: PublicKey,
    public readonly counterparty: PublicKey
  ) {}

  static generateId(self: Address, counterparty: Address) {
    let parties = self.sortPair(counterparty)
    return Hash.create(Buffer.concat(parties.map((x) => x.serialize())))
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

  async getBalances() {
    const state = await this.getState()
    const { partyA, partyB } = state.getBalances()
    const [self, counterparty] = state.partyA.eq(this.self.toAddress()) ? [partyA, partyB] : [partyB, partyA]

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
      throw Error(`Failed to fund channel`)
    }
  }

  async open(fundAmount: Balance) {
    const { account, hoprToken, hoprChannels } = this.connector

    // check if we have initialized account, initialize if we didnt
    await this.connector.initOnchainValues()

    // channel may not exist, we can still open it
    let state: ChannelEntry
    try {
      state = await this.getState()
    } catch {}
    if (state && state.getStatus() !== 'CLOSED') {
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

    const state = await this.getState()
    const counterpartyAddress = this.counterparty.toAddress()

    if (state.getStatus() !== 'OPEN') {
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

    const state = await this.getState()
    const counterpartyAddress = this.counterparty.toAddress()

    if (state.getStatus() !== 'PENDING_TO_CLOSE') {
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

  async createTicket(amount: Balance, challenge: PublicKey, winProb: number) {
    const counterpartyAddress = this.counterparty.toAddress()
    const counterpartyState = await this.connector.indexer.getAccount(counterpartyAddress)
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

  createDummyTicket(challenge: PublicKey): Ticket {
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
      const { hoprChannels, account } = this.connector
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

      const isWinning = await isWinningTicket(ticket.getHash(), ackTicket.response, ackTicket.preImage, ticket.winProb)
      if (!isWinning) {
        log(`Failed to submit ticket ${ackTicket.response.toHex()}:  'Not a winning ticket.'`)
        return {
          status: 'FAILURE',
          message: 'Not a winning ticket.'
        }
      }

      const transaction = await account.sendTransaction(
        hoprChannels.redeemTicket,
        this.counterparty.toAddress().toHex(),
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
