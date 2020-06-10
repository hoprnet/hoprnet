import BN from 'bn.js'
import { stringToU8a, u8aToHex } from '@hoprnet/hopr-utils'
import { Hash, TicketEpoch, Balance, SignedTicket, Ticket } from '../types'

import { Uint8ArrayE } from '../types/extended'
import type Channel from '.'

const DEFAULT_WIN_PROB = new BN(1)

class TicketFactory {
  constructor(public channel: Channel) {}

  async create(
    amount: Balance,
    challenge: Hash,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    }
  ): Promise<SignedTicket> {
    const account = await this.channel.coreConnector.utils.pubKeyToAccountId(this.channel.counterparty)
    const { hashedSecret } = await this.channel.coreConnector.hoprChannels.methods.accounts(u8aToHex(account)).call()

    const winProb = new Uint8ArrayE(
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
        // @TODO set this dynamically
        epoch: new TicketEpoch(0),
        amount: new Balance(amount.toString()),
        winProb,
        onChainSecret: new Uint8ArrayE(stringToU8a(hashedSecret)),
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

  // @TODO: implement submit
  async submit(signedTicket: SignedTicket) {
    throw Error('not implemented')
  }
}

export default TicketFactory
