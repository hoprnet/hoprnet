import type { Types } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'
import { stringToU8a, u8aToHex } from '@hoprnet/hopr-utils'
import { Hash, TicketEpoch, Balance, Signature } from '.'
import { Uint8ArrayE } from '../types/extended'
import { sign, hash } from '../utils'
import { HASHED_SECRET_WIDTH } from '../hashedSecret'
//
import Web3 from 'web3'
const web3 = new Web3()

/**
 * Given a message, prefix it with "\x19Ethereum Signed Message:\n" and return it's hash
 * @param msg the message to hash
 * @returns a hash
 */
function toEthSignedMessageHash(msg: string): Hash {
  return new Hash(stringToU8a(web3.eth.accounts.hashMessage(msg)))
}

function encode(items: { type: string; value: string }[]): string {
  const { types, values } = items.reduce(
    (result, item) => {
      result.types.push(item.type)
      result.values.push(item.value)

      return result
    },
    {
      types: [],
      values: [],
    }
  )

  return web3.eth.abi.encodeParameters(types, values)
}

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
    return new Hash(new Uint8Array(this.buffer, this.onChainSecretOffset, HASHED_SECRET_WIDTH))
  }

  get hash(): Promise<Hash> {
    return new Promise<Hash>(async (resolve) => {
      const encodedTicket = encode([
        { type: 'bytes32', value: u8aToHex(this.channelId) },
        { type: 'bytes32', value: u8aToHex(await hash(this.challenge)) },
        { type: 'bytes32', value: u8aToHex(this.onChainSecret) },
        { type: 'uint256', value: this.epoch.toString() },
        { type: 'uint256', value: this.amount.toString() },
        { type: 'bytes32', value: u8aToHex(this.winProb) },
      ])

      resolve(toEthSignedMessageHash(encodedTicket))
    })
  }

  static get SIZE(): number {
    return Hash.SIZE + Hash.SIZE + TicketEpoch.SIZE + Balance.SIZE + Hash.SIZE + Hash.SIZE
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
      channelId: Hash
      challenge: Hash
      epoch: TicketEpoch
      amount: Balance
      winProb: Hash
      onChainSecret: Hash
    }
  ): Ticket {
    return new Ticket(arr, struct)
  }
}

export default Ticket
