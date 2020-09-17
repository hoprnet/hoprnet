import type IChannel from '.'
import { u8aToHex } from '@hoprnet/hopr-utils'
import { Hash, TicketEpoch, Balance, SignedTicket, Ticket, AcknowledgedTicket } from '../types'
import { pubKeyToAccountId, computeWinningProbability, isWinningTicket, checkChallenge } from '../utils'
import assert from 'assert'
import type HoprEthereum from '..'

const DEFAULT_WIN_PROB = 1

class TicketStatic {
  constructor(public coreConnector: HoprEthereum) {}

  async submit(ticket: AcknowledgedTicket): Promise<void> {
    const { hoprChannels, signTransaction, account, utils } = this.coreConnector
    const { r, s, v } = utils.getSignatureParameters((await ticket.signedTicket).signature)

    assert(
      await checkChallenge((await ticket.signedTicket).ticket.challenge, ticket.response),
      'checks that the given response fulfills the challenge that has been signed by counterparty'
    )

    const onChainSecret = await this.coreConnector.account.onChainSecret

    const preImage = (await this.coreConnector.hashedSecret.findPreImage(onChainSecret)).preImage

    assert(
      await isWinningTicket(
        await (await ticket.signedTicket).ticket.hash,
        ticket.response,
        preImage,
        (await ticket.signedTicket).ticket.winProb
      )
    )

    const transaction = await signTransaction(
      hoprChannels.methods.redeemTicket(
        u8aToHex(preImage),
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

    this.coreConnector.account.updateLocalState(preImage)
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
