import type IChannel from '.'
import { u8aEquals, u8aToHex } from '@hoprnet/hopr-utils'
import { Hash, TicketEpoch, Balance, SignedTicket, Ticket, AcknowledgedTicket } from '../types'
import {
  pubKeyToAccountId,
  computeWinningProbability,
  isWinningTicket,
  checkChallenge,
  stateCounterToIteration
} from '../utils'
import type HoprEthereum from '..'
import { HASHED_SECRET_WIDTH } from '../hashedSecret'
import debug from 'debug'
const log = debug('hopr-core-ethereum:ticket')

const DEFAULT_WIN_PROB = 1
const EMPTY_PRE_IMAGE = new Uint8Array(HASHED_SECRET_WIDTH).fill(0x00)

class TicketStatic {
  private readonly INVALID_MESSAGES = {
    NO_PRE_IMAGE: 'PreImage is empty.',
    INVALID_CHALLENGE: 'Invalid challenge.',
    NOT_WINNING: 'Not a winning ticket.'
  }

  constructor(public coreConnector: HoprEthereum) {}

  public async submit(
    ackTicket: AcknowledgedTicket,
    _ticketIndex: Uint8Array
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
      const { hoprChannels, signTransaction, account, utils } = this.coreConnector
      const { r, s, v } = utils.getSignatureParameters(signedTicket.signature)

      const hasPreImage = !u8aEquals(ackTicket.preImage, EMPTY_PRE_IMAGE)
      if (!hasPreImage) {
        log(`Failed to submit ticket ${u8aToHex(ticketChallenge)}: ${this.INVALID_MESSAGES.NO_PRE_IMAGE}`)
        return {
          status: 'FAILURE',
          message: this.INVALID_MESSAGES.NO_PRE_IMAGE
        }
      }

      const validChallenge = await checkChallenge(ticket.challenge, ackTicket.response)
      if (!validChallenge) {
        log(`Failed to submit ticket ${u8aToHex(ticketChallenge)}: ${this.INVALID_MESSAGES.INVALID_CHALLENGE}`)
        return {
          status: 'FAILURE',
          message: this.INVALID_MESSAGES.INVALID_CHALLENGE
        }
      }

      const isWinning = await isWinningTicket(await ticket.hash, ackTicket.response, ackTicket.preImage, ticket.winProb)
      if (!isWinning) {
        log(`Failed to submit ticket ${u8aToHex(ticketChallenge)}: ${this.INVALID_MESSAGES.NOT_WINNING}`)
        return {
          status: 'FAILURE',
          message: this.INVALID_MESSAGES.NOT_WINNING
        }
      }

      const counterparty = await this.coreConnector.utils.pubKeyToAccountId(await signedTicket.signer)

      const transaction = await signTransaction(
        {
          from: (await account.address).toHex(),
          to: hoprChannels.options.address,
          nonce: (await account.nonce).valueOf()
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
      ackTicket.redeemed = true
      this.coreConnector.account.updateLocalState(ackTicket.preImage)

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

class TicketFactory {
  constructor(public channel: IChannel) {}

  async create(
    amount: Balance,
    challenge: Hash,
    winProb: number = DEFAULT_WIN_PROB,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    }
  ): Promise<SignedTicket> {
    const ticketWinProb = new Hash(computeWinningProbability(winProb))

    const counterparty = await pubKeyToAccountId(this.channel.counterparty)

    const epoch = await this.channel.coreConnector.hoprChannels.methods
      .accounts(counterparty.toHex())
      .call()
      .then((res) => new TicketEpoch(Number(res.counter)))

    const channelIteration = new TicketEpoch(stateCounterToIteration((await this.channel.stateCounter).toNumber()))

    const signedTicket = new SignedTicket(arr)

    const ticket = new Ticket(
      {
        bytes: signedTicket.buffer,
        offset: signedTicket.ticketOffset
      },
      {
        counterparty,
        challenge,
        epoch,
        amount,
        winProb: ticketWinProb,
        channelIteration
      }
    )

    await ticket.sign(this.channel.coreConnector.account.keys.onChain.privKey, undefined, {
      bytes: signedTicket.buffer,
      offset: signedTicket.signatureOffset
    })

    return signedTicket
  }

  async verify(signedTicket: SignedTicket): Promise<boolean> {
    // @TODO: check if this is needed
    // if ((await channel.currentBalanceOfCounterparty).add(signedTicket.ticket.amount).lt(await channel.balance)) {
    //   return false
    // }

    try {
      await this.channel.testAndSetNonce(signedTicket)
    } catch {
      return false
    }

    return await signedTicket.verify(await this.channel.offChainCounterparty)
  }
}

export { TicketStatic }
export default TicketFactory
