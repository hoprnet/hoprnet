import { SECRET_LENGTH } from './constants'
import { publicKeyCreate, privateKeyTweakAdd, publicKeyCombine, publicKeyTweakAdd } from 'secp256k1'
import { deriveAckKeyShare, deriveOwnKeyShare } from './keyDerivation'
import { SECP256K1_CONSTANTS } from '../constants'
import { Address, PublicKey } from '../../types'
import { u8aSplit } from '../../u8a'
import { randomBytes } from 'crypto'

export const POR_STRING_LENGTH = 2 * SECP256K1_CONSTANTS.COMPRESSED_PUBLIC_KEY_LENGTH

export { deriveAckKeyShare }

/**
 * Takes the secrets which the first and the second relayer are able
 * to derive from the packet header and computes the challenge for
 * the first ticket.
 * @param secretB shared secret with node +1
 * @param secretC shared secret with node +2
 * @returns the challenge for the first ticket sent to the first relayer
 */
export function createFirstChallenge(
  secretB: Uint8Array,
  secretC?: Uint8Array
): { ackChallenge: PublicKey; ticketChallenge: PublicKey; ownKey: Uint8Array } {
  if (secretB.length != SECRET_LENGTH || (secretC != undefined && secretC.length != SECRET_LENGTH)) {
    throw Error(`Invalid arguments`)
  }

  const s0 = deriveOwnKeyShare(secretB)
  const s1 = deriveAckKeyShare(secretC ?? randomBytes(SECRET_LENGTH))

  const ackChallenge = new PublicKey(publicKeyCreate(deriveAckKeyShare(secretB)))
  const ticketChallenge = new PublicKey(createChallenge(s0, s1))

  return { ackChallenge, ticketChallenge, ownKey: s0 }
}

/**
 * Creates the bitstring containing the PoR challenge for the next
 * downstream node as well as the hint that is used to verify the
 * challenge that is given to the relayer.
 * @param secretC shared secret with node +2
 * @param secretD shared secret with node +3
 * @returns the bitstring that is embedded next to the routing
 * information for each relayer
 */
export function createPoRString(secretC: Uint8Array, secretD?: Uint8Array): Uint8Array {
  if (secretC.length != SECRET_LENGTH || (secretD != undefined && secretD.length != SECRET_LENGTH)) {
    throw Error(`Invalid arguments`)
  }

  const s0 = deriveAckKeyShare(secretC)

  const s1 = deriveOwnKeyShare(secretC)
  const s2 = deriveAckKeyShare(secretD ?? randomBytes(SECRET_LENGTH))

  return Uint8Array.from([...createChallenge(s1, s2), ...publicKeyCreate(s0)])
}

type ValidOutput = {
  valid: true
  ownKey: Uint8Array
  ownShare: Uint8Array
  nextTicketChallenge: PublicKey
  ackChallenge: PublicKey
}

type InvalidOutput = {
  valid: false
}

/**
 * Verifies that an incoming packet contains all values that
 * are necessary to reconstruct the response to redeem the
 * incentive for relaying the packet
 * @param secret shared secret with the creator of the packet
 * @param porBytes PoR bitstring as included within the packet
 * @param challenge ticket challenge of the incoming ticket
 * @returns whether the challenge is derivable, if yes, it returns
 * the keyShare of the relayer as well as the secret that is used
 * to create it and the challenge for the next relayer.
 */
export function preVerify(secret: Uint8Array, porBytes: Uint8Array, challenge: Address): ValidOutput | InvalidOutput {
  if (secret.length != SECRET_LENGTH || porBytes.length != POR_STRING_LENGTH) {
    throw Error(`Invalid arguments`)
  }

  const [nextTicketChallenge, hint] = u8aSplit(porBytes, [
    SECP256K1_CONSTANTS.COMPRESSED_PUBLIC_KEY_LENGTH,
    SECP256K1_CONSTANTS.COMPRESSED_PUBLIC_KEY_LENGTH
  ])

  const ownKey = deriveOwnKeyShare(secret)
  const ownShare = publicKeyCreate(ownKey)

  const valid = new PublicKey(publicKeyCombine([ownShare, hint])).toAddress().eq(challenge)

  if (valid) {
    return { valid: true, ownKey, ownShare, nextTicketChallenge: new PublicKey(nextTicketChallenge), ackChallenge: new PublicKey(hint) }
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
  ownKey: Uint8Array | undefined,
  ack: Uint8Array | undefined,
  challenge: Address,
  ownShare?: Uint8Array | undefined,
  response?: Uint8Array
): { valid: true; response: Uint8Array } | { valid: false } {
  // clone ownKey before adding a tweak to it
  response = response ?? privateKeyTweakAdd(Uint8Array.from(ownKey), ack)

  let valid: boolean

  if (ownShare == undefined || ack == undefined) {
    valid = new PublicKey(publicKeyCreate(response)).toAddress().eq(challenge)
  } else {
    valid = new PublicKey(publicKeyTweakAdd(ownShare, ack)).toAddress().eq(challenge)
  }

  if (valid) {
    return { valid: true, response }
  } else {
    return { valid: false }
  }
}

function createChallenge(s0: Uint8Array, s1: Uint8Array) {
  // Clone s0 before adding tweak to it
  return publicKeyCreate(privateKeyTweakAdd(Uint8Array.from(s0), s1))
}
