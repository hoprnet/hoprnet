import type IChannel from '.'
import BN from 'bn.js'
import { u8aToHex, u8aConcat, stringToU8a } from '@hoprnet/hopr-utils'
import { Hash, TicketEpoch, Balance, SignedTicket, Ticket } from '../types'
import { pubKeyToAccountId } from '../utils'

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
    const counterParty = (await pubKeyToAccountId(this.channel.counterparty)).toHex()

    const epoch = await this.channel.coreConnector.hoprChannels.methods
      .accounts(counterParty)
      .call()
      .then((res) => new TicketEpoch(Number(res.counter)))

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

  async submit(signedTicket: SignedTicket, hashedSecretASecretB: Hash): Promise<void> {
    const { hoprChannels, signTransaction, account, utils } = this.channel.coreConnector
    const { ticket, signature } = signedTicket
    const { r, s, v } = utils.getSignatureParameters(signature)

    const onChainSecret = await this.channel.coreConnector.hoprChannels.methods
      .accounts(u8aToHex(await this.channel.coreConnector.account.address))
      .call()
      .then((res) => new Hash(stringToU8a(res.hashedSecret)))

    const preImage = await this.channel.coreConnector.hashedSecret.findPreImage(onChainSecret)

    const transaction = await signTransaction(
      hoprChannels.methods.redeemTicket(
        u8aToHex(preImage.preImage),
        u8aToHex(ticket.channelId),
        u8aToHex(hashedSecretASecretB),
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
