import { Challenge } from './challenge'
import { sampleGroupElement } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import assert from 'assert'

describe('test creation & verification of a challenge', function () {
  it('should create a verifiable challenge', async function () {
    const peerId = await PeerId.create({ keyType: 'secp256k1' })

    const [exponent, ackChallenge] = sampleGroupElement(true)

    const challenge = Challenge.create(ackChallenge, peerId)

    assert(challenge.serialize().length == Challenge.SIZE, `Size must be correct`)

    const deserializedChallenge = Challenge.deserialize(challenge.serialize(), ackChallenge, peerId)

    assert(deserializedChallenge.solve(exponent), `Challenge must be solvable`)
  })
})
