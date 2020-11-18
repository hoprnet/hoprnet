import type IChannel from '.'
import { u8aEquals, u8aToHex } from '@hoprnet/hopr-utils'
import { Hash, TicketEpoch, Balance, SignedTicket, Ticket, AcknowledgedTicket } from '../types'
import { pubKeyToAccountId, computeWinningProbability, isWinningTicket, checkChallenge } from '../utils'
import type HoprEthereum from '..'
import { HASHED_SECRET_WIDTH } from '../hashedSecret'

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
    ticket: AcknowledgedTicket,
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
    const ticketChallenge = ticket.response

    try {
      this.coreConnector.log('Submitting ticket', u8aToHex(ticketChallenge))

      const { hoprChannels, signTransaction, account, utils } = this.coreConnector
      const { r, s, v } = utils.getSignatureParameters((await ticket.signedTicket).signature)

      const hasPreImage = !u8aEquals(ticket.preImage, EMPTY_PRE_IMAGE)
      if (!hasPreImage) {
        this.coreConnector.log(
          `Failed to submit ticket ${u8aToHex(ticketChallenge)}: ${this.INVALID_MESSAGES.NO_PRE_IMAGE}`
        )
        return {
          status: 'FAILURE',
          message: this.INVALID_MESSAGES.NO_PRE_IMAGE
        }
      }

      const validChallenge = await checkChallenge((await ticket.signedTicket).ticket.challenge, ticket.response)
      if (!validChallenge) {
        this.coreConnector.log(
          `Failed to submit ticket ${u8aToHex(ticketChallenge)}: ${this.INVALID_MESSAGES.INVALID_CHALLENGE}`
        )
        return {
          status: 'FAILURE',
          message: this.INVALID_MESSAGES.INVALID_CHALLENGE
        }
      }

      const isWinning = await isWinningTicket(
        await (await ticket.signedTicket).ticket.hash,
        ticket.response,
        ticket.preImage,
        (await ticket.signedTicket).ticket.winProb
      )
      if (!isWinning) {
        this.coreConnector.log(
          `Failed to submit ticket ${u8aToHex(ticketChallenge)}: ${this.INVALID_MESSAGES.NOT_WINNING}`
        )
        return {
          status: 'FAILURE',
          message: this.INVALID_MESSAGES.NOT_WINNING
        }
      }

      const transaction = await signTransaction(
        {
          from: (await account.address).toHex(),
          to: hoprChannels.options.address,
          nonce: (await account.nonce).valueOf()
        },
        hoprChannels.methods.redeemTicket(
          u8aToHex(ticket.preImage),
          u8aToHex(ticket.response),
          (await ticket.signedTicket).ticket.amount.toString(),
          u8aToHex((await ticket.signedTicket).ticket.winProb),
          u8aToHex(r),
          u8aToHex(s),
          v + 27
        )
      )

      await transaction.send()
      ticket.redeemed = true
      this.coreConnector.account.updateLocalState(ticket.preImage)

      this.coreConnector.log('Successfully submitted ticket', u8aToHex(ticketChallenge))
      return {
        status: 'SUCCESS',
        receipt: transaction.transactionHash
      }
    } catch (err) {
      this.coreConnector.log('Unexpected error when submitting ticket', u8aToHex(ticketChallenge), err)
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

    const channelStateCounter = await this.channel.stateCounter

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
        channelStateCounter
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
