import BN from 'bn.js'
import { stringToU8a, u8aSplit, u8aToHex, serializeToU8a } from '..'
import { Address, Balance, Hash, Signature, UINT256, PublicKey, Response } from '.'
import { ecdsaRecover, ecdsaSign } from 'secp256k1'
import { ethers } from 'ethers'
import { Challenge } from './challenge'
import { EthereumChallenge } from './ethereumChallenge'

// Prefix message with "\x19Ethereum Signed Message:\n {length} HOPRnet {message}" and return hash
function toEthSignedMessageHash(message: string): Hash {
  const withHOPR = ethers.utils.concat([ethers.utils.toUtf8Bytes('HOPRnet'), message])
  const result = ethers.utils.solidityKeccak256(
    ['string', 'string', 'bytes'],
    ['\x19Ethereum Signed Message:\n', withHOPR.length.toString(), withHOPR]
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
  channelIteration
}: {
  counterparty: Address
  challenge: EthereumChallenge
  epoch: UINT256
  index: UINT256
  amount: Balance
  winProb: UINT256
  channelIteration: UINT256
}): Uint8Array {
  // the order of the items needs to be the same as the one used in the SC
  return serializeToU8a([
    [counterparty.serialize(), Address.SIZE],
    [challenge.serialize(), EthereumChallenge.SIZE],
    [epoch.serialize(), UINT256.SIZE],
    [index.serialize(), UINT256.SIZE],
    [amount.serialize(), Balance.SIZE],
    [winProb.serialize(), UINT256.SIZE],
    [channelIteration.serialize(), UINT256.SIZE]
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
    readonly channelIteration: UINT256,
    readonly signature: Signature
  ) {}

  static create(
    counterparty: Address,
    challenge: Challenge,
    epoch: UINT256,
    index: UINT256,
    amount: Balance,
    winProb: UINT256,
    channelIteration: UINT256,
    signPriv: Uint8Array
  ): Ticket {
    const encodedChallenge = challenge.toEthereumChallenge()

    const hash = toEthSignedMessageHash(
      u8aToHex(
        serializeToU8a([
          [counterparty.serialize(), Address.SIZE],
          [encodedChallenge.serialize(), EthereumChallenge.SIZE],
          [epoch.serialize(), UINT256.SIZE],
          [index.serialize(), UINT256.SIZE],
          [amount.serialize(), Balance.SIZE],
          [winProb.serialize(), Hash.SIZE],
          [channelIteration.serialize(), UINT256.SIZE]
        ])
      )
    )
    const sig = ecdsaSign(hash.serialize(), signPriv)
    const signature = new Signature(sig.signature, sig.recid)
    return new Ticket(counterparty, encodedChallenge, epoch, index, amount, winProb, channelIteration, signature)
  }

  public serialize(): Uint8Array {
    const unsigned = serializeUnsigned({ ...this })
    return Uint8Array.from([...unsigned, ...this.signature.serialize()])
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
    const index = new UINT256(new BN(components[3]))
    const amount = new Balance(new BN(components[4]))
    const winProb = new UINT256(new BN(components[5]))
    const channelIteration = new UINT256(new BN(components[6]))
    const signature = Signature.deserialize(components[7])
    return new Ticket(counterparty, challenge, epoch, index, amount, winProb, channelIteration, signature)
  }

  getHash(): Hash {
    return toEthSignedMessageHash(
      u8aToHex(
        serializeToU8a([
          [this.counterparty.serialize(), Address.SIZE],
          [this.challenge.serialize(), EthereumChallenge.SIZE],
          [this.epoch.serialize(), UINT256.SIZE],
          [this.index.serialize(), UINT256.SIZE],
          [this.amount.serialize(), Balance.SIZE],
          [this.winProb.serialize(), Hash.SIZE],
          [this.channelIteration.serialize(), UINT256.SIZE]
        ])
      )
    )
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
    return new PublicKey(ecdsaRecover(this.signature.signature, this.signature.recovery, this.getHash().serialize()))
  }

  verify(pubKey: PublicKey): boolean {
    return pubKey.eq(this.recoverSigner())
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
    const luck = new BN(
      Hash.create(
        Uint8Array.from([
          ...this.getHash().serialize(),
          ...preImage.serialize(),
          ...challengeResponse.serialize(),
          ...winProb.serialize()
        ])
      ).serialize()
    )

    return luck.lte(winProb.toBN())
  }

  getPathPosition(baseUnit: Balance): number {
    if (!this.amount.toBN().mod(baseUnit.toBN()).isZero()) {
      throw Error(`Invalid balance`)
    }

    return this.amount.toBN().div(baseUnit.toBN()).toNumber()
  }
}
