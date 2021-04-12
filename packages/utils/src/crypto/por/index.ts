import { SECRET_LENGTH } from './constants'
import { publicKeyCreate, privateKeyTweakAdd, publicKeyCombine, publicKeyTweakAdd } from 'secp256k1'
import { deriveAckKeyShare, deriveOwnKeyShare } from './keyDerivation'
import { COMPRESSED_PUBLIC_KEY_LENGTH } from './constants'
import { u8aEquals } from '../../u8a'
import { randomBytes } from 'crypto'

export const POR_STRING_LENGTH = 2 * COMPRESSED_PUBLIC_KEY_LENGTH

/**
 * Takes the secrets which the first and the second relayer are able
 * to derive from the packet header and computes the challenge for
 * the first ticket.
 * @param secrets shared secrets with creator of the packet
 * @returns the challenge first the ticket sent to the first relayer
 */
export function createFirstChallenge(secrets: Uint8Array[]) {
  if (secrets.length < 2 || secrets.some((secret) => secret.length != SECRET_LENGTH)) {
    throw Error(`Invalid arguments`)
  }

  const s0 = deriveOwnKeyShare(secrets[0])
  const s1 = deriveAckKeyShare(secrets[1])

  return createChallenge(s0, s1)
}

/**
 * Creates the bitstring containing the PoR challenge for the next
 * downstream node as well as the hint that is used to verify the
 * challenge that is given to the relayer.
 * @param secrets shared secrets with the creator of the packet
 * @returns the bitstring that is embedded next to the routing
 * information for each relayer
 */
export function createPoRString(secrets: Uint8Array[]) {
  if (secrets.length < 2 || secrets.some((s) => s.length != SECRET_LENGTH)) {
    throw Error(`Invalid arguments`)
  }

  const s0 = deriveAckKeyShare(secrets[1])

  const s1 = deriveOwnKeyShare(secrets[1])
  const s2 = deriveAckKeyShare(secrets.length >= 3 ? secrets[2] : randomBytes(SECRET_LENGTH))

  return Uint8Array.from([...createChallenge(s1, s2), ...publicKeyCreate(s0)])
}

/**
 * Verifies whether an incoming packet contains all values that
 * are necessary to reconstruct the response to redeem the
 * incentive for relaying the packet
 * @param secret shared secret with the creator of the packet
 * @param porBytes PoR bitstring as included within the packet
 * @param challenge ticket challenge of the incoming ticket
 * @returns whether the challenge is derivable, if yes, it returns
 * the keyShare of the relayer as well as the secret that is used
 * to create it and the challenge for the next relayer.
 */
export function preVerify(
  secret: Uint8Array,
  porBytes: Uint8Array,
  challenge: Uint8Array
): { valid: true; ownShare: Uint8Array; ownKey: Uint8Array; nextChallenge: Uint8Array } | { valid: false } {
  if (secret.length != SECRET_LENGTH || porBytes.length != POR_STRING_LENGTH) {
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
    return { valid: true, ownKey, ownShare, nextChallenge }
  } else {
    return { valid: false }
  }
}

/**
 * Takes an the second key share and reconstructs the secret
 * that is necessary to redeem the incentive for relaying the
 * packet.
 * @param ownShare own key share as computed from the packet
 * @param ownKey key that as derived from the shared secret with
 * the creator of the packet
 * @param ack second key share as given by the acknowledgement
 * @param challenge challenge of the ticket
 * @returns whether the input values led to a valid response that
 * can be used to redeem the incentive
 */
export function validateAcknowledgement(
  ownShare: Uint8Array,
  ownKey: Uint8Array,
  ack: Uint8Array,
  challenge: Uint8Array
): { valid: true; response: Uint8Array } | { valid: false } {
  const valid = u8aEquals(publicKeyTweakAdd(ownShare, ack), challenge)

  if (valid) {
    // clone ownKey before adding a tweak to it
    const response = privateKeyTweakAdd(ownKey.slice(), ack)
    return { valid: true, response }
  } else {
    return { valid: false }
  }
}

function createChallenge(s0: Uint8Array, s1: Uint8Array) {
  return publicKeyCreate(privateKeyTweakAdd(s0, s1))
}
