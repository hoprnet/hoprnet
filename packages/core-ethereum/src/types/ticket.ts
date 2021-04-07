import type { Types } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'
import { stringToU8a, u8aSplit, u8aToHex, u8aConcat, serializeToU8a } from '@hoprnet/hopr-utils'
import { Address, Balance, Hash, Signature, UINT256, PublicKey } from '.'
import { ecdsaVerify, ecdsaSign, ecdsaRecover } from 'secp256k1'
import Web3 from 'web3'
const web3 = new Web3()

// Prefix message with "\x19Ethereum Signed Message:\n" and return hash
function toEthSignedMessageHash(msg: string): Hash {
  const messageWithHOPR = u8aConcat(stringToU8a(Web3.utils.toHex('HOPRnet')), stringToU8a(msg))
  const messageWithHOPRHex = u8aToHex(messageWithHOPR)
  return new Hash(stringToU8a(web3.eth.accounts.hashMessage(messageWithHOPRHex)))
}

function serializeUnsigned({
  counterparty,
  challenge,
  epoch,
  amount,
  winProb,
  channelIteration
}: {
  counterparty: Address
  challenge: Hash
  epoch: UINT256
  amount: Balance
  winProb: Hash
  channelIteration: UINT256
}): Uint8Array {
  // the order of the items needs to be the same as the one used in the SC
  return serializeToU8a([
    [counterparty.serialize(), Address.SIZE],
    [challenge.serialize(), Hash.SIZE],
    [epoch.serialize(), UINT256.SIZE],
    [amount.serialize(), Balance.SIZE],
    [winProb.serialize(), Hash.SIZE],
    [channelIteration.serialize(), UINT256.SIZE]
  ])
}

class Ticket implements Types.Ticket {
  constructor(
    readonly counterparty: Address,
    readonly challenge: Hash,
    readonly epoch: UINT256,
    readonly amount: Balance,
    readonly winProb: Hash,
    readonly channelIteration: UINT256,
    readonly signature: Signature
  ) {}

  static create(
    counterparty: Address,
    challenge: Hash,
    epoch: UINT256,
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
          amount,
          winProb,
          channelIteration
        })
      )
    )
    const sig = ecdsaSign(hash.serialize(), signPriv)
    const signature = new Signature(sig.signature, sig.recid)
    return new Ticket(counterparty, challenge, epoch, amount, winProb, channelIteration, signature)
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
    const amount = new Balance(new BN(components[3]))
    const winProb = new Hash(components[4])
    const channelIteration = new UINT256(new BN(components[4]))
    const signature = Signature.deserialize(components[5])
    return new Ticket(counterparty, challenge, epoch, amount, winProb, channelIteration, signature)
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
