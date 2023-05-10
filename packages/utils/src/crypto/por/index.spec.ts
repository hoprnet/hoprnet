import {
  createPoRValuesForSender,
  createPoRString,
  decodePoRBytes,
  preVerify,
  validatePoRHalfKeys,
  validatePoRHint,
  validatePoRResponse
} from './index.js'
import { Response } from '../../types.js'
import { randomBytes } from 'crypto'
import { deriveAckKeyShare } from './keyDerivation.js'
import assert from 'assert'
import { SECRET_LENGTH } from './constants.js'
import { u8aEquals } from '../../u8a/index.js'

describe('PoR - proof of relay', function () {
  it('generate PoR string, preVerify, validate', function () {
    const AMOUNT = 4
    const secrets = Array.from({ length: AMOUNT }, (_) => randomBytes(SECRET_LENGTH))

    // Challenge generation
    const firstChallenge = createPoRValuesForSender(secrets[0], secrets[1])

    // To be included for first relayer
    const firstPorString = createPoRString(secrets[1], secrets[2])

    // To be included for second relayer
    const secondPorString = createPoRString(secrets[2], secrets[3])

    // Computation result of the first relayer before
    // receiving an acknowledgement from the second relayer
    const result = preVerify(secrets[0], firstPorString, firstChallenge.ticketChallenge.to_ethereum_challenge())

    assert(result.valid == true, `Challenge must be plausible`)

    assert(result.ackChallenge.eq(deriveAckKeyShare(secrets[1]).to_challenge()))

    // Simulates the transformation done by the first relayer
    assert(
      u8aEquals(result.nextTicketChallenge.serialize(),decodePoRBytes(firstPorString).nextTicketChallenge.serialize()),
      `Forward logic must extract correct challenge for next downstream node`
    )

    // Computes the cryptographic material that is part of
    // the acknowledgement
    const ack = deriveAckKeyShare(secrets[1])

    assert(
      validatePoRHalfKeys(firstChallenge.ticketChallenge.to_ethereum_challenge(), result.ownKey, ack),
      `Acknowledgement must solve the challenge`
    )

    // Simulates the transformation as done by the
    // second relayer
    const secondResult = preVerify(secrets[1], secondPorString, result.nextTicketChallenge.to_ethereum_challenge())

    assert(secondResult.valid == true, `Second challenge must be plausible`)

    const secondAck = deriveAckKeyShare(secrets[2])

    assert(
      validatePoRHalfKeys(result.nextTicketChallenge.to_ethereum_challenge(), secondResult.ownKey, secondAck),
      `Second acknowledgement must solve the challenge`
    )

    assert(
      validatePoRHint(result.nextTicketChallenge.to_ethereum_challenge(), secondResult.ownShare, secondAck),
      `Second acknowledgement must solve the challenge`
    )
  })

  it('test functionality for unit tests', function () {
    const AMOUNT = 2
    const secrets = Array.from({ length: AMOUNT }, (_) => randomBytes(SECRET_LENGTH))

    const firstChallenge = createPoRValuesForSender(secrets[0], secrets[1])

    assert(
      validatePoRHalfKeys(
        firstChallenge.ticketChallenge.to_ethereum_challenge(),
        firstChallenge.ownKey,
        deriveAckKeyShare(secrets[1])
      ),
      `Challenge must be solved`
    )

    assert(
      validatePoRResponse(
        firstChallenge.ticketChallenge.to_ethereum_challenge(),
        Response.from_half_keys(firstChallenge.ownKey, deriveAckKeyShare(secrets[1]))
      ),
      `Returned response must solve the challenge`
    )
  })
})
