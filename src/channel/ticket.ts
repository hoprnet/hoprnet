import type IChannel from '.'
import BN from 'bn.js'
import { u8aToHex, stringToU8a } from '@hoprnet/hopr-utils'
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
    const counterParty = await this.channel.coreConnector.utils
      .pubKeyToAccountId(this.channel.counterparty)
      .then((res) => res.toHex())

    const { onChainSecret, epoch } = await this.channel.coreConnector.hoprChannels.methods
      .accounts(counterParty)
      .call()
      .then((res) => {
        return {
          onChainSecret: new Hash(stringToU8a(res.hashedSecret)),
          epoch: new TicketEpoch(Number(res.counter)),
        }
      })

    const signedTicket = new SignedTicket(arr)

    const ticket = new Ticket(
      {
        bytes: signedTicket.buffer,
        offset: signedTicket.ticketOffset,
      },
      {
        channelId,
        challenge,
        epoch,
        amount,
        winProb,
        onChainSecret,
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

  async submit(signedTicket: SignedTicket, secretA: Uint8Array, secretB: Uint8Array): Promise<void> {
    const { hoprChannels, signTransaction, account, utils } = this.channel.coreConnector
    const { ticket, signature } = signedTicket
    const { r, s, v } = utils.getSignatureParameters(signature)

    const pre_image = await this.channel.coreConnector.hashedSecret
      .getPreimage(ticket.onChainSecret)
      .then((res) => res.preImage)

    const transaction = await signTransaction(
      hoprChannels.methods.redeemTicket(
        u8aToHex(pre_image),
        u8aToHex(ticket.channelId),
        u8aToHex(secretA),
        u8aToHex(secretB),
        ticket.amount.toString(),
        u8aToHex(ticket.winProb),
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

    await transaction.send()
  }
}

export default TicketFactory
