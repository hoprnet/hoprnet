import { Acknowledgement } from './acknowledgement.js'
import { AcknowledgementChallenge } from './acknowledgementChallenge.js'
import { SECRET_LENGTH } from './constants.js'
import { randomBytes } from 'crypto'
import { deriveAckKeyShare, HalfKey } from '@hoprnet/hopr-utils'
import assert from 'assert'

import { createSecp256k1PeerId } from '@libp2p/peer-id-factory'

describe('acknowledement message', async function () {
  let [self, counterparty] = [await createSecp256k1PeerId(), await createSecp256k1PeerId()]
  it('create, serialize and deserialize', async function () {
    const ackKey = new HalfKey(Uint8Array.from(randomBytes(SECRET_LENGTH)))

    const challenge = AcknowledgementChallenge.create(ackKey.toChallenge(), self)

    assert(
      Acknowledgement.deserialize(
        Acknowledgement.create(challenge, ackKey, counterparty).serialize(),
        self,
        counterparty
      ) != null
    )
  })

  it('create, serialize and deserialize - false positives', async function () {
    const key = randomBytes(SECRET_LENGTH)

    assert.throws(() =>
      Acknowledgement.deserialize(
        Acknowledgement.create(
          randomBytes(AcknowledgementChallenge.SIZE) as any,
          deriveAckKeyShare(key),
          counterparty
        ).serialize(),
        self,
        counterparty
      )
    )
  })
})
