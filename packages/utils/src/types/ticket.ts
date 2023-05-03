import BN from 'bn.js'
import { stringToU8a, u8aSplit, serializeToU8a } from '../index.js'
import { Address, Balance, Hash, Signature, UINT256, PublicKey, Response } from './index.js'
import { ethers } from 'ethers'
import type { Challenge } from './challenge.js'
import { EthereumChallenge } from './ethereumChallenge.js'
import { PRICE_PER_PACKET, INVERSE_TICKET_WIN_PROB } from '../constants.js'

// Prefix message with "\x19Ethereum Signed Message:\n {length} HOPRnet {message}" and return hash
export function toEthSignedMessageHash(message: Hash): Hash {
  const result = ethers.utils.solidityKeccak256(
    ['string', 'bytes'],
    ['\x19Ethereum Signed Message:\n32', message.serialize()]
  )

  return new Hash(stringToU8a(result))
}

function serializeUnsigned({
  counterparty,
  challenge,
  epoch,
  index,
  amount,
  winProb,
  channelEpoch
}: {
  counterparty: Address
  challenge: EthereumChallenge
  epoch: UINT256
  index: UINT256
  amount: Balance
  winProb: UINT256
  channelEpoch: UINT256
}): Uint8Array {
  // the order of the items needs to be the same as the one used in the SC
  return serializeToU8a([
    [counterparty.serialize(), Address.SIZE],
    [challenge.serialize(), EthereumChallenge.SIZE],
    [epoch.serialize(), UINT256.SIZE],
    [amount.serialize(), Balance.SIZE],
    [winProb.serialize(), UINT256.SIZE],
    [index.serialize(), UINT256.SIZE],
    [channelEpoch.serialize(), UINT256.SIZE]
  ])
}

export class Ticket {
  constructor(
    readonly counterparty: Address,
    readonly challenge: EthereumChallenge,
    readonly epoch: UINT256,
    readonly index: UINT256,
    readonly amount: Balance,
    readonly winProb: UINT256,
    readonly channelEpoch: UINT256,
    readonly signature: Signature
  ) {}

  static create(
    counterparty: Address,
    challenge: Challenge,
    epoch: UINT256,
    index: UINT256,
    amount: Balance,
    winProb: UINT256,
    channelEpoch: UINT256,
    signPriv: Uint8Array
  ): Ticket {
    const encodedChallenge = challenge.toEthereumChallenge()

    const hashedTicket = Hash.create(
      serializeUnsigned({
        counterparty,
        challenge: encodedChallenge,
        epoch,
        index,
        amount,
        winProb,
        channelEpoch
      })
    )

    const message = toEthSignedMessageHash(hashedTicket)
    const signature = Signature.create(message.serialize(), signPriv)
    return new Ticket(counterparty, encodedChallenge, epoch, index, amount, winProb, channelEpoch, signature)
  }

  public serialize(): Uint8Array {
    return Uint8Array.from([...this.serializeUnsigned(), ...this.signature.serialize()])
  }

  public serializeUnsigned(): Uint8Array {
    return serializeUnsigned({ ...this })
  }

  static deserialize(arr: Uint8Array): Ticket {
    const components = u8aSplit(arr, [
      Address.SIZE,
      EthereumChallenge.SIZE,
      UINT256.SIZE,
      UINT256.SIZE,
      Balance.SIZE,
      UINT256.SIZE,
      UINT256.SIZE,
      Signature.SIZE
    ])

    const counterparty = new Address(components[0])
    const challenge = new EthereumChallenge(components[1])
    const epoch = new UINT256(new BN(components[2]))
    const amount = new Balance(new BN(components[3]))
    const winProb = new UINT256(new BN(components[4]))
    const index = new UINT256(new BN(components[5]))
    const channelEpoch = new UINT256(new BN(components[6]))
    const signature = Signature.deserialize(components[7])
    return new Ticket(counterparty, challenge, epoch, index, amount, winProb, channelEpoch, signature)
  }

  toString() {
    return (
      // prettier-ignore
      `Ticket:\n` +
      `  counterparty:     ${this.counterparty.toHex()}\n` +
      `  challenge:        ${this.challenge.toHex()}\n` +
      `  epoch:            ${this.epoch.toBN().toString(10)}\n` +
      `  amount:           ${this.amount.toFormattedString()}\n` +
      `  index:            ${this.index.toBN().toString(10)}\n` +
      `  winProb:          ${this.winProb.toBN().div(new BN(new Uint8Array(UINT256.SIZE).fill(0xff))).muln(100)} %\n` +
      `  channelEpoch:     ${this.channelEpoch.toBN().toString(10)}`
    )
  }

  getHash(): Hash {
    return toEthSignedMessageHash(Hash.create(this.serializeUnsigned()))
  }

  static get SIZE(): number {
    return (
      Address.SIZE +
      EthereumChallenge.SIZE +
      UINT256.SIZE +
      UINT256.SIZE +
      Balance.SIZE +
      Hash.SIZE +
      UINT256.SIZE +
      Signature.SIZE
    )
  }

  recoverSigner() {
    return PublicKey.fromSignature(this.getHash().serialize(), this.signature.signature, this.signature.recovery)
  }

  verify(pubKey: PublicKey): boolean {
    const signer = this.recoverSigner()
    return pubKey.eq(signer)
  }

  getLuck(preImage: Hash, challengeResponse: Response): UINT256 {
    return new UINT256(
      new BN(
        Hash.create(
          Uint8Array.from([...this.getHash().serialize(), ...preImage.serialize(), ...challengeResponse.serialize()])
        ).serialize()
      )
    )
  }

  /**
   * Decides whether a ticket is a win or not.
   * Note that this mimics the on-chain logic.
   * @dev Purpose of the function is to check the validity of
   * a ticket before we submit it to the blockchain.
   * @param challengeResponse response that solves the signed challenge
   * @param preImage preImage of the current onChainSecret
   * @param winProb winning probability of the ticket
   */
  isWinningTicket(preImage: Hash, challengeResponse: Response, winProb: UINT256): boolean {
    const luck = this.getLuck(preImage, challengeResponse)
    return luck.toBN().lte(winProb.toBN())
  }

  getPathPosition(): number {
    const baseUnit = PRICE_PER_PACKET.mul(INVERSE_TICKET_WIN_PROB)

    if (!this.amount.toBN().mod(baseUnit).isZero()) {
      throw Error(`Invalid balance`)
    }

    return this.amount.toBN().div(baseUnit).toNumber()
  }
}
