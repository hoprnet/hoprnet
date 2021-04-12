import secp256k1 from 'secp256k1'
import { deriveTicketKeyBlinding } from '../packet/header'
import { KEY_LENGTH } from '../packet/header/parameters'
import { Challenge } from '../packet/challenge'
import { Hash, Signature, PublicKey } from '@hoprnet/hopr-core-ethereum'
import PeerId from 'peer-id'
import { serializeToU8a, u8aSplit, u8aToHex } from '@hoprnet/hopr-utils'
import { UnAcknowledgedTickets } from '../../dbKeys'

/**
 * This class encapsulates the message that is sent back to the relayer
 * and allows that party to compute the key that is necessary to redeem
 * the previously received transaction.
 */
class AcknowledgementMessage {
  constructor(readonly key: Uint8Array, readonly challenge: Challenge, readonly signature?: Signature) {}

  static deserialize(arr: Uint8Array) {
    const components = u8aSplit(arr, [KEY_LENGTH, Challenge.SIZE(), Signature.SIZE])
    return new AcknowledgementMessage(
      components[0],
      new Challenge({ bytes: components[1], offset: components[1].byteOffset }),
      Signature.deserialize(components[2])
    )
  }

  serialize(): Uint8Array {
    return serializeToU8a([
      [this.key, KEY_LENGTH],
      [this.challenge, Challenge.SIZE()],
      [this.signature.serialize(), Signature.SIZE]
    ])
  }

  getHashedKey(): Hash {
    return Hash.create(this.key)
  }

  getKey(): string {
    return u8aToHex(UnAcknowledgedTickets(this.getHashedKey().serialize()))
  }

  get responseSigningParty(): Uint8Array {
    const hash = Hash.createChallenge(this.key, this.challenge.serialize())
    return secp256k1.ecdsaRecover(this.signature.signature, this.signature.recovery, hash.serialize())
  }

  async verify(peerId: PeerId): Promise<boolean> {
    const hash = Hash.createChallenge(this.key, this.challenge.serialize())
    return this.signature.verify(hash.serialize(), new PublicKey(peerId.pubKey.marshal()))
  }

  /**
   * Takes a challenge from a relayer and returns an acknowledgement that includes a
   * signature over the requested key half.
   *
   * @param challenge the signed challenge of the relayer
   * @param derivedSecret the secret that is used to create the second key half
   * @param signer contains private key
   */
  static async create(
    challenge: Challenge,
    derivedSecret: Uint8Array,
    signer: PeerId
  ): Promise<AcknowledgementMessage> {
    const key = deriveTicketKeyBlinding(derivedSecret)
    const hash = Hash.createChallenge(key, challenge.serialize())
    const signature = Signature.create(hash.serialize(), signer.privKey.marshal())
    return new AcknowledgementMessage(key, challenge, signature)
  }

  static SIZE(): number {
    return KEY_LENGTH + Challenge.SIZE() + Signature.SIZE
  }
}

export { AcknowledgementMessage }
