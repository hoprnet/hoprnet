import { Challenge } from './challenge'
import { deriveAckKeyShare, PublicKey } from '@hoprnet/hopr-utils'
import { ecdsaSign, ecdsaVerify, publicKeyCreate } from 'secp256k1'
import { SECRET_LENGTH, HASH_ALGORITHM } from './constants'
import { SECP256K1, u8aSplit } from '@hoprnet/hopr-utils'
import { createHash } from 'crypto'
import type PeerId from 'peer-id'

export class Acknowledgement {
  private constructor(
    private ackSignature: Uint8Array,
    private challengeSignature: Uint8Array,
    public ackKeyShare: Uint8Array
  ) {}

  static get SIZE() {
    return SECP256K1.SIGNATURE_LENGTH + Challenge.SIZE + SECRET_LENGTH
  }

  static create(challenge: Challenge, derivedSecret: Uint8Array, privKey: PeerId) {
    const ackKeyShare = deriveAckKeyShare(derivedSecret)
    const toSign = Uint8Array.from([...challenge.serialize(), ...ackKeyShare])

    const signature = ecdsaSign(createHash(HASH_ALGORITHM).update(toSign).digest(), privKey.privKey.marshal())

    return new Acknowledgement(signature.signature, challenge.serialize(), deriveAckKeyShare(derivedSecret))
  }

  static deserialize(preArray: Uint8Array, ownPubKey: PeerId, senderPubKey: PeerId) {
    if (preArray.length != Acknowledgement.SIZE) {
      throw Error(`Invalid arguments`)
    }

    let arr: Uint8Array
    if (typeof Buffer !== 'undefined' && Buffer.isBuffer(preArray)) {
      arr = Uint8Array.from(arr)
    } else {
      arr = preArray
    }

    const [ackSignature, challengeSignature, ackKeyShare] = u8aSplit(arr, [
      SECP256K1.SIGNATURE_LENGTH,
      SECP256K1.SIGNATURE_LENGTH,
      SECRET_LENGTH
    ])

    const challengeToVerify = createHash(HASH_ALGORITHM).update(getAckChallenge(ackKeyShare)).digest()

    if (!ecdsaVerify(challengeSignature, challengeToVerify, ownPubKey.pubKey.marshal())) {
      throw Error(`General error.`)
    }

    const ackToVerify = createHash(HASH_ALGORITHM)
      .update(Uint8Array.from([...challengeSignature, ...ackKeyShare]))
      .digest()

    if (!ecdsaVerify(ackSignature, ackToVerify, senderPubKey.pubKey.marshal())) {
      throw Error(`General error.`)
    }

    return new Acknowledgement(ackSignature, challengeSignature, ackKeyShare)
  }

  get ackChallenge() {
    return new PublicKey(getAckChallenge(this.ackKeyShare))
  }

  serialize() {
    return Uint8Array.from([...this.ackSignature, ...this.challengeSignature, ...this.ackKeyShare])
  }
}

function getAckChallenge(ackKeyShare: Uint8Array) {
  return publicKeyCreate(ackKeyShare)
}
