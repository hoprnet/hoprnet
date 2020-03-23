import type { Types } from "@hoprnet/hopr-core-connector-interface"
import BN from 'bn.js'
import { Hash, TicketEpoch, Balance, SignedTicket, Signature } from '.'
import { Uint8ArrayE } from '../types/extended'
import { sign, verify, hash } from '../utils'
import { stringToU8a, u8aConcat, u8aToHex } from '../core/u8a'
import ChannelInstance from '../channel'

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
    if (arr != null && struct == null) {
      super(arr.bytes, arr.offset, Ticket.SIZE)
    } else if (arr == null && struct != null) {
      super(
        u8aConcat(
          new Hash(struct.channelId).toU8a(),
          new Hash(struct.challenge).toU8a(),
          new TicketEpoch(struct.epoch).toU8a(),
          new Balance(struct.amount).toU8a(),
          new Hash(struct.winProb).toU8a(),
          new Hash(struct.onChainSecret).toU8a()
        )
      )
    } else {
      throw Error(`Invalid constructor arguments.`)
    }
  }

  get channelId(): Hash {
    return new Hash(this.subarray(0, Hash.SIZE))
  }

  get challenge(): Hash {
    return new Hash(this.subarray(Hash.SIZE, Hash.SIZE + Hash.SIZE))
  }

  get epoch(): TicketEpoch {
    const start = Hash.SIZE + Hash.SIZE
    return new TicketEpoch(this.subarray(start, start + TicketEpoch.SIZE))
  }

  get amount(): Balance {
    const start = Hash.SIZE + Hash.SIZE + TicketEpoch.SIZE
    return new Balance(this.subarray(start, start + Balance.SIZE))
  }

  get winProb(): Hash {
    const start = Hash.SIZE + Hash.SIZE + TicketEpoch.SIZE + Balance.SIZE
    return new Hash(this.subarray(start, start + Hash.SIZE))
  }

  get onChainSecret(): Hash {
    const start = Hash.SIZE + Hash.SIZE + TicketEpoch.SIZE + Balance.SIZE + Hash.SIZE
    return new Hash(this.subarray(start, start + Hash.SIZE))
  }

  getEmbeddedFunds() {
    return this.amount.mul(new BN(this.winProb)).div(new BN(new Uint8Array(Hash.SIZE).fill(0xff)))
  }

  get hash() {
    return hash(
      u8aConcat(
        this.challenge,
        this.onChainSecret,
        this.epoch.toU8a(),
        new Uint8Array(this.amount.toNumber()),
        this.winProb
      )
    )
  }

  static get SIZE(): number {
    return Hash.SIZE + Hash.SIZE + TicketEpoch.SIZE + Balance.SIZE + Hash.SIZE + Hash.SIZE
  }

  static async create(
    channel: ChannelInstance,
    amount: Balance,
    challenge: Hash
  ): Promise<SignedTicket> {
    const account = await channel.coreConnector.utils.pubKeyToAccountId(channel.counterparty)
    const { hashedSecret } = await channel.coreConnector.hoprChannels.methods
      .accounts(u8aToHex(account))
      .call()

    const winProb = new Uint8ArrayE(new BN(new Uint8Array(Hash.SIZE).fill(0xff)).div(WIN_PROB).toArray('le', Hash.SIZE))
    const channelId = await channel.channelId

    const ticket = new Ticket(undefined, {
      channelId: channelId,
      challenge: challenge,
      epoch: new TicketEpoch(0),
      amount: new Balance(amount.toString()),
      winProb: winProb,
      onChainSecret: new Uint8ArrayE(stringToU8a(hashedSecret)),
    })

    const signature = await sign(await ticket.hash, channel.coreConnector.self.privateKey).then(res => new Signature({
      bytes: res.buffer,
      offset: res.byteOffset
    }))

    return new SignedTicket(undefined, {
      signature,
      ticket
    })
  }

  // TODO: use this
  static async verify(channel: ChannelInstance, signedTicket: SignedTicket): Promise<boolean> {
    // if ((await channel.currentBalanceOfCounterparty).add(signedTicket.ticket.amount).gt(await channel.balance)) {
    //   return false
    // }

    // try {
    //   await channel.testAndSetNonce(signedTicket)
    // } catch {
    //   return false
    // }

    return verify(await signedTicket.ticket.hash, signedTicket.signature, channel.offChainCounterparty)
  }

  // TODO: implement
  static async submit(channel: any, signedTicket: SignedTicket) {
    throw Error('not implemented')
  }
}

export default Ticket
