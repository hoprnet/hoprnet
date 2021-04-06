import type { Types } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'
import { stringToU8a, u8aSplit, u8aToHex, u8aConcat, serializeToU8a } from '@hoprnet/hopr-utils'
import { Address, Balance, Hash, Signature, UINT256 } from '.'
import { sign } from '../utils'

import Web3 from 'web3'
const web3 = new Web3()

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

class Ticket implements Types.Ticket {
  constructor(
    readonly counterparty: Address,
    readonly challenge: Hash,
    readonly epoch: UINT256,
    readonly amount: Balance,
    readonly winProb: Hash,
    readonly channelIteration: UINT256
  ) {}

  /*
  get counterpartyOffset(): number {
    return this.byteOffset
  }

  get counterparty(): Address {
    return new Address(new Uint8Array(this.buffer, this.counterpartyOffset, Address.SIZE))
  }
  */

  public serialize(): Uint8Array {
    // the order of the items needs to be the same as the one used in the SC
    return serializeToU8a([
      [this.counterparty.serialize(), Address.SIZE],
      [this.challenge.serialize(), Hash.SIZE],
      [this.epoch.serialize(), UINT256.SIZE],
      [this.amount.serialize(), Balance.SIZE],
      [this.winProb.serialize(), Hash.SIZE],
      [this.channelIteration.serialize(), UINT256.SIZE]
    ])
  }

 static deserialize(arr: Uint8Array): Ticket {
    const components = u8aSplit(arr, [
      Address.SIZE,
      Hash.SIZE,
      UINT256.SIZE,
      Balance.SIZE,
      Hash.SIZE,
      UINT256.SIZE
    ])

    const counterparty = new Address(components[0])
    const challenge = new Hash(components[1])
    const epoch = new UINT256(new BN(components[2]))
    const amount = new Balance(new BN(components[3]))
    const winProb = new Hash(components[4])
    const channelIteration = new UINT256(new BN(components[4]))
    return new Ticket(counterparty, challenge, epoch, amount, winProb, channelIteration)
  }

  /*
  get challenge(): Hash {
    return new Hash(new Uint8Array(this.buffer, this.challengeOffset, Hash.SIZE))
  }

  get epoch(): UINT256 {
    return new UINT256(new BN(new Uint8Array(this.buffer, this.epochOffset, UINT256.SIZE)))
  }

  get amount(): Balance {
    return new Balance(new BN(new Uint8Array(this.buffer, this.amountOffset, Balance.SIZE)))
  }

  get winProb(): Hash {
    return new Hash(new Uint8Array(this.buffer, this.winProbOffset, Hash.SIZE))
  }

  get channelIterationOffset(): number {
    return this.byteOffset + Address.SIZE + Hash.SIZE + UINT256.SIZE + UINT256.SIZE + Hash.SIZE
  }

  get channelIteration(): UINT256 {
    return new UINT256(new BN(new Uint8Array(this.buffer, this.channelIterationOffset, UINT256.SIZE)))
  }
  */

  getHash(): Hash {
    return toEthSignedMessageHash(u8aToHex(this.serialize()))
  }

  static get SIZE(): number {
    return Address.SIZE + Hash.SIZE + UINT256.SIZE + UINT256.SIZE + Hash.SIZE + UINT256.SIZE
  }

  getEmbeddedFunds(): Balance {
    return new Balance(
      this.amount
        .toBN()
        .mul(new BN(this.winProb.serialize()))
        .div(new BN(new Uint8Array(Hash.SIZE).fill(0xff)))
    )
  }

  async sign(privKey: Uint8Array): Promise<Signature> {
    return sign(this.getHash().serialize(), privKey)
  }
}

export default Ticket
