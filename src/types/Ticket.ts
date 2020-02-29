// @ts-ignore
import TypeConstructors from '@hoprnet/hopr-core-connector-interface/src/types'
import BN from 'bn.js'
import { Hash, TicketEpoch, Balance, SignedTicket, Signature } from '.'
import { Uint8Array } from 'src/types/extended'
import { typedClass } from 'src/tsc/utils'
import { sign, verify, hash } from 'src/utils'
import { stringToU8a, u8aConcat } from 'src/core/u8a'

const WIN_PROB = new BN(1)

@typedClass<TypeConstructors['Ticket']>()
class Ticket extends Uint8Array {
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
          struct.channelId.toU8a(),
          struct.challenge.toU8a(),
          struct.epoch.toU8a(),
          struct.amount.toU8a(),
          struct.winProb.toU8a(),
          struct.onChainSecret.toU8a()
        )
      )
    } else {
      throw Error(`Invalid constructor arguments.`)
    }
  }

  subarray(begin: number = 0, end?: number): Uint8Array {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end != null ? end - begin : undefined)
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
    channel: any, // TODO: update type
    amount: Balance,
    challenge: Hash,
    privKey: Uint8Array,
    pubKey: Uint8Array
  ): Promise<SignedTicket> {
    const { hashedSecret } = await channel.hoprEthereum.hoprChannels.methods.accounts(channel.counterpartyHex).call()
    const winProb = new Uint8Array(new BN(new Uint8Array(Hash.SIZE).fill(0xff)).div(WIN_PROB).toArray('le', Hash.SIZE))
    const channelId = await channel.channelId

    const ticket = new Ticket(undefined, {
      channelId: channelId,
      epoch: new TicketEpoch(new BN(0)),
      challenge: challenge,
      onChainSecret: new Uint8Array(stringToU8a(hashedSecret)),
      amount: amount,
      winProb: winProb
    })

    const signature = await sign(await ticket.hash, privKey, pubKey)

    return new SignedTicket(undefined, {
      signature: signature as Signature,
      ticket
    })
  }

  // TODO: update type
  static async verify(channel: any, signedTicket: SignedTicket): Promise<boolean> {
    if ((await channel.currentBalanceOfCounterparty).add(signedTicket.ticket.amount).gt(await channel.balance)) {
      return false
    }

    try {
      await channel.testAndSetNonce(signedTicket)
    } catch {
      return false
    }

    return verify(await signedTicket.ticket.hash, signedTicket.signature, channel.offChainCounterparty)
  }

  // TODO: update type
  static async submit(channel: any, signedTicket: SignedTicket) {}
}

export default Ticket
