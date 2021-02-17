import { u8aIsEmpty, u8aToHex } from '@hoprnet/hopr-utils'
import { AcknowledgedTicket } from '../types'
import { checkChallenge } from '../utils'
import type HoprEthereum from '..'
import debug from 'debug'
const log = debug('hopr-core-ethereum:ticket')

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
      const { hoprChannels, account, utils } = this.coreConnector
      const { r, s, v } = utils.getSignatureParameters(signedTicket.signature)

      if (u8aIsEmpty(ackTicket.preImage)) {
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

      const isWinning = await this.coreConnector.probabilisticPayments.validateTicket(ackTicket)
      if (!isWinning) {
        log(`Failed to submit ticket ${u8aToHex(ticketChallenge)}: ${this.INVALID_MESSAGES.NOT_WINNING}`)
        return {
          status: 'FAILURE',
          message: this.INVALID_MESSAGES.NOT_WINNING
        }
      }

      const counterparty = await this.coreConnector.utils.pubKeyToAccountId(await signedTicket.signer)

      const transaction = await account.signTransaction(
        {
          from: (await account.address).toHex(),
          to: hoprChannels.options.address
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
      //ackTicket.redeemed = true

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

export { TicketStatic }
