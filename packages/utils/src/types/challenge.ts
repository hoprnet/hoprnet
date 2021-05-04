import { u8aEquals, SECP256K1_CONSTANTS } from '..'
import { HASH_ALGORITHM } from '../crypto'
import { ecdsaSign, ecdsaVerify, publicKeyCreate } from 'secp256k1'
import { createHash } from 'crypto'

import type PeerId from 'peer-id'

export class Challenge {
  private constructor(private ackChallenge: Uint8Array, private signature: Uint8Array) {}

  static get SIZE(): number {
    return SECP256K1_CONSTANTS.SIGNATURE_LENGTH
  }

  static deserialize(preArray: Uint8Array | Buffer, ackChallenge: Uint8Array, pubKey: PeerId): Challenge {
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

  static create(ackChallenge: Uint8Array, privKey: PeerId): Challenge {
    if (ackChallenge.length != SECP256K1_CONSTANTS.COMPRESSED_PUBLIC_KEY_LENGTH || privKey.privKey == null) {
      throw Error(`Invalid arguments`)
    }

    const toSign = hashChallenge(ackChallenge)

    const signature = ecdsaSign(toSign, privKey.privKey.marshal())

    return new Challenge(ackChallenge, signature.signature)
  }

  solve(secret: Uint8Array): boolean {
    return u8aEquals(publicKeyCreate(secret), this.ackChallenge)
  }
}

function verifyChallenge(pubKey: PeerId, signature: Uint8Array, challenge: Uint8Array): boolean {
  return ecdsaVerify(signature, hashChallenge(challenge), pubKey.pubKey.marshal())
}

function hashChallenge(ackChallenge: Uint8Array): Uint8Array {
  return createHash(HASH_ALGORITHM).update(ackChallenge).digest()
}
