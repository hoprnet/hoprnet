import { SECP256K1_CONSTANTS, HalfKeyChallenge, HalfKey } from '@hoprnet/hopr-utils'
import { HASH_ALGORITHM } from './constants'
import { ecdsaSign, ecdsaVerify } from 'secp256k1'
import { createHash } from 'crypto'

import type PeerId from 'peer-id'

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
      arr = Uint8Array.from(preArray)
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
    if (privKey.privKey == null) {
      throw Error(`Invalid arguments`)
    }

    const toSign = hashChallenge(ackChallenge)

    const signature = ecdsaSign(toSign, privKey.privKey.marshal())

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
  return ecdsaVerify(signature, hashChallenge(challenge), pubKey.pubKey.marshal())
}

function hashChallenge(ackChallenge: HalfKeyChallenge): Uint8Array {
  return createHash(HASH_ALGORITHM).update(ackChallenge.serialize()).digest()
}
