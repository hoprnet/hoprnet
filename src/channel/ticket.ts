import type IChannel from '.'
import BN from 'bn.js'
import { u8aToHex, u8aXOR } from '@hoprnet/hopr-utils'
import { Hash, TicketEpoch, Balance, SignedTicket, Ticket } from '../types'

const DEFAULT_WIN_PROB = new BN(1)

class TicketFactory {
  constructor(public channel: IChannel) {}

  async create(
    amount: Balance,
    challenge: Hash,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    }
  ): Promise<SignedTicket> {
    const winProb = new Hash(
      new BN(new Uint8Array(Hash.SIZE).fill(0xff)).div(DEFAULT_WIN_PROB).toArray('le', Hash.SIZE)
    )
    const channelId = await this.channel.channelId

    const signedTicket = new SignedTicket(arr)

    const ticket = new Ticket(
      {
        bytes: signedTicket.buffer,
        offset: signedTicket.ticketOffset,
      },
      {
        channelId,
        challenge,
        epoch: new TicketEpoch(1), // @TODO: set this dynamically
        amount: new Balance(amount.toString()),
        winProb,
        onChainSecret: new Hash(), // @TODO: set pre_image
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

  async submit(signedTicket: SignedTicket): Promise<void> {
    const { hoprChannels, signTransaction, account, utils } = this.channel.coreConnector
    const { ticket, signature } = signedTicket
    const { r, s, v } = utils.getSignatureParameters(signature)
    const counterPartySecret = u8aXOR(false, ticket.challenge, ticket.onChainSecret)

    const transaction = await signTransaction(
      hoprChannels.methods.redeemTicket(
        u8aToHex(ticket.challenge),
        u8aToHex(ticket.onChainSecret),
        u8aToHex(counterPartySecret),
        ticket.amount.toString(),
        u8aToHex(ticket.winProb),
        u8aToHex(r),
        u8aToHex(s),
        v
      ),
      {
        from: (await account.address).toHex(),
        to: hoprChannels.options.address,
        nonce: (await account.nonce).valueOf(),
      }
    )

    const receipt = await transaction.send()
    console.log(receipt)
  }
}

export default TicketFactory
