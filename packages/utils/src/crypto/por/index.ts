//import { SECRET_LENGTH } from './constants'
import { deriveAckKeyShare, deriveOwnKeyShare } from './keyDerivation'
import { SECP256K1_CONSTANTS } from '../constants'
import { HalfKeyChallenge, HalfKey, Challenge, Response, EthereumChallenge } from '../../types'
import { u8aSplit } from '../../u8a'
import { randomBytes } from 'crypto'
import { SECRET_LENGTH } from './constants'

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
export function createPoRValuesForSender(
  secretB: Uint8Array,
  secretC?: Uint8Array
): { ackChallenge: HalfKeyChallenge; ticketChallenge: Challenge; ownKey: HalfKey } {
  if (secretB.length != SECRET_LENGTH || (secretC != undefined && secretC.length != SECRET_LENGTH)) {
    throw Error(`Invalid arguments`)
  }

  const s0 = deriveOwnKeyShare(secretB)
  const s1 = deriveAckKeyShare(secretC ?? randomBytes(SECRET_LENGTH))

  const ackChallenge = deriveAckKeyShare(secretB).toChallenge()
  const ticketChallenge = Response.fromHalfKeys(s0, s1).toChallenge()

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

  return Uint8Array.from([...createChallenge(s1, s2).serialize(), ...s0.toChallenge().serialize()])
}

type ValidOutput = {
  valid: true
  ownKey: HalfKey
  ownShare: HalfKeyChallenge
  nextTicketChallenge: Challenge
  ackChallenge: HalfKeyChallenge
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
export function preVerify(
  secret: Uint8Array,
  porBytes: Uint8Array,
  challenge: EthereumChallenge
): ValidOutput | InvalidOutput {
  if (secret.length != SECRET_LENGTH || porBytes.length != POR_STRING_LENGTH) {
    throw Error(`Invalid arguments`)
  }

  const { nextTicketChallenge, ackChallenge } = decodePoRBytes(porBytes)

  const ownKey = deriveOwnKeyShare(secret)
  const ownShare = ownKey.toChallenge()

  const valid = Challenge.fromHintAndShare(ownShare, ackChallenge).toEthereumChallenge().eq(challenge)

  if (valid) {
    return {
      valid: true,
      ownKey,
      ownShare,
      nextTicketChallenge,
      ackChallenge
    }
  } else {
    return { valid: false }
  }
}

export function decodePoRBytes(porBytes: Uint8Array): {
  nextTicketChallenge: Challenge
  ackChallenge: HalfKeyChallenge
} {
  const [nextTicketChallenge, hint] = u8aSplit(porBytes, [
    SECP256K1_CONSTANTS.COMPRESSED_PUBLIC_KEY_LENGTH,
    SECP256K1_CONSTANTS.COMPRESSED_PUBLIC_KEY_LENGTH
  ])

  return {
    nextTicketChallenge: new Challenge(nextTicketChallenge),
    ackChallenge: new HalfKeyChallenge(hint)
  }
}

// @TODO add description
export function validatePoRHalfKeys(ethereumChallenge: EthereumChallenge, ownKey: HalfKey, ack: HalfKey): boolean {
  return Response.fromHalfKeys(ownKey, ack).toChallenge().toEthereumChallenge().eq(ethereumChallenge)
}

// @TODO add description
export function validatePoRResponse(ethereumChallenge: EthereumChallenge, response: Response): boolean {
  return response.toChallenge().toEthereumChallenge().eq(ethereumChallenge)
}

// @TODO add description
export function validatePoRHint(
  ethereumChallenge: EthereumChallenge,
  ownShare: HalfKeyChallenge,
  ack: HalfKey
): boolean {
  return Challenge.fromOwnShareAndHalfKey(ownShare, ack).toEthereumChallenge().eq(ethereumChallenge)
}

function createChallenge(s0: HalfKey, s1: HalfKey): Challenge {
  return Response.fromHalfKeys(s0, s1).toChallenge()
}
