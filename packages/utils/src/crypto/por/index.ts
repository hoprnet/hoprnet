import { SECRET_LENGTH } from './constants'
import { publicKeyCreate, privateKeyTweakAdd, publicKeyCombine, publicKeyTweakAdd } from 'secp256k1'
import { deriveAckKeyShare, deriveOwnKeyShare } from './keyDerivation'
import { COMPRESSED_PUBLIC_KEY_LENGTH } from './constants'
import { u8aEquals } from '../../u8a'
import { randomBytes } from 'crypto'

const POR_STRING_LENGTH = 2 * COMPRESSED_PUBLIC_KEY_LENGTH

export function createFirstChallenge(secrets: Uint8Array[]) {
  if (secrets.length < 2 || secrets.some((secret) => secret.length != SECRET_LENGTH)) {
    throw Error(`Invalid arguments`)
  }

  const s0 = deriveOwnKeyShare(secrets[0])
  const s1 = deriveAckKeyShare(secrets[1])

  return createChallenge(s0, s1)
}

export function createPoR(secrets: Uint8Array[]) {
  if (secrets.length < 2 || secrets.some((s) => s.length != SECRET_LENGTH)) {
    throw Error(`Invalid arguments`)
  }

  const s0 = deriveAckKeyShare(secrets[1])

  const s1 = deriveOwnKeyShare(secrets[1])
  const s2 = deriveAckKeyShare(secrets.length >= 3 ? secrets[2] : randomBytes(SECRET_LENGTH))

  return Uint8Array.from([...createChallenge(s1, s2), ...publicKeyCreate(s0)])
}

export function preVerify(
  secret: Uint8Array,
  porBytes: Uint8Array,
  challenge: Uint8Array
): [valid: true, ownShare: Uint8Array, ownKey: Uint8Array, nextChallenge: Uint8Array] | [valid: false] {
  if (secret.length != SECRET_LENGTH || porBytes.length != POR_LENGTH) {
    throw Error(`Invalid arguments`)
  }

  const [nextChallenge, hint] = [
    porBytes.subarray(0, COMPRESSED_PUBLIC_KEY_LENGTH),
    porBytes.subarray(COMPRESSED_PUBLIC_KEY_LENGTH, COMPRESSED_PUBLIC_KEY_LENGTH + COMPRESSED_PUBLIC_KEY_LENGTH)
  ]

  const ownKey = deriveOwnKeyShare(secret)
  const ownShare = publicKeyCreate(ownKey)

  const valid = u8aEquals(publicKeyCombine([ownShare, hint]), challenge)

  if (valid) {
    return [true, ownKey, ownShare, nextChallenge]
  } else {
    return [false]
  }
}

export function validateAcknowledgement(
  ownShare: Uint8Array,
  ownKey: Uint8Array,
  ack: Uint8Array,
  challenge: Uint8Array
): [valid: true, response: Uint8Array] | [valid: false] {
  const valid = u8aEquals(publicKeyTweakAdd(ownShare, ack), challenge)

  if (valid) {
    // clone ownKey before
    const response = privateKeyTweakAdd(ownKey.slice(), ack)
    return [true, response]
  } else {
    return [false]
  }
}

function createChallenge(s0: Uint8Array, s1: Uint8Array) {
  return publicKeyCreate(privateKeyTweakAdd(s0, s1))
}
