import type { Types } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'
import { stringToU8a, u8aToHex, u8aConcat } from '@hoprnet/hopr-utils'
import { Address, Balance, Hash, Signature, TicketEpoch } from '.'
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
  const messageWithHOPR = u8aConcat(stringToU8a(Web3.utils.toHex('HOPRnet')), stringToU8a(msg))
  const messageWithHOPRHex = u8aToHex(messageWithHOPR)
  return new Hash(stringToU8a(web3.eth.accounts.hashMessage(messageWithHOPRHex)))
}

class Ticket extends Uint8ArrayE implements Types.Ticket {
  constructor(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      counterparty: Address
      challenge: Hash
      epoch: TicketEpoch
      amount: Balance
      winProb: Hash
      channelIteration: TicketEpoch
    }
  ) {
    if (!arr && !struct) {
      throw Error(`Invalid constructor arguments.`)
    }

    if (!arr) {
      super(Ticket.SIZE)
    } else {
      super(arr.bytes, arr.offset, Ticket.SIZE)
    }

    if (struct) {
      this.set(struct.counterparty.serialize(), this.counterpartyOffset - this.byteOffset)
      this.set(struct.challenge, this.challengeOffset - this.byteOffset)
      this.set(new Uint8Array(struct.epoch.toBuffer('be', EPOCH_SIZE)), this.epochOffset - this.byteOffset)
      this.set(new Uint8Array(struct.amount.toBuffer('be', AMOUNT_SIZE)), this.amountOffset - this.byteOffset)
      this.set(struct.winProb, this.winProbOffset - this.byteOffset)
      this.set(
        new Uint8Array(struct.channelIteration.toBuffer('be', EPOCH_SIZE)),
        this.channelIterationOffset - this.byteOffset
      )
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

  get counterparty(): Address {
    return new Address(new Uint8Array(this.buffer, this.counterpartyOffset, Address.SIZE))
  }

  get challengeOffset(): number {
    return this.byteOffset + Address.SIZE
  }

  get challenge(): Hash {
    return new Hash(new Uint8Array(this.buffer, this.challengeOffset, Hash.SIZE))
  }

  get epochOffset(): number {
    return this.byteOffset + Address.SIZE + Hash.SIZE
  }

  get epoch(): TicketEpoch {
    return new TicketEpoch(new Uint8Array(this.buffer, this.epochOffset, EPOCH_SIZE))
  }

  get amountOffset(): number {
    return this.byteOffset + Address.SIZE + Hash.SIZE + EPOCH_SIZE
  }

  get amount(): Balance {
    return new Balance(new Uint8Array(this.buffer, this.amountOffset, AMOUNT_SIZE))
  }

  get winProbOffset(): number {
    return this.byteOffset + Address.SIZE + Hash.SIZE + EPOCH_SIZE + AMOUNT_SIZE
  }

  get winProb(): Hash {
    return new Hash(new Uint8Array(this.buffer, this.winProbOffset, Hash.SIZE))
  }

  get channelIterationOffset(): number {
    return this.byteOffset + Address.SIZE + Hash.SIZE + EPOCH_SIZE + AMOUNT_SIZE + Hash.SIZE
  }

  get channelIteration(): TicketEpoch {
    return new TicketEpoch(new Uint8Array(this.buffer, this.channelIterationOffset, EPOCH_SIZE))
  }

  get hash(): Promise<Hash> {
    return Promise.resolve(toEthSignedMessageHash(u8aToHex(this)))
  }

  static get SIZE(): number {
    return Address.SIZE + Hash.SIZE + EPOCH_SIZE + AMOUNT_SIZE + Hash.SIZE + EPOCH_SIZE
  }

  getEmbeddedFunds(): Balance {
    return new Balance(this.amount.mul(new BN(this.winProb)).div(new BN(new Uint8Array(Hash.SIZE).fill(0xff))))
  }

  async sign(privKey: Uint8Array): Promise<Signature> {
    return sign(await this.hash, privKey)
  }

  static create(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      counterparty: Address
      challenge: Hash
      epoch: TicketEpoch
      amount: Balance
      winProb: Hash
      channelIteration: TicketEpoch
    }
  ): Ticket {
    return new Ticket(arr, struct)
  }
}

export default Ticket
