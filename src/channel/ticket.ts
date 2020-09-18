import type IChannel from '.'
import { u8aEquals, u8aToHex } from '@hoprnet/hopr-utils'
import { Hash, TicketEpoch, Balance, SignedTicket, Ticket, AcknowledgedTicket } from '../types'
import { pubKeyToAccountId, computeWinningProbability, isWinningTicket, checkChallenge } from '../utils'
import assert from 'assert'
import type HoprEthereum from '..'
import { HASHED_SECRET_WIDTH } from '../hashedSecret'

const DEFAULT_WIN_PROB = 1

const EMPTY_PRE_IMAGE = new Uint8Array(HASHED_SECRET_WIDTH).fill(0x00)

class TicketStatic {
  constructor(public coreConnector: HoprEthereum) {}

  async submit(ticket: AcknowledgedTicket): Promise<void> {
    const { hoprChannels, signTransaction, account, utils } = this.coreConnector
    const { r, s, v } = utils.getSignatureParameters((await ticket.signedTicket).signature)

    assert(
      await checkChallenge((await ticket.signedTicket).ticket.challenge, ticket.response),
      'checks that the given response fulfills the challenge that has been signed by counterparty'
    )

    if (u8aEquals(ticket.preImage, EMPTY_PRE_IMAGE)) {
      throw Error(`PreImage is empty. Please set the preImage before submitting.`)
    }
    assert(
      await isWinningTicket(
        await (await ticket.signedTicket).ticket.hash,
        ticket.response,
        ticket.preImage,
        (await ticket.signedTicket).ticket.winProb
      )
    )

    const transaction = await signTransaction(
      hoprChannels.methods.redeemTicket(
        u8aToHex(ticket.preImage),
        u8aToHex(ticket.response),
        (await ticket.signedTicket).ticket.amount.toString(),
        u8aToHex((await ticket.signedTicket).ticket.winProb),
        u8aToHex(r),
        u8aToHex(s),
        v + 27
      ),
      {
        from: (await account.address).toHex(),
        to: hoprChannels.options.address,
        nonce: (await account.nonce).valueOf(),
      }
    )

    ticket.redeemed = true

    await transaction.send()

    this.coreConnector.account.updateLocalState(ticket.preImage)
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

    const signedTicket = new SignedTicket(arr)

    const ticket = new Ticket(
      {
        bytes: signedTicket.buffer,
        offset: signedTicket.ticketOffset,
      },
      {
        counterparty,
        challenge,
        epoch,
        amount,
        winProb: ticketWinProb,
      }
    )

    await ticket.sign(this.channel.coreConnector.account.keys.onChain.privKey, undefined, {
      bytes: signedTicket.buffer,
      offset: signedTicket.signatureOffset,
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
