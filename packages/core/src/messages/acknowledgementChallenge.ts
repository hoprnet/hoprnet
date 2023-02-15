import { SECP256K1_CONSTANTS, HalfKeyChallenge, HalfKey } from '@hoprnet/hopr-utils'
import { HASH_ALGORITHM } from './constants.js'
import secp256k1 from 'secp256k1'

import { createHash } from 'crypto'

import type { PeerId } from '@libp2p/interface-peer-id'
import { keysPBM, unmarshalPublicKey } from '@libp2p/crypto/keys'

export class AcknowledgementChallenge {
  private constructor(private ackChallenge: HalfKeyChallenge, private signature: Uint8Array) {}

  static get SIZE(): number {
    return SECP256K1_CONSTANTS.SIGNATURE_LENGTH
  }

  static deserialize(
    preArray: Uint8Array | Buffer,
    ackChallenge: HalfKeyChallenge,
    pubKey: PeerId
  ): AcknowledgementChallenge {
    if (preArray.length != SECP256K1_CONSTANTS.SIGNATURE_LENGTH) {
      throw Error(`Invalid arguments`)
    }

    let arr: Uint8Array
    if (typeof Buffer !== 'undefined' && Buffer.isBuffer(preArray)) {
      arr = new Uint8Array(preArray.buffer, preArray.byteOffset, preArray.byteLength)
    } else {
      arr = preArray
    }

    if (!verifyChallenge(pubKey, arr, ackChallenge)) {
      throw Error(`Challenge is not derivable.`)
    }

    return new AcknowledgementChallenge(ackChallenge, arr)
  }

  serialize(): Uint8Array {
    return Uint8Array.from(this.signature)
  }

  static create(ackChallenge: HalfKeyChallenge, privKey: PeerId): AcknowledgementChallenge {
    if (!privKey.privateKey) {
      throw Error(`Invalid arguments`)
    }

    const toSign = hashChallenge(ackChallenge)

    const signature = secp256k1.ecdsaSign(toSign, keysPBM.PrivateKey.decode(privKey.privateKey).Data)

    return new AcknowledgementChallenge(ackChallenge, signature.signature)
  }

  solve(secret: Uint8Array): boolean {
    return this.ackChallenge.eq(new HalfKey(secret).toChallenge())
  }

  clone(): AcknowledgementChallenge {
    return new AcknowledgementChallenge(this.ackChallenge.clone(), this.signature.slice())
  }
}

function verifyChallenge(pubKey: PeerId, signature: Uint8Array, challenge: HalfKeyChallenge): boolean {
  return secp256k1.ecdsaVerify(signature, hashChallenge(challenge), unmarshalPublicKey(pubKey.publicKey).marshal())
}

function hashChallenge(ackChallenge: HalfKeyChallenge): Uint8Array {
  return createHash(HASH_ALGORITHM).update(ackChallenge.serialize()).digest()
}
