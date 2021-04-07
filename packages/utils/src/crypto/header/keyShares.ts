import { PRIVATE_KEY_LENGTH } from './constants'
import { generateKeyPair } from './utils'
import { privateKeyTweakMul, publicKeyTweakMul, publicKeyConvert, publicKeyVerify, privateKeyVerify } from 'secp256k1'
import { extract } from 'futoin-hkdf'

import type PeerId from 'peer-id'

export function generateKeyShares(path: PeerId[]): [alpha: Uint8Array, secrets: Uint8Array[]] {
  let done = false
  let secrets: Uint8Array[]

  let keyPair: [x: Uint8Array, alpha: Uint8Array]

  const product = new Uint8Array(PRIVATE_KEY_LENGTH)

  // Generate the Diffie-Hellman key shares and
  // the respective blinding factors for the
  // relays.
  // There exists a negligible, but NON-ZERO,
  // probability that the key share is chosen
  // such that it yields non-group elements.
  do {
    secrets = []

    keyPair = generateKeyPair()

    product.set(keyPair[0])

    for (const [index, peerId] of path.entries()) {
      let y = publicKeyTweakMul(publicKeyConvert(peerId.pubKey.marshal(), false), product, false)

      const secret = keyExtract(y, peerId.pubKey.marshal())

      if (!privateKeyVerify(secret)) {
        return
      }

      secrets.push(secret)

      product.set(privateKeyTweakMul(product, secret))

      if (index == path.length - 1) {
        done = true
      }
    }
  } while (!done)

  return [keyPair[1], secrets]
}

/**
 * @param alpha
 * @param key
 */
export function forwardTransform(alpha: Uint8Array, privKey: PeerId): [alpha: Uint8Array, secret: Uint8Array] {
  if (!publicKeyVerify(alpha) || privKey.privKey == null) {
    throw Error(`Invalid arguments`)
  }

  const key = keyExtract(publicKeyTweakMul(alpha, privKey.privKey.marshal(), false), privKey.pubKey.marshal())

  return [publicKeyTweakMul(alpha, key, false), key]
}

function keyExtract(groupElement: Uint8Array, pubKey: Uint8Array) {
  return extract('blake2s256', 32, Buffer.from(publicKeyConvert(groupElement)), Buffer.from(pubKey))
}
