import { SECP256K1_CONSTANTS, PublicKey } from '@hoprnet/hopr-utils'
import { HASH_ALGORITHM } from './constants'
import { ecdsaSign, ecdsaVerify, publicKeyCreate } from 'secp256k1'
import { createHash } from 'crypto'

import type PeerId from 'peer-id'

export class Challenge {
  private constructor(private ackChallenge: PublicKey, private signature: Uint8Array) {}

  static get SIZE(): number {
    return SECP256K1_CONSTANTS.SIGNATURE_LENGTH
  }

  static deserialize(preArray: Uint8Array | Buffer, ackChallenge: PublicKey, pubKey: PeerId): Challenge {
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

    return new Challenge(ackChallenge, arr)
  }

  serialize(): Uint8Array {
    return Uint8Array.from(this.signature)
  }

  static create(ackChallenge: PublicKey, privKey: PeerId): Challenge {
    if (privKey.privKey == null) {
      throw Error(`Invalid arguments`)
    }

    const toSign = hashChallenge(ackChallenge)

    const signature = ecdsaSign(toSign, privKey.privKey.marshal())

    return new Challenge(ackChallenge, signature.signature)
  }

  solve(secret: Uint8Array): boolean {
    return this.ackChallenge.eq(new PublicKey(publicKeyCreate(secret)))
  }
}

function verifyChallenge(pubKey: PeerId, signature: Uint8Array, challenge: PublicKey): boolean {
  return ecdsaVerify(signature, hashChallenge(challenge), pubKey.pubKey.marshal())
}

function hashChallenge(ackChallenge: PublicKey): Uint8Array {
  return createHash(HASH_ALGORITHM).update(ackChallenge.serialize()).digest()
}
