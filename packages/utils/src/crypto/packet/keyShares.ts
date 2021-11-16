import { HASH_ALGORITHM, HASH_LENGTH } from './constants'
import { SECP256K1_CONSTANTS } from '../constants'
import { sampleGroupElement } from '../sampleGroupElement'
import { privateKeyTweakMul, publicKeyTweakMul, publicKeyConvert, publicKeyVerify, privateKeyVerify } from 'secp256k1'
import { extract } from 'futoin-hkdf'

import type PeerId from 'peer-id'

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

  const product = new Uint8Array(SECP256K1_CONSTANTS.PRIVATE_KEY_LENGTH)

  do {
    secrets = []

    keyPair = sampleGroupElement()

    product.set(keyPair[0])

    for (const [index, peerId] of path.entries()) {
      let y = publicKeyTweakMul(publicKeyConvert(peerId.pubKey.marshal(), false), product, false)

      const secret = keyExtract(y, peerId.pubKey.marshal())

      if (!privateKeyVerify(secret)) {
        break
      }

      secrets.push(secret)

      product.set(privateKeyTweakMul(product, secret))

      if (!privateKeyVerify(product)) {
        break
      }

      if (index == path.length - 1) {
        done = true
      }
    }
  } while (!done)

  return { alpha: publicKeyConvert(keyPair[1]), secrets }
}

/**
 * Applies the forward transformation of the key shares to
 * an incoming packet.
 * @param alpha the group element used for the offline
 * Diffie-Hellman key exchange
 * @param privKey private key of the relayer
 */
export function forwardTransform(alpha: Uint8Array, privKey: PeerId): { alpha: Uint8Array; secret: Uint8Array } {
  if (!publicKeyVerify(alpha) || privKey.privKey == null) {
    throw Error(`Invalid arguments`)
  }

  const secret = keyExtract(publicKeyTweakMul(alpha, privKey.privKey.marshal(), false), privKey.pubKey.marshal())

  return { alpha: publicKeyConvert(publicKeyTweakMul(alpha, secret, false)), secret }
}

function keyExtract(groupElement: Uint8Array, pubKey: Uint8Array): Uint8Array {
  return extract(HASH_ALGORITHM, HASH_LENGTH, Buffer.from(publicKeyConvert(groupElement)), Buffer.from(pubKey))
}
