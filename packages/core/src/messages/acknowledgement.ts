import { AcknowledgementChallenge } from './acknowledgementChallenge'
import { SECP256K1_CONSTANTS, u8aSplit, HalfKey } from '@hoprnet/hopr-utils'
import type { HalfKeyChallenge } from '@hoprnet/hopr-utils'
import { ecdsaSign, ecdsaVerify } from 'secp256k1'
import { SECRET_LENGTH, HASH_ALGORITHM } from './constants'
import { createHash } from 'crypto'
import type PeerId from 'peer-id'

export class Acknowledgement {
  private constructor(
    private ackSignature: Uint8Array,
    private challengeSignature: Uint8Array,
    public ackKeyShare: HalfKey
  ) {}

  static get SIZE(): number {
    return SECP256K1_CONSTANTS.SIGNATURE_LENGTH + AcknowledgementChallenge.SIZE + SECRET_LENGTH
  }

  static create(challenge: AcknowledgementChallenge, ackKey: HalfKey, privKey: PeerId): Acknowledgement {
    const toSign = Uint8Array.from([...challenge.serialize(), ...ackKey.serialize()])

    const signature = ecdsaSign(createHash(HASH_ALGORITHM).update(toSign).digest(), privKey.privKey.marshal())

    return new Acknowledgement(signature.signature, challenge.serialize(), ackKey)
  }

  static deserialize(preArray: Uint8Array, ownPubKey: PeerId, senderPubKey: PeerId): Acknowledgement {
    if (preArray.length != Acknowledgement.SIZE) {
      throw Error(`Invalid arguments`)
    }

    let arr: Uint8Array
    if (typeof Buffer !== 'undefined' && Buffer.isBuffer(preArray)) {
      arr = Uint8Array.from(arr)
    } else {
      arr = preArray
    }

    const [ackSignature, challengeSignature, ackKey] = u8aSplit(arr, [
      SECP256K1_CONSTANTS.SIGNATURE_LENGTH,
      AcknowledgementChallenge.SIZE,
      SECRET_LENGTH
    ])

    const challengeToVerify = createHash(HASH_ALGORITHM).update(new HalfKey(ackKey).toChallenge().serialize()).digest()

    if (!ecdsaVerify(challengeSignature, challengeToVerify, ownPubKey.pubKey.marshal())) {
      throw Error(`Challenge signature verification failed.`)
    }

    const ackToVerify = createHash(HASH_ALGORITHM)
      .update(Uint8Array.from([...challengeSignature, ...ackKey]))
      .digest()

    if (!ecdsaVerify(ackSignature, ackToVerify, senderPubKey.pubKey.marshal())) {
      throw Error(`Acknowledgement signature verification failed.`)
    }

    return new Acknowledgement(ackSignature, challengeSignature, new HalfKey(ackKey))
  }

  get ackChallenge(): HalfKeyChallenge {
    return this.ackKeyShare.toChallenge()
  }

  serialize(): Uint8Array {
    return Uint8Array.from([...this.ackSignature, ...this.challengeSignature, ...this.ackKeyShare.serialize()])
  }
}
