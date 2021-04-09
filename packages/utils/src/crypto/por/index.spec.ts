import { createFirstChallenge, createPoR, preVerify, validateAcknowledgement } from '.'
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
    const firstPorString = createPoR(secrets)

    // To be included for second relayer
    const secondPorString = createPoR(secrets.slice(1))

    // Computation result of the first relayer before
    // receiving an acknowledgement from the second relayer
    const result = preVerify(secrets[0], firstPorString, firstChallenge)

    assert(result[0], `Challenge must be plausible`)

    // Simulates the transformation done by the first relayer
    assert(
      u8aEquals(result[3], firstPorString.subarray(0, COMPRESSED_PUBLIC_KEY_LENGTH)),
      `Forward logic must extract correct challenge for next downstream node`
    )

    // Computes the cryptographic material that is part of
    // the acknowledgement
    const ack = deriveAckKeyShare(secrets[1])

    const validateResponseResult = validateAcknowledgement(result[2], result[1], ack, firstChallenge)

    assert(validateResponseResult[0], `Acknowledgement must solve the challenge`)

    // Simulates the transformation as done by the
    // second relayer
    const secondResult = preVerify(secrets[1], secondPorString, result[3])

    assert(secondResult[0], `Second challenge must be plausible`)

    const secondAck = deriveAckKeyShare(secrets[2])

    const secondValidateResponseResult = validateAcknowledgement(secondResult[2], secondResult[1], secondAck, result[3])

    assert(secondValidateResponseResult[0], `Second acknowledgement must solve the challenge`)
  })
})
