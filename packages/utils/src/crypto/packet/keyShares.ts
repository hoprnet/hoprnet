import { HASH_ALGORITHM, HASH_LENGTH } from './constants'
import { SECP256K1_CONSTANTS } from '../constants'
import { sampleGroupElement } from '../sampleGroupElement'
import { privateKeyTweakMul, publicKeyTweakMul, publicKeyConvert, publicKeyVerify, privateKeyVerify } from 'secp256k1'

import type PeerId from 'peer-id'
import hkdf from 'futoin-hkdf'

/**
 * Performs an offline Diffie-Hellman key exchange with
 * the nodes along the given path
 * @param path the path to use for the mixnet packet
 * @returns the first group element and the shared secrets
 * with the nodes along the path
 */
export function generateKeyShares(path: PeerId[]): { alpha: Uint8Array; secrets: Uint8Array[] } {
  let done = false
  let secrets: Uint8Array[]

  let keyPair: [x: Uint8Array, alpha: Uint8Array]

  const coeff_prev = new Uint8Array(SECP256K1_CONSTANTS.PRIVATE_KEY_LENGTH) // This becomes: x * b_0 * b_1 * b_2 * ...
  const alpha_prev = new Uint8Array(SECP256K1_CONSTANTS.UNCOMPRESSED_PUBLIC_KEY_LENGTH) // This becomes: x * b_0 * b_1 * b_2 * ... * G

  do {
    secrets = []

    // NOTE: we're keeping alpha uncompressed during computation for better performance
    keyPair = sampleGroupElement(false)

    coeff_prev.set(keyPair[0]) // x
    alpha_prev.set(keyPair[1]) // alpha_0 = x*G

    for (const [k, peerId] of path.entries()) {
      // Compute the shared group element and extract keying material as a shared secret
      const s_k = publicKeyTweakMul(peerId.pubKey.marshal(), coeff_prev, true)
      secrets.push(keyExtract(s_k, peerId.pubKey.marshal()))

      // If this was the last shared secret, no need to compute anymore
      if (k == path.length - 1) {
        done = true
        break
      }

      // Compute the new blinding factor b_k (alpha needs compressing, s_k is already compressed)
      const b_k = fullKdf(s_k, publicKeyConvert(alpha_prev, true)) // KDF(secret, salt)

      // NOTE: This check would not be needed on modern curves
      if (!privateKeyVerify(b_k)) {
        break
      }

      // Accumulate the new blinding factor b_k in the coeff_prev
      coeff_prev.set(privateKeyTweakMul(coeff_prev, b_k))

      if (!privateKeyVerify(coeff_prev)) {
        break
      }

      // Also update alpha_prev with the new blinding factor b_k, keep alpha uncompressed
      alpha_prev.set(publicKeyTweakMul(alpha_prev, b_k, false))
    }
  } while (!done)

  return { alpha: publicKeyConvert(keyPair[1]), secrets }
}

/**
 * Applies the forward transformation of the key shares to
 * an incoming packet.
 * @param alpha the group element used for the offline
 * Diffie-Hellman key exchange (compressed EC point)
 * @param peerId id of the relayer
 * @return Next public key (compressed EC point) and derived secret
 */
export function forwardTransform(alpha: Uint8Array, peerId: PeerId): { alpha: Uint8Array; secret: Uint8Array } {
  if (!publicKeyVerify(alpha) || peerId.privKey == null || peerId.pubKey == null) {
    throw Error(`Invalid arguments`)
  }

  const s_k = publicKeyTweakMul(alpha, peerId.privKey.marshal(), true)
  const b_k = fullKdf(s_k, alpha)

  return {
    alpha: publicKeyTweakMul(alpha, b_k, true), // advance alpha by the blinding factor
    secret: keyExtract(s_k, peerId.pubKey.marshal()) // extract keying material from the group element
  }
}

function fullKdf(secret: Uint8Array, pubKey: Uint8Array): Uint8Array {
  return hkdf(Buffer.from(secret), HASH_LENGTH, { hash: HASH_ALGORITHM, salt: Buffer.from(pubKey) })
}

function keyExtract(groupElement: Uint8Array, pubKey: Uint8Array): Uint8Array {
  return hkdf.extract(HASH_ALGORITHM, HASH_LENGTH, Buffer.from(groupElement), Buffer.from(pubKey))
}
