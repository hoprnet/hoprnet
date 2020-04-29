import type { Types } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'
import { stringToU8a, u8aToHex } from '@hoprnet/hopr-utils'
import { Hash, TicketEpoch, Balance, SignedTicket } from '.'
import { Uint8ArrayE } from '../types/extended'
import { sign, verify, hash } from '../utils'
import type ChannelInstance from '../channel'

const WIN_PROB = new BN(1)

class Ticket extends Uint8ArrayE implements Types.Ticket {
  constructor(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      channelId: Hash
      challenge: Hash
      epoch: TicketEpoch
      amount: Balance
      winProb: Hash
      onChainSecret: Hash
    }
  ) {
    if (arr == null && struct == null) {
      throw Error(`Invalid constructor arguments.`)
    }

    if (arr == null) {
      super(Ticket.SIZE)
    } else {
      super(arr.bytes, arr.offset, Ticket.SIZE)
    }

    if (struct != null) {
      this.set(struct.channelId, this.channelIdOffset - this.byteOffset)
      this.set(struct.challenge, this.challengeOffset - this.byteOffset)
      this.set(struct.epoch.toU8a(), this.epochOffset - this.byteOffset)
      this.set(struct.amount.toU8a(), this.amountOffset - this.byteOffset)
      this.set(struct.winProb, this.winProbOffset - this.byteOffset)
      this.set(struct.onChainSecret, this.onChainSecretOffset - this.byteOffset)
    }
  }

  get channelIdOffset(): number {
    return this.byteOffset
  }

  get channelId(): Hash {
    return new Hash(new Uint8Array(this.buffer, this.channelIdOffset, Hash.SIZE))
  }

  get challengeOffset(): number {
    return this.byteOffset + Hash.SIZE
  }

  get challenge(): Hash {
    return new Hash(new Uint8Array(this.buffer, this.challengeOffset, Hash.SIZE))
  }

  get epochOffset(): number {
    return this.byteOffset + Hash.SIZE + Hash.SIZE
  }

  get epoch(): TicketEpoch {
    return new TicketEpoch(new Uint8Array(this.buffer, this.epochOffset, TicketEpoch.SIZE))
  }

  get amountOffset(): number {
    return this.byteOffset + Hash.SIZE + Hash.SIZE + TicketEpoch.SIZE
  }

  get amount(): Balance {
    return new Balance(new Uint8Array(this.buffer, this.amountOffset, Balance.SIZE))
  }

  get winProbOffset(): number {
    return this.byteOffset + Hash.SIZE + Hash.SIZE + TicketEpoch.SIZE + Balance.SIZE
  }

  get winProb(): Hash {
    return new Hash(new Uint8Array(this.buffer, this.winProbOffset, Hash.SIZE))
  }

  get onChainSecretOffset(): number {
    return this.byteOffset + Hash.SIZE + Hash.SIZE + TicketEpoch.SIZE + Balance.SIZE + Hash.SIZE
  }

  get onChainSecret(): Hash {
    return new Hash(new Uint8Array(this.buffer, this.onChainSecretOffset, Hash.SIZE))
  }

  get hash(): Promise<Hash> {
    return hash(this)
  }

  static get SIZE(): number {
    return Hash.SIZE + Hash.SIZE + TicketEpoch.SIZE + Balance.SIZE + Hash.SIZE + Hash.SIZE
  }

  getEmbeddedFunds() {
    return this.amount.mul(new BN(this.winProb)).div(new BN(new Uint8Array(Hash.SIZE).fill(0xff)))
  }

  static async create(
    channel: ChannelInstance,
    amount: Balance,
    challenge: Hash,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    }
  ): Promise<SignedTicket> {
    const account = await channel.coreConnector.utils.pubKeyToAccountId(channel.counterparty)
    const { hashedSecret } = await channel.coreConnector.hoprChannels.methods.accounts(u8aToHex(account)).call()

    const winProb = new Uint8ArrayE(new BN(new Uint8Array(Hash.SIZE).fill(0xff)).div(WIN_PROB).toArray('le', Hash.SIZE))
    const channelId = await channel.channelId

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

    await sign(await ticket.hash, channel.coreConnector.self.privateKey, undefined, {
      bytes: signedTicket.buffer,
      offset: signedTicket.signatureOffset,
    })

    return signedTicket
  }

  static async verify(channel: ChannelInstance, signedTicket: SignedTicket): Promise<boolean> {
    // @TODO: check if this is needed
    // if ((await channel.currentBalanceOfCounterparty).add(signedTicket.ticket.amount).lt(await channel.balance)) {
    //   return false
    // }

    try {
      await channel.testAndSetNonce(signedTicket)
    } catch {
      return false
    }

    return verify(await signedTicket.ticket.hash, signedTicket.signature, await channel.offChainCounterparty)
  }

  // @TODO: implement submit
  static async submit(channel: any, signedTicket: SignedTicket) {
    throw Error('not implemented')
  }
}

export default Ticket
