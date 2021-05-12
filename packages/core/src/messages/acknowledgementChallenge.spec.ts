import { AcknowledgementChallenge } from './acknowledgementChallenge'
import { HalfKeyChallenge, sampleGroupElement } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import assert from 'assert'
import { randomBytes } from 'libp2p-crypto'

describe('test creation & verification of a challenge', function () {
  it('should create a verifiable challenge', async function () {
    const peerId = await PeerId.create({ keyType: 'secp256k1' })

    const [exponent, ackChallenge] = sampleGroupElement(true)

    const challenge = AcknowledgementChallenge.create(new HalfKeyChallenge(ackChallenge), peerId)

    assert(challenge.serialize().length == AcknowledgementChallenge.SIZE, `Size must be correct`)

    const deserializedChallenge = AcknowledgementChallenge.deserialize(
      challenge.serialize(),
      new HalfKeyChallenge(ackChallenge),
      peerId
    )

    assert(deserializedChallenge.solve(exponent), `Challenge must be solvable`)
  })

  it('should create a verifiable challenge - false positives', async function () {
    const peerId = await PeerId.create({ keyType: 'secp256k1' })

    const [_, ackChallenge] = sampleGroupElement(true)

    const challenge = AcknowledgementChallenge.create(new HalfKeyChallenge(ackChallenge), peerId)

    assert(challenge.serialize().length == AcknowledgementChallenge.SIZE, `Size must be correct`)

    assert.throws(() =>
      AcknowledgementChallenge.deserialize(
        randomBytes(AcknowledgementChallenge.SIZE),
        new HalfKeyChallenge(ackChallenge),
        peerId
      )
    )
  })
})
