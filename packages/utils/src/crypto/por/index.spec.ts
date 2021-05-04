import { createFirstChallenge, createPoRString, preVerify, validateAcknowledgement } from '.'
import { randomBytes } from 'crypto'
import { SECRET_LENGTH } from './constants'
import { SECP256K1_CONSTANTS } from '../constants'
import { deriveAckKeyShare } from './keyDerivation'
import assert from 'assert'
import { u8aEquals } from '../../u8a'
import { publicKeyCreate } from 'secp256k1'
import { PublicKey } from '../../types'

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
    const result = preVerify(
      secrets[0],
      firstPorString,
      new PublicKey(firstChallenge.ticketChallenge).toAddress().serialize()
    )

    assert(result.valid == true, `Challenge must be plausible`)

    assert(u8aEquals(result.ackChallenge, publicKeyCreate(deriveAckKeyShare(secrets[1]))))

    // Simulates the transformation done by the first relayer
    assert(
      u8aEquals(
        result.nextTicketChallenge,
        firstPorString.subarray(0, SECP256K1_CONSTANTS.COMPRESSED_PUBLIC_KEY_LENGTH)
      ),
      `Forward logic must extract correct challenge for next downstream node`
    )

    // Computes the cryptographic material that is part of
    // the acknowledgement
    const ack = deriveAckKeyShare(secrets[1])

    const validateResponseResult = validateAcknowledgement(
      result.ownKey,
      ack,
      firstChallenge.ticketChallenge,
      result.ownShare
    )

    assert(validateResponseResult.valid == true, `Acknowledgement must solve the challenge`)

    assert(
      validateAcknowledgement(result.ownKey, ack, firstChallenge.ticketChallenge).valid,
      `Should be valid also without group element`
    )

    // Simulates the transformation as done by the
    // second relayer
    const secondResult = preVerify(
      secrets[1],
      secondPorString,
      new PublicKey(result.nextTicketChallenge).toAddress().serialize()
    )

    assert(secondResult.valid == true, `Second challenge must be plausible`)

    const secondAck = deriveAckKeyShare(secrets[2])

    const secondValidateResponseResult = validateAcknowledgement(
      secondResult.ownKey,
      secondAck,
      result.nextTicketChallenge,
      secondResult.ownShare
    )

    assert(secondValidateResponseResult.valid == true, `Second acknowledgement must solve the challenge`)
  })

  it('test functionality for unit tests', function () {
    const AMOUNT = 2
    const secrets = Array.from({ length: AMOUNT }, (_) => randomBytes(SECRET_LENGTH))

    const firstChallenge = createFirstChallenge(secrets)

    const validateResult = validateAcknowledgement(
      firstChallenge.ownKey,
      deriveAckKeyShare(secrets[1]),
      firstChallenge.ticketChallenge
    )

    assert(validateResult.valid == true, `Challenge must be solved`)

    assert(
      validateAcknowledgement(
        undefined,
        deriveAckKeyShare(secrets[1]),
        firstChallenge.ticketChallenge,
        undefined,
        validateResult.response
      ).valid == true,
      `Returned response must solve the challenge`
    )
  })
})
