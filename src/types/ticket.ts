import type { Types } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'
import { u8aConcat } from '@hoprnet/hopr-utils'
import { Hash, TicketEpoch, Balance } from '.'
import { Uint8ArrayE } from '../types/extended'
import { hash, sign } from '../utils'

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
    return hash(u8aConcat(this.challenge, this.onChainSecret, this.epoch.toU8a(), this.amount.toU8a(), this.winProb))
  }

  static get SIZE(): number {
    return Hash.SIZE + Hash.SIZE + TicketEpoch.SIZE + Balance.SIZE + Hash.SIZE + Hash.SIZE
  }

  getEmbeddedFunds() {
    return this.amount.mul(new BN(this.winProb)).div(new BN(new Uint8Array(Hash.SIZE).fill(0xff)))
  }

  async sign(
    privKey: Uint8Array,
    pubKey: Uint8Array,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    }
  ): Promise<Types.Signature> {
    return await sign(await this.hash, privKey, undefined, arr)
  }

  static create(
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
    return new Ticket(arr, struct)
  }
}

export default Ticket
