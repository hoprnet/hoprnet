import { Acknowledgement } from './acknowledgement.js'
import { AcknowledgementChallenge } from './acknowledgementChallenge.js'
import { SECRET_LENGTH } from './constants.js'
import { randomBytes } from 'crypto'
import { deriveAckKeyShare, HalfKey } from '@hoprnet/hopr-utils'
import assert from 'assert'

import { createPeerId } from '@libp2p/peer-id'

describe('acknowledement message', function () {
  let [self, counterparty] = createPeerId({ type: 'secp256k1' })
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
