import { createFirstChallenge, createPoRString, preVerify, validateAcknowledgement } from '.'
import { randomBytes } from 'crypto'
import { SECRET_LENGTH, COMPRESSED_PUBLIC_KEY_LENGTH } from './constants'
import { deriveAckKeyShare } from './keyDerivation'
// import { u8aSplit } from '../../u8a'
import assert from 'assert'
import { u8aEquals } from '../../u8a'

describe('PoR - proof of relay', function () {
  it('generate PoR string, preVerify, validate', function () {
    const AMOUNT = 4
    const secrets = Array.from({ length: AMOUNT }, (_) => randomBytes(SECRET_LENGTH))

    // Challenge generation
    const firstChallenge = createFirstChallenge(secrets)

    // To be included for first relayer
    const firstPorString = createPoRString(secrets)

    // To be included for second relayer
    const secondPorString = createPoRString(secrets.slice(1))

    // Computation result of the first relayer before
    // receiving an acknowledgement from the second relayer
    const result = preVerify(secrets[0], firstPorString, firstChallenge)

    assert(result.valid == true, `Challenge must be plausible`)

    // Simulates the transformation done by the first relayer
    assert(
      u8aEquals(result.nextChallenge, firstPorString.subarray(0, COMPRESSED_PUBLIC_KEY_LENGTH)),
      `Forward logic must extract correct challenge for next downstream node`
    )

    // Computes the cryptographic material that is part of
    // the acknowledgement
    const ack = deriveAckKeyShare(secrets[1])

    const validateResponseResult = validateAcknowledgement(result.ownShare, result.ownKey, ack, firstChallenge)

    assert(validateResponseResult.valid == true, `Acknowledgement must solve the challenge`)

    // Simulates the transformation as done by the
    // second relayer
    const secondResult = preVerify(secrets[1], secondPorString, result.nextChallenge)

    assert(secondResult.valid == true, `Second challenge must be plausible`)

    const secondAck = deriveAckKeyShare(secrets[2])

    const secondValidateResponseResult = validateAcknowledgement(
      secondResult.ownShare,
      secondResult.ownKey,
      secondAck,
      result.nextChallenge
    )

    assert(secondValidateResponseResult.valid == true, `Second acknowledgement must solve the challenge`)
  })
})
