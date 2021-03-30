import type IChannel from '.'
import { u8aToHex } from '@hoprnet/hopr-utils'
import { Hash, Balance, SignedTicket, Ticket, AcknowledgedTicket, UINT256 } from '../types'
import {
  pubKeyToAddress,
  computeWinningProbability,
  isWinningTicket,
  checkChallenge,
  stateCounterToIteration
} from '../utils'
import type HoprEthereum from '..'
import debug from 'debug'
const log = debug('hopr-core-ethereum:ticket')

const DEFAULT_WIN_PROB = 1
const EMPTY_PRE_IMAGE = new Hash(new Uint8Array(Hash.SIZE).fill(0x00))

class TicketStatic {
  constructor(public coreConnector: HoprEthereum) {}

  public async submit(
    ackTicket: AcknowledgedTicket,
    _ticketIndex: Uint8Array
  ): Promise<
    | {
        status: 'SUCCESS'
        receipt: string
        ackTicket: AcknowledgedTicket
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

      log('Submitting ticket', ticketChallenge.toHex())
      const { hoprChannels, account, utils } = this.coreConnector
      const { r, s, v } = utils.getSignatureParameters(signedTicket.signature)

      const hasPreImage = !ackTicket.preImage.eq(EMPTY_PRE_IMAGE)
      if (!hasPreImage) {
        log(`Failed to submit ticket ${ticketChallenge.toHex()}: 'PreImage is empty.'`)
        return {
          status: 'FAILURE',
          message:'PreImage is empty.' 
        }
      }

      console.log(ticket.challenge, ackTicket.response)
      const validChallenge = await checkChallenge(ticket.challenge, ackTicket.response)
      if (!validChallenge) {
        log(`Failed to submit ticket ${ticketChallenge.toHex()}: 'Invalid challenge.'`)
        return {
          status: 'FAILURE',
          message:'Invalid challenge.' 
        }
      }

      const isWinning = await isWinningTicket(await ticket.hash, ackTicket.response, ackTicket.preImage, ticket.winProb)
      if (!isWinning) {
        log(`Failed to submit ticket ${ticketChallenge.toHex()}:  'Not a winning ticket.'`)
        return {
          status: 'FAILURE',
          message: 'Not a winning ticket.' 
        }
      }

      const counterparty = await this.coreConnector.utils.pubKeyToAddress(await signedTicket.signer)

      const transaction = await account.signTransaction(
        {
          from: (await account.address).toHex(),
          to: hoprChannels.options.address
        },
        hoprChannels.methods.redeemTicket(
          counterparty.toHex(),
          ackTicket.preImage.toHex(),
          ackTicket.response.toHex(),
          ticket.amount.toBN().toString(),
          ticket.winProb.toHex(),
          u8aToHex(r),
          u8aToHex(s),
          v + 27
        )
      )

      await transaction.send()
      ackTicket.redeemed = true
      this.coreConnector.account.updateLocalState(ackTicket.preImage)

      log('Successfully submitted ticket', ticketChallenge.toHex())
      return {
        status: 'SUCCESS',
        receipt: transaction.transactionHash,
        ackTicket
      }
    } catch (err) {
      log('Unexpected error when submitting ticket', ticketChallenge.toHex(), err)
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
    const ticketWinProb = computeWinningProbability(winProb)

    const counterparty = await pubKeyToAddress(this.channel.counterparty)

    const epoch = await this.channel.coreConnector.hoprChannels.methods
      .accounts(counterparty.toHex())
      .call()
      .then((res) => UINT256.fromString(res.counter))

    // TODO: wtf, make stateCounterToIteration accept BN
    const channelIteration = UINT256.fromString(
      String(stateCounterToIteration((await this.channel.stateCounter).toBN().toNumber()))
    )

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

    const signature = await ticket.sign(this.channel.coreConnector.account.keys.onChain.privKey)
    signedTicket.set(signature, signedTicket.signatureOffset - signedTicket.byteOffset)

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
