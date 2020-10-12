import type { Types } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'
import { stringToU8a, u8aToHex } from '@hoprnet/hopr-utils'
import { AccountId, Balance, Hash, Signature, TicketEpoch } from '.'
import { Uint8ArrayE } from '../types/extended'
import { sign } from '../utils'

import Web3 from 'web3'
const web3 = new Web3()

const EPOCH_SIZE = 3
const AMOUNT_SIZE = 12

/**
 * Given a message, prefix it with "\x19Ethereum Signed Message:\n" and return it's hash
 * @param msg the message to hash
 * @returns a hash
 */
function toEthSignedMessageHash(msg: string): Hash {
  return new Hash(stringToU8a(web3.eth.accounts.hashMessage(msg)))
}

class Ticket extends Uint8ArrayE implements Types.Ticket {
  constructor(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      counterparty: AccountId
      challenge: Hash
      epoch: TicketEpoch
      amount: Balance
      winProb: Hash
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
      this.set(struct.counterparty, this.counterpartyOffset - this.byteOffset)
      this.set(struct.challenge, this.challengeOffset - this.byteOffset)
      this.set(new Uint8Array(struct.epoch.toBuffer('be', EPOCH_SIZE)), this.epochOffset - this.byteOffset)
      this.set(new Uint8Array(struct.amount.toBuffer('be', AMOUNT_SIZE)), this.amountOffset - this.byteOffset)
      this.set(struct.winProb, this.winProbOffset - this.byteOffset)
    }
  }

  slice(begin = 0, end = Ticket.SIZE) {
    return this.subarray(begin, end)
  }

  subarray(begin = 0, end = Ticket.SIZE) {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }

  get counterpartyOffset(): number {
    return this.byteOffset
  }

  get counterparty(): AccountId {
    return new AccountId(new Uint8Array(this.buffer, this.counterpartyOffset, AccountId.SIZE))
  }

  get challengeOffset(): number {
    return this.byteOffset + AccountId.SIZE
  }

  get challenge(): Hash {
    return new Hash(new Uint8Array(this.buffer, this.challengeOffset, Hash.SIZE))
  }

  get epochOffset(): number {
    return this.byteOffset + AccountId.SIZE + Hash.SIZE
  }

  get epoch(): TicketEpoch {
    return new TicketEpoch(new Uint8Array(this.buffer, this.epochOffset, EPOCH_SIZE))
  }

  get amountOffset(): number {
    return this.byteOffset + AccountId.SIZE + Hash.SIZE + EPOCH_SIZE
  }

  get amount(): Balance {
    return new Balance(new Uint8Array(this.buffer, this.amountOffset, AMOUNT_SIZE))
  }

  get winProbOffset(): number {
    return this.byteOffset + AccountId.SIZE + Hash.SIZE + EPOCH_SIZE + AMOUNT_SIZE
  }

  get winProb(): Hash {
    return new Hash(new Uint8Array(this.buffer, this.winProbOffset, Hash.SIZE))
  }

  get hash(): Promise<Hash> {
    return Promise.resolve(toEthSignedMessageHash(u8aToHex(this)))
  }

  static get SIZE(): number {
    return AccountId.SIZE + Hash.SIZE + EPOCH_SIZE + AMOUNT_SIZE + Hash.SIZE
  }

  getEmbeddedFunds(): Balance {
    return new Balance(this.amount.mul(new BN(this.winProb)).div(new BN(new Uint8Array(Hash.SIZE).fill(0xff))))
  }

  async sign(
    privKey: Uint8Array,
    pubKey: Uint8Array | undefined,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    }
  ): Promise<Signature> {
    return sign(await this.hash, privKey, undefined, arr)
  }

  static create(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      counterparty: AccountId
      challenge: Hash
      epoch: TicketEpoch
      amount: Balance
      winProb: Hash
    }
  ): Ticket {
    return new Ticket(arr, struct)
  }
}

export default Ticket
