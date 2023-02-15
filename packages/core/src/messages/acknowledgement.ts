import { AcknowledgementChallenge } from './acknowledgementChallenge.js'
import { SECP256K1_CONSTANTS, u8aSplit, HalfKey } from '@hoprnet/hopr-utils'
import type { HalfKeyChallenge } from '@hoprnet/hopr-utils'
import secp256k1 from 'secp256k1'
import { SECRET_LENGTH, HASH_ALGORITHM } from './constants.js'
import { createHash } from 'crypto'
import type { PeerId } from '@libp2p/interface-peer-id'
import { keysPBM, unmarshalPublicKey } from '@libp2p/crypto/keys'

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

    const signature = secp256k1.ecdsaSign(
      createHash(HASH_ALGORITHM).update(toSign).digest(),
      keysPBM.PrivateKey.decode(privKey.privateKey).Data
    )

    return new Acknowledgement(signature.signature, challenge.serialize(), ackKey)
  }

  static deserialize(preArray: Uint8Array, ownPubKey: PeerId, senderPubKey: PeerId): Acknowledgement {
    if (preArray.length != Acknowledgement.SIZE) {
      throw Error(`Invalid arguments`)
    }

    let arr: Uint8Array
    if (typeof Buffer !== 'undefined' && Buffer.isBuffer(preArray)) {
      arr = new Uint8Array(preArray.buffer, preArray.byteOffset, preArray.byteLength)
    } else {
      arr = preArray
    }

    const [ackSignature, challengeSignature, ackKey] = u8aSplit(arr, [
      SECP256K1_CONSTANTS.SIGNATURE_LENGTH,
      AcknowledgementChallenge.SIZE,
      SECRET_LENGTH
    ])

    const challengeToVerify = createHash(HASH_ALGORITHM).update(new HalfKey(ackKey).toChallenge().serialize()).digest()

    if (
      !secp256k1.ecdsaVerify(challengeSignature, challengeToVerify, unmarshalPublicKey(ownPubKey.publicKey).marshal())
    ) {
      throw Error(`Challenge signature verification failed.`)
    }

    const ackToVerify = createHash(HASH_ALGORITHM)
      .update(Uint8Array.from([...challengeSignature, ...ackKey]))
      .digest()

    if (!secp256k1.ecdsaVerify(ackSignature, ackToVerify, unmarshalPublicKey(senderPubKey.publicKey).marshal())) {
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
