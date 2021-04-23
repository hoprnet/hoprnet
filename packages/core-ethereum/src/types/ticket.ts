import BN from 'bn.js'
import { stringToU8a, u8aSplit, u8aToHex, u8aConcat, serializeToU8a } from '@hoprnet/hopr-utils'
import { Address, Balance, Hash, Signature, UINT256, PublicKey } from '.'
import { ecdsaVerify, ecdsaSign, ecdsaRecover } from 'secp256k1'
import { ethers } from 'ethers'

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
  challenge: Hash
  epoch: UINT256
  index: UINT256
  amount: Balance
  winProb: Hash
  channelIteration: UINT256
}): Uint8Array {
  // the order of the items needs to be the same as the one used in the SC
  return serializeToU8a([
    [counterparty.serialize(), Address.SIZE],
    [challenge.serialize(), Hash.SIZE],
    [epoch.serialize(), UINT256.SIZE],
    [index.serialize(), UINT256.SIZE],
    [amount.serialize(), Balance.SIZE],
    [winProb.serialize(), Hash.SIZE],
    [channelIteration.serialize(), UINT256.SIZE]
  ])
}

class Ticket {
  constructor(
    readonly counterparty: Address,
    readonly challenge: Hash,
    readonly epoch: UINT256,
    readonly index: UINT256,
    readonly amount: Balance,
    readonly winProb: Hash,
    readonly channelIteration: UINT256,
    readonly signature: Signature
  ) {}

  static create(
    counterparty: Address,
    challenge: Hash,
    epoch: UINT256,
    index: UINT256,
    amount: Balance,
    winProb: Hash,
    channelIteration: UINT256,
    signPriv: Uint8Array
  ): Ticket {
    const hash = toEthSignedMessageHash(
      u8aToHex(
        serializeUnsigned({
          counterparty,
          challenge,
          epoch,
          index,
          amount,
          winProb,
          channelIteration
        })
      )
    )
    const sig = ecdsaSign(hash.serialize(), signPriv)
    const signature = new Signature(sig.signature, sig.recid)
    return new Ticket(counterparty, challenge, epoch, index, amount, winProb, channelIteration, signature)
  }

  public serialize(): Uint8Array {
    const unsigned = serializeUnsigned({ ...this })
    return u8aConcat(serializeToU8a([[this.signature.serialize(), Signature.SIZE]]), unsigned)
  }

  static deserialize(arr: Uint8Array): Ticket {
    const components = u8aSplit(arr, [
      Address.SIZE,
      Hash.SIZE,
      UINT256.SIZE,
      Balance.SIZE,
      Hash.SIZE,
      UINT256.SIZE,
      Signature.SIZE
    ])

    const counterparty = new Address(components[0])
    const challenge = new Hash(components[1])
    const epoch = new UINT256(new BN(components[2]))
    const index = new UINT256(new BN(components[3]))
    const amount = new Balance(new BN(components[4]))
    const winProb = new Hash(components[5])
    const channelIteration = new UINT256(new BN(components[6]))
    const signature = Signature.deserialize(components[7])
    return new Ticket(counterparty, challenge, epoch, index, amount, winProb, channelIteration, signature)
  }

  getHash(): Hash {
    return toEthSignedMessageHash(u8aToHex(serializeUnsigned({ ...this })))
  }

  static get SIZE(): number {
    return Address.SIZE + Hash.SIZE + UINT256.SIZE + UINT256.SIZE + Hash.SIZE + UINT256.SIZE + Signature.SIZE
  }

  getEmbeddedFunds(): Balance {
    return new Balance(
      this.amount
        .toBN()
        .mul(new BN(this.winProb.serialize()))
        .div(new BN(new Uint8Array(Hash.SIZE).fill(0xff)))
    )
  }

  getSigner(): PublicKey {
    return new PublicKey(ecdsaRecover(this.signature.signature, this.signature.recovery, this.getHash().serialize()))
  }

  async verify(pubKey: PublicKey): Promise<boolean> {
    return ecdsaVerify(this.signature.signature, this.getHash().serialize(), pubKey.serialize())
  }
}
export default Ticket
